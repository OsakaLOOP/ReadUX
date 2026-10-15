//! 活用链配置的类型与词法条件匹配；语言规则集中在 morphology.json。
use std::{collections::BTreeMap, sync::LazyLock};
use serde::Deserialize;
use crate::{model::ProviderToken, morphology::ConnectionForm};

pub const SOURCE: &str = include_str!("../../../configs/unidic-nlp/morphology.json");
pub static CONFIG: LazyLock<Config> = LazyLock::new(|| serde_json::from_str(SOURCE).expect("活用链配置无效"));

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub operators: BTreeMap<String, Operator>,
    pub features: Vec<FeatureRule>,
    pub connections: Vec<ConnectionRule>,
    pub forms: Vec<FormRule>,
    pub insertion: Vec<TokenPattern>,
    pub presentation: serde_json::Value,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Operator {
    pub label: String,
    pub title: String,
    pub concept: String,
    pub prototype: String,
    pub variants: Vec<PrototypeVariant>,
    pub token_lemma: bool,
    pub normalized: Option<String>,
    pub candidates: Vec<String>,
    pub support: Option<Support>,
    pub compound_candidate: bool,
    pub connector: Option<String>,
    pub output_category: Option<String>,
    pub output_form: Option<ConnectionForm>,
    pub display_suffix: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrototypeVariant { pub token: TokenPattern, pub form: String }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Support {
    pub group: String,
    pub bases: Vec<String>,
    #[serde(default)]
    pub expanded: Option<[String; 2]>,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TokenPattern {
    pub equals: BTreeMap<TokenField, Vec<String>>,
    pub starts_with: BTreeMap<TokenField, Vec<String>>,
    pub contains: BTreeMap<TokenField, Vec<String>>,
}

#[derive(Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenField {
    #[serde(rename = "pos")] Pos,
    #[serde(rename = "sub")] Sub,
    #[serde(rename = "detail")] Detail,
    #[serde(rename = "ctype")] CType,
    #[serde(rename = "cform")] CForm,
    #[serde(rename = "lemma")] Lemma,
    #[serde(rename = "base")] Base,
    #[serde(rename = "surface")] Surface,
}

pub fn field(token: &ProviderToken, index: usize) -> &str {
    token.fields.get(index).and_then(|f| f.value.as_deref()).unwrap_or("")
}

pub fn identity(token: &ProviderToken) -> &str {
    let base = field(token, 10);
    if base.is_empty() { field(token, 7) } else { base }
}

impl TokenField {
    fn value<'a>(&self, token: &'a ProviderToken) -> &'a str {
        match self {
            Self::Pos => field(token, 0), Self::Sub => field(token, 1), Self::Detail => field(token, 2),
            Self::CType => field(token, 4), Self::CForm => field(token, 5), Self::Lemma => field(token, 7),
            Self::Base => identity(token), Self::Surface => &token.surface,
        }
    }
}

impl TokenPattern {
    pub fn matches(&self, token: &ProviderToken) -> bool {
        self.equals.iter().all(|(f, values)| values.iter().any(|v| f.value(token) == v))
            && self.starts_with.iter().all(|(f, values)| values.iter().any(|v| f.value(token).starts_with(v)))
            && self.contains.iter().all(|(f, values)| values.iter().any(|v| f.value(token).contains(v)))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureRule { pub kind: String, pub token: TokenPattern }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormRule { pub prefixes: Vec<String>, pub form: ConnectionForm }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionRule {
    pub action: Action,
    #[serde(default)] pub kinds: Vec<String>,
    #[serde(default)] pub support_group: String,
    pub when: Vec<ConnectionCondition>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action { Core, Attach, Support, Stop }

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConnectionCondition {
    pub next: TokenPattern,
    pub last: TokenPattern,
    pub forms: Vec<ConnectionForm>,
    pub categories: Vec<String>,
    pub operators: Vec<String>,
    pub suffixes: Vec<String>,
    pub functional: bool,
    pub auxiliary_relation: bool,
    pub sahen: bool,
}

pub fn definition(kind: &str) -> Option<&'static Operator> { CONFIG.operators.get(kind) }

pub fn is_inflection(kind: &str) -> bool {
    CONFIG.presentation["inflections"].as_array().unwrap().iter().any(|value| value.as_str() == Some(kind))
}

pub fn prototype(kind: &str, token: &ProviderToken) -> Option<String> {
    let entry = definition(kind)?;
    if let Some(form) = entry.variants.iter().find(|v| v.token.matches(token)) { return Some(form.form.clone()); }
    if entry.token_lemma && !field(token, 7).is_empty() { return Some(field(token, 7).into()); }
    (!entry.prototype.is_empty()).then(|| entry.prototype.clone())
}

pub fn features(token: &ProviderToken) -> Vec<String> {
    let mut kinds = Vec::new();
    for rule in &CONFIG.features {
        if rule.token.matches(token) && !kinds.contains(&rule.kind) { kinds.push(rule.kind.clone()); }
    }
    kinds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_resolve_within_the_single_configuration() {
        for rule in &CONFIG.features { assert!(CONFIG.operators.contains_key(&rule.kind), "{}", rule.kind); }
        for rule in &CONFIG.connections {
            assert!(!rule.when.is_empty());
            for kind in &rule.kinds { assert!(CONFIG.operators.contains_key(kind), "{kind}"); }
            if matches!(rule.action, Action::Support) {
                assert!(CONFIG.operators.values().any(|entry| entry.support.as_ref().is_some_and(|support| support.group == rule.support_group)));
            }
        }
        for entry in CONFIG.operators.values() {
            assert!(!entry.title.is_empty() && !entry.concept.is_empty());
            if let Some(connector) = &entry.connector { assert!(CONFIG.operators.contains_key(connector)); }
            for variant in &entry.variants {
                assert!(!variant.form.is_empty());
            }
        }
        assert!(CONFIG.presentation["inflections"].as_array().is_some());
    }
}
