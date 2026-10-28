//! 首选英文译文的防抖与朗读；独立持有语音，取消候选不会打断已接受的选区译文。

mod state;
mod ticker;

use objc2::rc::Retained;
use objc2::{MainThreadMarker, sel};
use objc2_foundation::NSTimer;
use qingjian_core::{Cell, Language};

use self::state::State;
use self::ticker::CandidateSpeechTicker;
use super::Host;
use crate::app::Speech;
use crate::imk::secure_input;

const DEBOUNCE_SECONDS: f64 = 0.4;

pub struct CandidateSpeech {
    state: State,

    timer: Option<Retained<NSTimer>>,

    speech: Speech,

    mtm: MainThreadMarker,
}

impl CandidateSpeech {
    pub fn new(mtm: MainThreadMarker) -> Self {
        Self {
            state: State::default(),
            timer: None,
            speech: Speech::new(mtm),
            mtm,
        }
    }

    fn update(&mut self, text: Option<String>, keys: &str) {
        if self.state.text() != text.as_deref() {
            self.speech.stop();
        }
        if !self.state.update(text, keys) {
            return;
        }
        self.stop_timer();
        if self.state.pending().is_some() {
            let target = CandidateSpeechTicker::new(self.mtm);
            // SAFETY: 单次定时器与回调目标均在主线程创建。
            self.timer = Some(unsafe {
                NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                    DEBOUNCE_SECONDS,
                    &target,
                    sel!(tick:),
                    None,
                    false,
                )
            });
        }
    }

    pub fn cancel(&mut self) {
        self.stop_timer();
        self.state = State::default();
        self.speech.stop();
    }

    fn stop_timer(&mut self) {
        if let Some(timer) = self.timer.take() {
            timer.invalidate();
        }
    }
}

impl Host {
    fn first_candidate_english(&self) -> Option<String> {
        if self.translation.is_some()
            || self.notice.is_some()
            || self.engine.composition().is_empty()
            || self.engine.is_private()
            || secure_input::enabled()
            || self.engine.symbol_panel().is_some()
        {
            return None;
        }
        let cells = self
            .session
            .grid_cells()
            .map_or_else(|| self.session.page_cells(), |(cells, _)| cells);
        let candidate = cells.into_iter().find_map(Cell::candidate)?;
        let translation = candidate.translation.as_ref()?;
        if translation.language != Language::English {
            return None;
        }
        let text = translation.senses().first()?.text.trim();
        (!text.is_empty()).then(|| text.to_owned())
    }

    pub(super) fn update_candidate_speech(&mut self) {
        let text = self.first_candidate_english();
        self.candidate_speech
            .update(text, self.engine.composition().text());
    }

    fn read_first_candidate(&mut self, timer: &NSTimer) {
        if self
            .candidate_speech
            .timer
            .as_deref()
            .is_none_or(|current| !std::ptr::eq(current, timer))
        {
            return;
        }
        let text = self.first_candidate_english();
        let keys = self.engine.composition().text();
        if !self.candidate_speech.state.matches(text.as_deref(), keys) {
            self.candidate_speech.update(text, keys);
            return;
        }
        self.candidate_speech.stop_timer();
        if let Some(text) = self.candidate_speech.state.take() {
            self.speech.stop();
            self.candidate_speech.speech.speak(&text, Language::English);
        }
    }
}
