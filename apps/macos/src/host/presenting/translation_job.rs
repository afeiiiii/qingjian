//! 一次选区翻译的请求身份、目标语言与替换范围。

use objc2_foundation::NSRange;
use qingjian_core::Language;

/// 一次「翻译选中文字」：从按下快捷键到用户接受或放弃。
#[derive(Debug, Clone)]
pub struct TranslationJob {
    /// 只接收本次请求的结果，避免联想或旧译文触发朗读。
    pub sequence: u64,

    /// 发出请求时的目标语言，不受等待期间的设置变化影响。
    pub language: Language,

    /// 选区在应用里的范围，接受时用译文替换它。
    pub range: NSRange,

    /// 云端回来的译文；`None` 表示还在等。
    pub result: Option<String>,
}
