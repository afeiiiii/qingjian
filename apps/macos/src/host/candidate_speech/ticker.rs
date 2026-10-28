//! 首选译文停键后的单次回调。

use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_foundation::{NSObject, NSObjectProtocol, NSTimer};

define_class!(
    // SAFETY: NSObject 没有子类化要求；所有回调都在主线程。
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ()]
    pub(super) struct CandidateSpeechTicker;

    impl CandidateSpeechTicker {
        #[unsafe(method(tick:))]
        fn tick(&self, timer: &NSTimer) {
            crate::host::with(|host| host.read_first_candidate(timer));
        }
    }

    unsafe impl NSObjectProtocol for CandidateSpeechTicker {}
);

impl CandidateSpeechTicker {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(());
        unsafe { msg_send![super(this), init] }
    }
}
