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
    #[serde(default)] pub rule_metadata: BTreeMap<String, RuleMetadata>,
    #[serde(default)] pub entity_classes: BTreeMap<String, EntityClass>,
    #[serde(default)] pub rule_tree: Vec<RuleTreeNode>,
    pub features: Vec<FeatureRule>,
    pub connections: Vec<ConnectionRule>,
    #[serde(default)] pub sequences: Vec<Sequence>,
    #[serde(default)] pub predicate_links: Vec<PredicateLink>,
    pub forms: Vec<FormRule>,
    pub insertion: Vec<TokenPattern>,
    pub presentation: serde_json::Value,
    #[serde(default = "default_auxiliary_depth")] pub max_auxiliary_depth: usize,
    #[serde(default)] pub classical_forms: Vec<ClassicalForm>,
}

fn default_auxiliary_depth() -> usize { 8 }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassicalForm {
    pub lemma: String,
    pub class_name: String,
    pub forms: BTreeMap<String, String>,
    #[serde(default)] pub connection: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RuleMetadata {
    pub stage: String,
    pub executor: String,
    pub rule_group: String,
    pub input_category: String,
    pub output_category: String,
    pub terminal_mode: String,
    pub order_rank: i32,
    #[serde(default)] pub allowed_after: Vec<String>,
    #[serde(default)] pub same_layer: Vec<String>,
}

#[derive(Deserialize)]
pub struct EntityClass {
    pub label: String,
    pub executor: String,
    #[serde(default)] pub summary: String,
}

#[derive(Deserialize)]
pub struct RuleTreeNode {
    pub id: String,
    pub label: String,
    #[serde(default)] pub summary: String,
    #[serde(default)] pub input_examples: Vec<String>,
    #[serde(default)] pub output: String,
    #[serde(default)] pub children: Vec<RuleTreeNode>,
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
    pub preserve_state: bool,
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

pub fn support_kind(group: &str, token: &ProviderToken) -> Option<&'static str> {
    let potential = crate::morphology::potential_lemma(token);
    CONFIG.operators.iter().find(|(_, entry)| entry.support.as_ref().is_some_and(|support|
        support.group == group && support.bases.iter().any(|base| base == identity(token)
            || potential.as_ref().is_some_and(|query| &query.form == base))))
        .map(|(kind, _)| kind.as_str())
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
pub struct FeatureRule {
    pub kind: String,
    pub token: TokenPattern,
    #[serde(default)] pub exclude: Vec<TokenPattern>,
}

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
    pub following: Option<TokenPattern>,
    pub at_end: bool,
    pub forms: Vec<ConnectionForm>,
    pub categories: Vec<String>,
    pub operators: Vec<String>,
    pub suffixes: Vec<String>,
    pub functional: bool,
    pub auxiliary_relation: bool,
    pub sahen: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    pub kind: String,
    pub tokens: Vec<TokenPattern>,
    pub forms: Vec<ConnectionForm>,
    #[serde(default)] pub categories: Vec<String>,
    #[serde(default)] pub operators: Vec<String>,
    #[serde(default)] pub terminal_only: bool,
    #[serde(default)] pub phenomena: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateLink {
    pub prefix: String,
    pub next: TokenPattern,
    pub kind: String,
    pub required_operators: Vec<String>,
}

pub fn definition(kind: &str) -> Option<&'static Operator> { CONFIG.operators.get(kind) }

pub fn rule_metadata(kind: &str) -> Option<&'static RuleMetadata> {
    CONFIG.rule_metadata.get(kind)
}

/// 将来源 token 映射到六类实体。阶段和优先级不参与此判断。
pub fn entity_class_for_token(token: &ProviderToken) -> &'static str {
    let pos = field(token, 0);
    let sub = field(token, 1);
    let lemma = identity(token);
    if pos == "助詞" {
        if matches!(lemma, "て" | "で") && sub == "接続助詞" { return ""; }
        if matches!(lemma, "は" | "も" | "だけ" | "さえ" | "こそ" | "しか" | "すら" | "など" | "とか")
            || (sub == "副助詞" && matches!(lemma, "まで" | "なんて"))
        {
            return "插入语类";
        }
        return "";
    }
    if matches!(pos, "助動詞") {
        return match lemma {
            "ない" | "たい" | "らしい" => "イ形容词转换类",
            "そう" | "よう" | "みたい" | "べし" | "はず" => "ナ形容词判断类",
            "ず" | "ぬ" | "ごとし" | "ごとき" => "文言活用类",
            _ => "动词循环类",
        };
    }
    if pos == "動詞" && (sub.contains("非自立") || CONFIG.operators.values().any(|entry| entry.support.as_ref().is_some_and(|support| support.bases.iter().any(|base| base == lemma)))) {
        return "补助动词类";
    }
    if pos == "形容詞" { return "イ形容词转换类"; }
    if pos == "形状詞" { return "ナ形容词判断类"; }
    if pos == "動詞" { return "动词循环类"; }
    "动词循环类"
}

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
        if rule.token.matches(token) && !rule.exclude.iter().any(|pattern| pattern.matches(token))
            && !kinds.contains(&rule.kind) { kinds.push(rule.kind.clone()); }
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
        for (kind, metadata) in &CONFIG.rule_metadata {
            assert!(!metadata.stage.is_empty(), "{kind} 缺少阶段");
            assert!(!metadata.executor.is_empty(), "{kind} 缺少执行器");
        }
        assert_eq!(CONFIG.rule_metadata.get("nominalization").map(|metadata| metadata.executor.as_str()), Some("nominalization"));
    }

    #[test]
    fn classifies_only_declared_particles_as_insertion_entities() {
        let mut values = vec![None; crate::model::FIELD_NAMES.len()];
        let set = |values: &mut Vec<Option<String>>, name: &str, value: &str| {
            let index = crate::model::FIELD_NAMES.iter().position(|item| *item == name).unwrap();
            values[index] = Some(value.into());
        };
        set(&mut values, "pos1", "助詞");
        set(&mut values, "pos2", "接続助詞");
        set(&mut values, "lemma", "て");
        set(&mut values, "orthBase", "て");
        let mut token = ProviderToken { index: 0, surface: "て".into(), char_range: [0, 1], byte_range: [0, 3], lexicon_type: "test".into(), left_id: 0, right_id: 0, word_cost: 0, total_cost: 0, raw_feature: String::new(), fields: values.into_iter().enumerate().map(|(index, value)| crate::model::FeatureField { index, name: crate::model::FIELD_NAMES[index].into(), label: crate::model::FIELD_LABELS[index].into(), raw: value.clone(), value }).collect() };
        assert_eq!(entity_class_for_token(&token), "");
        token.fields[7].value = Some("は".into());
        token.fields[10].value = Some("は".into());
        token.surface = "は".into();
        token.fields[1].value = Some("係助詞".into());
        assert_eq!(entity_class_for_token(&token), "插入语类");
        token.fields[1].value = Some("格助詞".into());
        token.fields[7].value = Some("に".into());
        token.fields[10].value = Some("に".into());
        token.surface = "に".into();
        assert_eq!(entity_class_for_token(&token), "");
    }
}
