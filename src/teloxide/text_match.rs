pub enum TextMatch {
    Verbatim(&'static str),
    Prefix(&'static str),
    Inner(&'static str),
}

impl TextMatch {
    pub fn matches(&self, text: &str) -> bool {
        match self {
            TextMatch::Verbatim(match_text) => text == *match_text,
            TextMatch::Prefix(match_text) => text.starts_with(match_text),
            TextMatch::Inner(match_text) => text.contains(match_text),
        }
    }

    pub fn find<T: Clone>(matches: &[(T, TextMatch)], text: &str) -> Option<T> {
        for (button, match_text) in matches {
            if match_text.matches(text) {
                return Some(button.clone());
            }
        }
        None
    }
}
