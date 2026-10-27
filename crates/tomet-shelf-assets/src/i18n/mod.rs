//! UI strings and internationalization dictionaries for tomet-shelf-assets.

pub mod en;
pub mod ja;

use std::collections::HashMap;

/// Returns true if the given language code represents English (e.g. "en", "en-US").
pub fn is_english(lang: &str) -> bool {
    tomet_shelf_config::is_english(lang)
}

/// Look up default UI strings for a given book language code.
pub fn default_ui_strings_for(lang: &str) -> HashMap<String, String> {
    if is_english(lang) {
        en::ui_strings()
    } else {
        ja::ui_strings()
    }
}

/// Populate missing UI strings in `strings` based on `lang`.
pub fn apply_ui_defaults(strings: &mut HashMap<String, String>, lang: &str) {
    for (k, v) in default_ui_strings_for(lang) {
        strings.entry(k).or_insert(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_and_japanese_ui_dictionaries_have_identical_keys() {
        let ja_strings = ja::ui_strings();
        let en_strings = en::ui_strings();
        assert_eq!(ja_strings.len(), en_strings.len());
        for key in ja_strings.keys() {
            assert!(
                en_strings.contains_key(key),
                "Missing English UI string key: {key}"
            );
        }
    }
}
