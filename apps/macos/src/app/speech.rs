//! 选区译文的系统朗读；合成器延迟创建，始终只播放最新一次翻译。

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_avf_audio::{
    AVSpeechBoundary, AVSpeechSynthesisVoice, AVSpeechSynthesizer, AVSpeechUtterance,
};
use objc2_foundation::NSString;
use qingjian_core::Language;

pub struct Speech {
    synthesizer: Option<Retained<AVSpeechSynthesizer>>,

    /// 与输入法会话一样，只在主线程创建和控制语音。
    _main_thread: MainThreadMarker,
}

impl Speech {
    pub fn new(mtm: MainThreadMarker) -> Self {
        Self {
            synthesizer: None,
            _main_thread: mtm,
        }
    }

    pub fn speak(&mut self, text: &str, language: Language) {
        self.stop();
        if text.trim().is_empty() {
            return;
        }
        let locale = match language {
            Language::English => "en-US",
            Language::Chinese => "zh-CN",
            Language::Japanese => "ja-JP",
            Language::Spanish => "es-ES",
            Language::Vietnamese => "vi-VN",
            Language::German => "de-DE",
        };
        // SAFETY: 字符串与合成器均有效，所有语音调用都在主线程，合成器持有到播放结束。
        unsafe {
            let Some(voice) =
                AVSpeechSynthesisVoice::voiceWithLanguage(Some(&NSString::from_str(locale)))
            else {
                tracing::warn!(locale, "No system speech voice available for translation");
                return;
            };
            let utterance = AVSpeechUtterance::speechUtteranceWithString(&NSString::from_str(text));
            utterance.setVoice(Some(&voice));
            let synthesizer = self
                .synthesizer
                .get_or_insert_with(|| AVSpeechSynthesizer::new());
            synthesizer.speakUtterance(&utterance);
        }
    }

    pub fn stop(&self) {
        if let Some(synthesizer) = &self.synthesizer {
            // SAFETY: 合成器在主线程创建，当前仍持有它。
            unsafe { synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate) };
        }
    }
}
