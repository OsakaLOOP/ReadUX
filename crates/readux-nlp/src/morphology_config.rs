//! 活用链配置的类型与词法条件匹配；语言规则集中在 morphology.json。
use crate::{model::ProviderToken, morphology::ConnectionForm};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};

pub const SOURCE: &str = include_str!("../../../configs/unidic-nlp/morphology.json");
pub static CONFIG: LazyLock<Config> =
    LazyLock::new(|| serde_json::from_str(SOURCE).expect("活用链配置无效"));

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub operators: BTreeMap<String, Operator>,
    #[serde(default)]
    pub rule_metadata: BTreeMap<String, RuleMetadata>,
    #[serde(default)]
    pub entity_classes: BTreeMap<String, EntityClass>,
    #[serde(default)]
    pub rule_tree: Vec<RuleTreeNode>,
    pub features: Vec<FeatureRule>,
    pub connections: Vec<ConnectionRule>,
    #[serde(default)]
    pub sequences: Vec<Sequence>,
    #[serde(default)]
    pub predicate_links: Vec<PredicateLink>,
    pub forms: Vec<FormRule>,
    pub insertion: Vec<TokenPattern>,
    #[serde(default)]
    pub insertion_positions: Vec<InsertionPosition>,
    pub presentation: serde_json::Value,
    #[serde(default = "default_auxiliary_depth")]
    pub max_auxiliary_depth: usize,
    #[serde(default)]
    pub classical_forms: Vec<ClassicalForm>,
    #[serde(default)]
    pub classical_auxiliaries: Vec<ClassicalAuxiliary>,
}

fn default_auxiliary_depth() -> usize {
    8
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassicalForm {
    pub lemma: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub lemma_suffix: String,
    pub ctype: Vec<String>,
    pub class_name: String,
    pub forms: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub connection: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassicalAuxiliary {
    pub lemma: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub rule: String,
    pub class_name: String,
    #[serde(default)]
    pub ctype: Vec<String>,
    pub forms: BTreeMap<String, Vec<String>>,
    pub connection: Vec<String>,
    pub inputs: Vec<ClassicalInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassicalInput {
    pub categories: Vec<String>,
    pub forms: Vec<ConnectionForm>,
    #[serde(default)]
    pub ctype: Vec<String>,
    #[serde(default)]
    pub exclude_ctype: Vec<String>,
}

impl ClassicalAuxiliary {
    pub fn form(
        &self,
        lemma: &str,
        surface: &str,
        ctype: &str,
        cform: &str,
    ) -> Option<&'static str> {
        let form = classical_form_name(cform)?;
        ((self.ctype.is_empty() || self.ctype.iter().any(|prefix| ctype.starts_with(prefix)))
            && (self.lemma == lemma || self.aliases.iter().any(|alias| alias == lemma))
            && self.forms.get(form)?.iter().any(|value| value == surface))
        .then_some(form)
    }
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
    #[serde(default)]
    pub input_categories: Vec<String>,
    #[serde(default)]
    pub input_forms: Vec<ConnectionForm>,
    #[serde(default)]
    pub allowed_after: Vec<String>,
    #[serde(default)]
    pub same_layer: Vec<String>,
}

#[derive(Deserialize)]
pub struct EntityClass {
    pub label: String,
    pub executor: String,
    #[serde(default)]
    pub summary: String,
}

#[derive(Deserialize)]
pub struct RuleTreeNode {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub input_examples: Vec<String>,
    #[serde(default)]
    pub output: String,
    #[serde(default)]
    pub children: Vec<RuleTreeNode>,
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
pub struct PrototypeVariant {
    pub token: TokenPattern,
    pub form: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Support {
    pub group: String,
    pub bases: Vec<String>,
    #[serde(default)]
    pub expanded: Option<[String; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InsertionPosition {
    pub id: String,
    #[serde(default)]
    pub connector: String,
    #[serde(default)]
    pub support_group: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub forms: Vec<ConnectionForm>,
    #[serde(default)]
    pub bases: Vec<String>,
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
    #[serde(rename = "pos")]
    Pos,
    #[serde(rename = "sub")]
    Sub,
    #[serde(rename = "detail")]
    Detail,
    #[serde(rename = "ctype")]
    CType,
    #[serde(rename = "cform")]
    CForm,
    #[serde(rename = "lemma")]
    Lemma,
    #[serde(rename = "base")]
    Base,
    #[serde(rename = "surface")]
    Surface,
}

pub fn field(token: &ProviderToken, index: usize) -> &str {
    token
        .fields
        .get(index)
        .and_then(|f| f.value.as_deref())
        .unwrap_or("")
}

pub fn identity(token: &ProviderToken) -> &str {
    let base = field(token, 10);
    if base.is_empty() {
        field(token, 7)
    } else {
        base
    }
}

pub fn support_kind(group: &str, token: &ProviderToken) -> Option<&'static str> {
    let potential = crate::morphology::potential_lemma(token);
    CONFIG
        .operators
        .iter()
        .find(|(_, entry)| {
            entry.support.as_ref().is_some_and(|support| {
                support.group == group
                    && support.bases.iter().any(|base| {
                        base == identity(token)
                            || potential.as_ref().is_some_and(|query| &query.form == base)
                    })
            })
        })
        .map(|(kind, _)| kind.as_str())
}

pub fn is_insertion(token: &ProviderToken) -> bool {
    CONFIG
        .insertion
        .iter()
        .any(|pattern| pattern.matches(token))
}

impl TokenField {
    fn value<'a>(&self, token: &'a ProviderToken) -> &'a str {
        match self {
            Self::Pos => field(token, 0),
            Self::Sub => field(token, 1),
            Self::Detail => field(token, 2),
            Self::CType => field(token, 4),
            Self::CForm => field(token, 5),
            Self::Lemma => field(token, 7),
            Self::Base => identity(token),
            Self::Surface => &token.surface,
        }
    }
}

impl TokenPattern {
    pub fn matches(&self, token: &ProviderToken) -> bool {
        self.equals
            .iter()
            .all(|(f, values)| values.iter().any(|v| f.value(token) == v))
            && self
                .starts_with
                .iter()
                .all(|(f, values)| values.iter().any(|v| f.value(token).starts_with(v)))
            && self
                .contains
                .iter()
                .all(|(f, values)| values.iter().any(|v| f.value(token).contains(v)))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureRule {
    pub kind: String,
    pub token: TokenPattern,
    #[serde(default)]
    pub exclude: Vec<TokenPattern>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormRule {
    pub prefixes: Vec<String>,
    pub form: ConnectionForm,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionRule {
    pub action: Action,
    #[serde(default)]
    pub kinds: Vec<String>,
    #[serde(default)]
    pub support_group: String,
    pub when: Vec<ConnectionCondition>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Core,
    Attach,
    Support,
    Stop,
}

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
    #[serde(default)]
    pub head: usize,
    pub forms: Vec<ConnectionForm>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub operators: Vec<String>,
    #[serde(default)]
    pub terminal_only: bool,
    #[serde(default)]
    pub phenomena: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateLink {
    pub prefix: String,
    pub next: TokenPattern,
    pub kind: String,
    pub required_operators: Vec<String>,
}

pub fn definition(kind: &str) -> Option<&'static Operator> {
    CONFIG.operators.get(kind)
}

pub fn rule_metadata(kind: &str) -> Option<&'static RuleMetadata> {
    CONFIG.rule_metadata.get(kind)
}

pub fn classical_form_name(cform: &str) -> Option<&'static str> {
    [
        ("未然形", "未然形"),
        ("連用形", "連用形"),
        ("終止形", "終止形"),
        ("連体形", "連体形"),
        ("已然形", "已然形"),
        ("仮定形", "已然形"),
        ("命令形", "命令形"),
        ("基本形", "終止形"),
    ]
    .into_iter()
    .find_map(|(prefix, form)| cform.starts_with(prefix).then_some(form))
}

pub fn classical_connection_form(cform: &str) -> Option<ConnectionForm> {
    Some(match classical_form_name(cform)? {
        "未然形" => ConnectionForm::Irrealis,
        "連用形" => ConnectionForm::Continuative,
        "終止形" => ConnectionForm::Terminal,
        "連体形" => ConnectionForm::Attributive,
        "已然形" => ConnectionForm::Conditional,
        "命令形" => ConnectionForm::Imperative,
        _ => return None,
    })
}

pub fn classical_paradigm_form(
    token: &ProviderToken,
) -> Option<(&'static ClassicalForm, &'static str)> {
    classical_paradigm_match(
        &[identity(token), field(token, 7)],
        &token.surface,
        field(token, 4),
        field(token, 5),
    )
}

pub fn classical_paradigm_match(
    lemmas: &[&str],
    surface: &str,
    ctype: &str,
    cform: &str,
) -> Option<(&'static ClassicalForm, &'static str)> {
    let form_name = classical_form_name(cform)?;
    CONFIG.classical_forms.iter().find_map(|entry| {
        lemmas
            .iter()
            .copied()
            .filter(|lemma| !lemma.is_empty())
            .find_map(|lemma| {
                let stem = lemma.strip_suffix(&entry.lemma_suffix)?;
                let known_lemma = entry.lemma == lemma
                    || entry.aliases.iter().any(|alias| alias == lemma)
                    || entry.ctype.iter().any(|prefix| ctype.starts_with(prefix));
                if !known_lemma {
                    return None;
                }
                let suffixes = entry.forms.get(form_name)?;
                suffixes
                    .iter()
                    .any(|suffix| surface == format!("{stem}{suffix}"))
                    .then_some((entry, form_name))
            })
    })
}

pub fn classical_auxiliary_form(
    lemma: &str,
    surface: &str,
    ctype: &str,
    cform: &str,
) -> Option<(&'static ClassicalAuxiliary, &'static str)> {
    CONFIG.classical_auxiliaries.iter().find_map(|entry| {
        entry
            .form(lemma, surface, ctype, cform)
            .map(|form| (entry, form))
    })
}

pub fn classical_auxiliary(
    token: &ProviderToken,
) -> Option<(&'static ClassicalAuxiliary, &'static str)> {
    if field(token, 0) != "助動詞"
        && !(field(token, 0) == "形容詞" && matches!(identity(token), "ごとし" | "如し"))
    {
        return None;
    }
    let lemma = [identity(token), field(token, 7)]
        .into_iter()
        .find(|candidate| {
            !candidate.is_empty()
                && CONFIG.classical_auxiliaries.iter().any(|entry| {
                    (entry.lemma == *candidate
                        || entry.aliases.iter().any(|alias| alias == candidate))
                        && (entry.ctype.is_empty()
                            || entry
                                .ctype
                                .iter()
                                .any(|prefix| field(token, 4).starts_with(prefix)))
                })
        })?;
    classical_auxiliary_form(&lemma, &token.surface, field(token, 4), field(token, 5))
}

pub fn classical_accepts(
    entry: &ClassicalAuxiliary,
    state: &crate::morphology::MorphologyState,
) -> bool {
    entry.inputs.iter().any(|input| {
        input.categories.contains(&state.category)
            && input.forms.contains(&state.form)
            && (input.ctype.is_empty()
                || input
                    .ctype
                    .iter()
                    .any(|prefix| state.conjugation_type.starts_with(prefix)))
            && !input
                .exclude_ctype
                .iter()
                .any(|prefix| state.conjugation_type.starts_with(prefix))
    })
}

pub fn classical_rule_for_input(
    token: &ProviderToken,
    state: &crate::morphology::MorphologyState,
) -> Option<&'static str> {
    CONFIG
        .classical_auxiliaries
        .iter()
        .find(|entry| {
            [identity(token), field(token, 7)].iter().any(|lemma| {
                entry
                    .form(lemma, &token.surface, field(token, 4), field(token, 5))
                    .is_some()
            }) && classical_accepts(entry, state)
        })
        .map(|entry| entry.rule.as_str())
}

/// 将来源 token 映射到六类实体。阶段和优先级不参与此判断。
pub fn entity_class_for_token(token: &ProviderToken) -> &'static str {
    let pos = field(token, 0);
    let sub = field(token, 1);
    let lemma = identity(token);
    if is_insertion(token) {
        return "插入语类";
    }
    if pos == "助詞" {
        return "";
    }
    if matches!(pos, "助動詞") {
        if classical_auxiliary(token).is_some() {
            return "文言活用类";
        }
        return match lemma {
            "ない" | "たい" | "らしい" => "イ形容词转换类",
            "そう" | "よう" | "みたい" | "はず" => "ナ形容词判断类",
            "べし" | "ず" | "ぬ" | "ごとし" | "ごとき" => "文言活用类",
            _ => "动词循环类",
        };
    }
    if matches!(pos, "形容詞" | "形状詞") && classical_paradigm_form(token).is_some() {
        return "文言活用类";
    }
    if pos == "動詞"
        && (sub.contains("非自立")
            || CONFIG.operators.values().any(|entry| {
                entry
                    .support
                    .as_ref()
                    .is_some_and(|support| support.bases.iter().any(|base| base == lemma))
            }))
    {
        return "补助动词类";
    }
    if pos == "形容詞" {
        return "イ形容词转换类";
    }
    if pos == "形状詞" {
        return "ナ形容词判断类";
    }
    if pos == "動詞" {
        return "动词循环类";
    }
    "动词循环类"
}

pub fn is_inflection(kind: &str) -> bool {
    CONFIG.presentation["inflections"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value.as_str() == Some(kind))
}

pub fn prototype(kind: &str, token: &ProviderToken) -> Option<String> {
    let entry = definition(kind)?;
    if let Some(form) = entry.variants.iter().find(|v| v.token.matches(token)) {
        return Some(form.form.clone());
    }
    if entry.token_lemma && !field(token, 7).is_empty() {
        return Some(field(token, 7).into());
    }
    (!entry.prototype.is_empty()).then(|| entry.prototype.clone())
}

pub fn features(token: &ProviderToken) -> Vec<String> {
    if let Some((entry, _)) = classical_auxiliary(token) {
        return vec![entry.rule.clone()];
    }
    let mut kinds = Vec::new();
    for rule in &CONFIG.features {
        if rule.token.matches(token)
            && !rule.exclude.iter().any(|pattern| pattern.matches(token))
            && !kinds.contains(&rule.kind)
        {
            kinds.push(rule.kind.clone());
        }
    }
    if classical_paradigm_form(token).is_some()
        && !kinds.iter().any(|kind| kind == "classical_adjective")
    {
        kinds.push("classical_adjective".into());
    }
    kinds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_resolve_within_the_single_configuration() {
        assert!(!CONFIG.insertion.is_empty());
        assert!(!CONFIG.insertion_positions.is_empty());
        for position in &CONFIG.insertion_positions {
            assert!(!position.id.is_empty());
            if position.connector.is_empty() {
                assert!(!position.categories.is_empty(), "{}", position.id);
                assert!(!position.forms.is_empty(), "{}", position.id);
                assert!(!position.bases.is_empty(), "{}", position.id);
            } else {
                assert_eq!(
                    CONFIG.rule_metadata[&position.connector].executor, "auxiliary_verb",
                    "{}",
                    position.id
                );
                assert!(CONFIG.operators.values().any(|operator| operator
                    .support
                    .as_ref()
                    .is_some_and(|support| support.group == position.support_group)));
            }
        }
        for sequence in &CONFIG.sequences {
            assert!(
                sequence.head < sequence.tokens.len(),
                "{} 的序列核心越界",
                sequence.kind
            );
        }
        for entry in &CONFIG.classical_auxiliaries {
            assert!(
                !entry.forms.is_empty() && !entry.inputs.is_empty(),
                "{}",
                entry.rule
            );
            assert_eq!(
                CONFIG.rule_metadata[&entry.rule].executor,
                "classical_conjugation"
            );
            assert!(CONFIG.operators[&entry.rule]
                .concept
                .starts_with("grammar.classical."));
        }
        for rule in &CONFIG.features {
            assert!(CONFIG.operators.contains_key(&rule.kind), "{}", rule.kind);
        }
        for rule in &CONFIG.connections {
            assert!(!rule.when.is_empty());
            for kind in &rule.kinds {
                assert!(CONFIG.operators.contains_key(kind), "{kind}");
            }
            if matches!(rule.action, Action::Support) {
                assert!(CONFIG.operators.values().any(|entry| entry
                    .support
                    .as_ref()
                    .is_some_and(|support| support.group == rule.support_group)));
            }
        }
        for entry in CONFIG.operators.values() {
            assert!(!entry.title.is_empty() && !entry.concept.is_empty());
            if let Some(connector) = &entry.connector {
                assert!(CONFIG.operators.contains_key(connector));
            }
            for variant in &entry.variants {
                assert!(!variant.form.is_empty());
            }
        }
        assert!(CONFIG.presentation["inflections"].as_array().is_some());
        for (kind, metadata) in &CONFIG.rule_metadata {
            assert!(!metadata.stage.is_empty(), "{kind} 缺少阶段");
            assert!(!metadata.executor.is_empty(), "{kind} 缺少执行器");
            assert!(
                matches!(
                    metadata.executor.as_str(),
                    "verb_cycle"
                        | "i_adjective_conversion"
                        | "verb_cycle+i_adjective_conversion"
                        | "na_adjective_judgement"
                        | "nominalization"
                        | "auxiliary_verb"
                        | "classical_conjugation"
                        | "insertion"
                ),
                "{kind} 使用未知执行器 {}",
                metadata.executor
            );
        }
        assert_eq!(
            CONFIG
                .rule_metadata
                .get("nominalization")
                .map(|metadata| metadata.executor.as_str()),
            Some("nominalization")
        );
        let tree_labels = CONFIG
            .rule_tree
            .iter()
            .map(|node| node.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            tree_labels,
            vec![
                "动词循环类",
                "补助动词类",
                "插入语类",
                "ナ形容词判断类",
                "イ形容词转换类",
                "文言活用类",
            ]
        );
        for label in ["ナ形容词判断类", "イ形容词转换类", "文言活用类"] {
            let node = CONFIG
                .rule_tree
                .iter()
                .find(|node| node.label == label)
                .unwrap();
            assert!(
                (2..=3).contains(&node.children.len()),
                "{label} 的规则树分支应保持可读规模"
            );
            assert!(node.children.iter().all(|child| !child.label.is_empty()));
        }
        for kind in [
            "excessive",
            "inceptive",
            "continuative_aspect",
            "terminative",
            "iterative_aspect",
            "inchoative",
            "corrective",
            "incomplete",
            "completion",
            "reluctance",
            "failure",
            "missed_opportunity",
        ] {
            let metadata = CONFIG.rule_metadata.get(kind).unwrap();
            assert_eq!(
                metadata.executor, "auxiliary_verb",
                "{kind} 应由补助动词执行器处理"
            );
            assert_eq!(metadata.stage, "te_support", "{kind} 应位于补助层");
            assert_eq!(
                metadata.rule_group, "连用形补助",
                "{kind} 应属于连用形补助组"
            );
            assert_eq!(
                CONFIG.operators.get(kind).unwrap().concept,
                format!(
                    "grammar.auxiliary.{}",
                    match kind {
                        "excessive" => "sugiru",
                        "inceptive" => "hajimeru",
                        "continuative_aspect" => "tsuzukeru",
                        "terminative" => "owaru",
                        "iterative_aspect" => "makuru",
                        "inchoative" => "dasu",
                        "corrective" => "naosu",
                        "incomplete" => "kakeru",
                        "completion" => "kiru",
                        "reluctance" => "kaneru",
                        "failure" => "sokonau",
                        "missed_opportunity" => "sobireru",
                        _ => unreachable!(),
                    }
                )
            );
        }
        for kind in [
            "te_connection",
            "te_hoshii",
            "te_goran",
            "te_miseru",
            "te_tamaranai",
            "te_naranai",
            "te_yamanai",
        ] {
            assert_ne!(
                CONFIG.operators.get(kind).unwrap().concept,
                "morphology.chain",
                "{kind} 不应继续使用通用活用链概念"
            );
            assert_eq!(
                CONFIG.rule_metadata.get(kind).unwrap().executor,
                "auxiliary_verb",
                "{kind} 应由补助动词执行器处理"
            );
        }
    }

    #[test]
    fn classifies_only_declared_particles_as_insertion_entities() {
        let mut values = vec![None; crate::model::FIELD_NAMES.len()];
        let set = |values: &mut Vec<Option<String>>, name: &str, value: &str| {
            let index = crate::model::FIELD_NAMES
                .iter()
                .position(|item| *item == name)
                .unwrap();
            values[index] = Some(value.into());
        };
        set(&mut values, "pos1", "助詞");
        set(&mut values, "pos2", "接続助詞");
        set(&mut values, "lemma", "て");
        set(&mut values, "orthBase", "て");
        let mut token = ProviderToken {
            index: 0,
            surface: "て".into(),
            char_range: [0, 1],
            byte_range: [0, 3],
            lexicon_type: "test".into(),
            left_id: 0,
            right_id: 0,
            word_cost: 0,
            total_cost: 0,
            raw_feature: String::new(),
            fields: values
                .into_iter()
                .enumerate()
                .map(|(index, value)| crate::model::FeatureField {
                    index,
                    name: crate::model::FIELD_NAMES[index].into(),
                    label: crate::model::FIELD_LABELS[index].into(),
                    raw: value.clone(),
                    value,
                })
                .collect(),
        };
        assert_eq!(entity_class_for_token(&token), "");

        for (lemma, sub) in [
            ("は", "係助詞"),
            ("も", "係助詞"),
            ("さえ", "副助詞"),
            ("こそ", "係助詞"),
            ("だけ", "副助詞"),
            ("まで", "副助詞"),
            ("しか", "副助詞"),
            ("すら", "副助詞"),
            ("なんて", "副助詞"),
            ("など", "副助詞"),
            ("とか", "副助詞"),
        ] {
            token.fields[1].value = Some(sub.into());
            token.fields[7].value = Some(lemma.into());
            token.fields[10].value = Some(lemma.into());
            token.surface = lemma.into();
            assert!(is_insertion(&token), "{lemma}");
            assert_eq!(entity_class_for_token(&token), "插入语类", "{lemma}");
        }

        token.fields[1].value = Some("格助詞".into());
        token.fields[7].value = Some("に".into());
        token.fields[10].value = Some("に".into());
        token.surface = "に".into();
        assert!(!is_insertion(&token));
        assert_eq!(entity_class_for_token(&token), "");

        token.fields[7].value = Some("は".into());
        token.fields[10].value = Some("は".into());
        token.surface = "は".into();
        assert!(!is_insertion(&token));
        assert_eq!(entity_class_for_token(&token), "");
    }
}
