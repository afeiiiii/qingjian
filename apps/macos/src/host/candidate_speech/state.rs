//! 首选译文的去重状态：按键变化延后待播，同一译文在一次连续输入中只读一次。

#[derive(Default)]
pub(super) struct State {
    text: Option<String>,

    keys: String,

    spoken: bool,
}

impl State {
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub fn matches(&self, text: Option<&str>, keys: &str) -> bool {
        self.text() == text && self.keys == keys
    }

    pub fn update(&mut self, text: Option<String>, keys: &str) -> bool {
        if self.matches(text.as_deref(), keys) {
            return false;
        }
        if self.text != text {
            self.spoken = false;
        }
        self.text = text;
        self.keys = keys.to_owned();
        true
    }

    pub fn pending(&self) -> Option<&str> {
        self.text().filter(|_| !self.spoken)
    }

    pub fn take(&mut self) -> Option<String> {
        let text = self.pending()?.to_owned();
        self.spoken = true;
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn redraw_does_not_restart_or_repeat_speech() {
        let mut state = State::default();
        assert!(state.update(Some("I".into()), "wo"));
        assert!(!state.update(Some("I".into()), "wo"));
        assert_eq!(state.take().as_deref(), Some("I"));
        assert!(!state.update(Some("I".into()), "wo"));
        assert_eq!(state.take(), None);
    }

    #[test]
    fn typing_delays_pending_speech_but_does_not_repeat_spoken_text() {
        let mut state = State::default();
        state.update(Some("input".into()), "shur");
        assert!(state.update(Some("input".into()), "shuru"));
        assert_eq!(state.pending(), Some("input"));
        state.take();
        assert!(state.update(Some("input".into()), "shurur"));
        assert_eq!(state.pending(), None);
    }

    #[test]
    fn untranslated_candidate_discards_the_previous_word() {
        let mut state = State::default();
        state.update(Some("I".into()), "wo");
        assert!(state.update(None, "woshuru"));
        assert_eq!(state.take(), None);
        assert!(!state.matches(Some("I"), "wo"));
    }

    #[test]
    fn new_word_and_new_composition_can_be_read_again() {
        let mut state = State::default();
        state.update(Some("I".into()), "wo");
        state.take();
        state.update(Some("input".into()), "shuru");
        assert_eq!(state.take().as_deref(), Some("input"));
        state.update(None, "");
        state.update(Some("input".into()), "shuru");
        assert_eq!(state.take().as_deref(), Some("input"));
    }
}
