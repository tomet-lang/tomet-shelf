use std::collections::HashMap;
use tomet_shelf_config::HeroChipConfig;

/// Default hero chips in Japanese.
pub fn hero_chips_ja() -> Vec<HeroChipConfig> {
    vec![
        HeroChipConfig {
            key: "parent".to_string(),
            label: Some("所属".to_string()),
            format: None,
        },
        HeroChipConfig {
            key: "type.list-a".to_string(),
            label: Some("ファン".to_string()),
            format: None,
        },
        HeroChipConfig {
            key: "type.element-a".to_string(),
            label: None,
            format: None,
        },
        HeroChipConfig {
            key: "birth".to_string(),
            label: Some("🎂".to_string()),
            format: Some("birthday".to_string()),
        },
    ]
}

/// Default hero chips in English.
pub fn hero_chips_en() -> Vec<HeroChipConfig> {
    vec![
        HeroChipConfig {
            key: "parent".to_string(),
            label: Some("Affiliation".to_string()),
            format: None,
        },
        HeroChipConfig {
            key: "type.list-a".to_string(),
            label: Some("Fans".to_string()),
            format: None,
        },
        HeroChipConfig {
            key: "type.element-a".to_string(),
            label: None,
            format: None,
        },
        HeroChipConfig {
            key: "birth".to_string(),
            label: Some("🎂".to_string()),
            format: Some("birthday".to_string()),
        },
    ]
}

/// Look up default hero chips for a given book language code.
pub fn default_hero_chips_for(lang: &str) -> Vec<HeroChipConfig> {
    if tomet_shelf_config::is_english(lang) {
        hero_chips_en()
    } else {
        hero_chips_ja()
    }
}

/// Default infobox key labels in Japanese.
pub fn key_labels_ja() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("kind".to_string(), "種別".to_string());
    m.insert("type".to_string(), "種別".to_string());
    m.insert("birth".to_string(), "誕生日".to_string());
    m.insert("height".to_string(), "身長".to_string());
    m.insert("creator".to_string(), "デザイン".to_string());
    m.insert("type.list-a".to_string(), "ファンネーム".to_string());
    m.insert("type.element-a".to_string(), "属性 / MBTI".to_string());
    m.insert("colors".to_string(), "カラー".to_string());
    m.insert("parent".to_string(), "所属ユニット".to_string());
    m.insert("group".to_string(), "所属組織".to_string());
    m.insert("place".to_string(), "出身地".to_string());
    m.insert("created".to_string(), "作成日".to_string());
    m.insert("modified".to_string(), "更新日".to_string());
    m.insert("reviewed".to_string(), "確認日".to_string());
    m.insert("rating".to_string(), "評価".to_string());
    m.insert("aliases".to_string(), "別名 / 愛称".to_string());
    m.insert("topics".to_string(), "トピック".to_string());
    m.insert("flags".to_string(), "フラグ".to_string());
    m.insert("published".to_string(), "公開日".to_string());
    m.insert("url.wiki".to_string(), "Wikipedia".to_string());
    m.insert("url.else".to_string(), "関連リンク".to_string());
    m
}

/// Default infobox key labels in English.
pub fn key_labels_en() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("kind".to_string(), "Kind".to_string());
    m.insert("type".to_string(), "Type".to_string());
    m.insert("birth".to_string(), "Birthday".to_string());
    m.insert("height".to_string(), "Height".to_string());
    m.insert("creator".to_string(), "Creator".to_string());
    m.insert("type.list-a".to_string(), "Fan Name".to_string());
    m.insert("type.element-a".to_string(), "Element / MBTI".to_string());
    m.insert("colors".to_string(), "Colors".to_string());
    m.insert("parent".to_string(), "Affiliation".to_string());
    m.insert("group".to_string(), "Organization".to_string());
    m.insert("place".to_string(), "Origin".to_string());
    m.insert("created".to_string(), "Created".to_string());
    m.insert("modified".to_string(), "Modified".to_string());
    m.insert("reviewed".to_string(), "Reviewed".to_string());
    m.insert("rating".to_string(), "Rating".to_string());
    m.insert("aliases".to_string(), "Aliases / Nicknames".to_string());
    m.insert("topics".to_string(), "Topics".to_string());
    m.insert("flags".to_string(), "Flags".to_string());
    m.insert("published".to_string(), "Published".to_string());
    m.insert("url.wiki".to_string(), "Wikipedia".to_string());
    m.insert("url.else".to_string(), "Related Links".to_string());
    m
}

/// Look up default infobox labels for a given book language code.
pub fn default_key_labels_for(lang: &str) -> HashMap<String, String> {
    if tomet_shelf_config::is_english(lang) {
        key_labels_en()
    } else {
        key_labels_ja()
    }
}

/// Resolve label for a given infobox key, falling back to default language labels and then the key itself.
pub fn resolve_label<'a>(
    configured_labels: &'a HashMap<String, String>,
    default_labels: &'a HashMap<String, String>,
    key: &str,
) -> String {
    if let Some(lbl) = configured_labels.get(key) {
        return lbl.clone();
    }
    if let Some(lbl) = default_labels.get(key) {
        return lbl.clone();
    }
    key.to_string()
}
