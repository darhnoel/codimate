use crate::LogoAnimation;

#[derive(Clone, Debug, PartialEq)]
pub enum LogoAnimationEvent {
    IntroduceKeyboard,
    IntroduceLatinKey,
    KeyboardImpulse,
    TransliterateToKhmer,
    HoldKhmerLogo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogoAnimationTrace {
    pub events: Vec<LogoAnimationEvent>,
}

pub fn logo_animation_algorithm(_state: LogoAnimation) -> LogoAnimationTrace {
    LogoAnimationTrace {
        events: vec![
            LogoAnimationEvent::IntroduceKeyboard,
            LogoAnimationEvent::IntroduceLatinKey,
            LogoAnimationEvent::KeyboardImpulse,
            LogoAnimationEvent::TransliterateToKhmer,
            LogoAnimationEvent::HoldKhmerLogo,
        ],
    }
}
