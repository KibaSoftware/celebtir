//! LCH-3: words that turn a configuration fact into an accusation. Generated
//! text describes what the chain allows, never intent.

pub const BANNED_WORDS: &[&str] = &["scam", "unsafe", "rug", "rugpull", "fraud", "ponzi"];

/// The first banned word in `text`, matched as a whole word, case-insensitive.
pub fn banned_word(text: &str) -> Option<&'static str> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .find_map(|w| {
            BANNED_WORDS
                .iter()
                .copied()
                .find(|b| w.eq_ignore_ascii_case(b))
        })
}
