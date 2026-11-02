//! 词元接续资格。原始词性与本句活用状态分别保存。
use crate::{model::ProviderToken, morphology_config::{field, CONFIG}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Capabilities(pub u8);

impl Capabilities {
    pub const NONE: u8 = 1;
    pub const SURU: u8 = 2;
    pub const NA_DA: u8 = 4;
    pub const TO_SURU: u8 = 8;
    pub const TARI: u8 = 16;

    pub fn has(self, flag: u8) -> bool { self.0 & flag != 0 }
    pub fn inflecting(self) -> bool { self.0 & 30 != 0 }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexemeCapabilities {
    pub lemma: String,
    pub reading: String,
    pub flags: Capabilities,
}

pub fn for_token(token: &ProviderToken) -> Option<Capabilities> {
    if !matches!(field(token, 0), "副詞" | "形状詞" | "名詞") { return None; }
    let lemma = field(token, 7);
    let reading = field(token, 6);
    let id = field(token, 28);
    let mapped = if id.is_empty() {
        CONFIG.adverb_capabilities.values().find(|entry| entry.lemma == lemma && entry.reading == reading)
    } else {
        CONFIG.adverb_capabilities.get(id)
            .filter(|entry| entry.lemma == lemma && entry.reading == reading)
    };
    if let Some(entry) = mapped { return Some(entry.flags); }
    match (field(token, 0), field(token, 1), field(token, 2)) {
        ("形状詞", "タリ", _) => Some(Capabilities(Capabilities::TARI | Capabilities::TO_SURU)),
        ("形状詞", "一般", _) => Some(Capabilities(Capabilities::NA_DA)),
        ("名詞", "普通名詞", detail) if detail.contains("サ変") => {
            let flags = Capabilities::SURU | if detail.contains("形状詞") { Capabilities::NA_DA } else { 0 };
            Some(Capabilities(flags))
        }
        _ => None,
    }
}

pub fn is_suru(token: &ProviderToken) -> bool {
    field(token, 0) == "動詞" && field(token, 4).contains("サ行変格")
        && matches!(crate::morphology_config::identity(token), "する" | "為る")
}
