//! L2 形态链层。
//!
//! UniDic 词法连接与 GiNZA 正式词界、活用、依存共同决定核心和功能所有权。
use crate::linguistic_context::SourceEvidence;
use crate::model::{MorphemeToken, QueryForm};
use serde::{Deserialize, Serialize};

pub const SCHEMA: &str = "readux.morphology-artifact.v3";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionForm {
    #[default]
    Stem,
    Irrealis,
    Continuative,
    Terminal,
    Attributive,
    Conditional,
    Imperative,
    Volitional,
    Te,
    Other,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MorphologyState {
    pub category: String,
    pub form: ConnectionForm,
    pub conjugation_type: String,
    pub conjugation_form: String,
    /// 六类实体身份；阶段信息仍由规则层管理。
    #[serde(default)]
    pub entity_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MorphologyOccurrence {
    pub id: String,
    pub chain_id: String,
    pub operator_ids: Vec<String>,
    pub kind: String,
    pub char_range: [usize; 2],
    pub context_range: [usize; 2],
    pub hit_ranges: Vec<[usize; 2]>,
    pub morpheme_indices: Vec<usize>,
    pub candidates: Vec<String>,
    pub status: String,
    pub source_evidence: Vec<SourceEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MorphologyRole {
    Lexical,
    Functional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MorphologyOperator {
    pub operator_id: String,
    pub kind: String,
    pub source_morpheme_range: [usize; 2],
    pub char_range: [usize; 2],
    pub output_state: String,
    #[serde(default)]
    pub input_state: String,
    pub concept_id: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub candidates: Vec<String>,
    pub label: String,
    pub description: String,
    #[serde(default)]
    pub state_before: MorphologyState,
    #[serde(default)]
    pub state_after: MorphologyState,
    #[serde(default)]
    pub normalized_form: Option<String>,
    /// 规则阶段；阶段属于规则，不属于六类实体。
    #[serde(default)]
    pub stage: String,
    /// 类别专用执行器名称。
    #[serde(default)]
    pub executor: String,
    #[serde(default)]
    pub rule_group: String,
    #[serde(default)]
    pub input_category: String,
    #[serde(default)]
    pub output_category: String,
    #[serde(default)]
    pub terminal_mode: String,
    #[serde(default)]
    pub order_rank: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MorphologyChain {
    pub chain_id: String,
    pub anchor_morpheme: usize,
    pub anchor_range: [usize; 2],
    pub morpheme_range: [usize; 2],
    pub char_range: [usize; 2],
    pub role: MorphologyRole,
    pub base_lexeme: String,
    pub surface_form: String,
    pub dictionary_form: String,
    pub lemma_form: String,
    pub lookup_form: String,
    #[serde(default)]
    pub display_form: String,
    #[serde(default)]
    pub parent_chain_id: Option<String>,
    pub source_ranges: Vec<[usize; 2]>,
    pub operators: Vec<MorphologyOperator>,
    pub connection_forms: Vec<String>,
    pub evidence: Vec<String>,
    #[serde(default)]
    pub morpheme_indices: Vec<usize>,
    #[serde(default)]
    pub final_state: MorphologyState,
    #[serde(default)]
    pub core_morpheme_indices: Vec<usize>,
    #[serde(default)]
    pub query_forms: Vec<QueryForm>,
    #[serde(default)]
    pub source_evidence: Vec<SourceEvidence>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub terminal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MorphologyArtifact {
    pub schema: String,
    pub chains: Vec<MorphologyChain>,
    #[serde(default)]
    pub occurrences: Vec<MorphologyOccurrence>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

fn value(token: &crate::model::ProviderToken, index: usize) -> Option<String> {
    token
        .fields
        .get(index)
        .and_then(|field| field.value.clone())
        .filter(|item| !item.trim().is_empty())
}

fn first(value: Option<String>, fallback: &str) -> String {
    value.unwrap_or_else(|| fallback.to_owned())
}

/// UniDic 将五段动词的可能形式记作下一段基本表记，查询原型采用其词元。
pub(crate) fn potential_lemma(token: &crate::model::ProviderToken) -> Option<QueryForm> {
    if value(token, 0).as_deref() != Some("動詞") || !value(token, 4)?.starts_with("下一段") {
        return None;
    }
    let lemma = value(token, 7)?;
    let base = value(token, 10)?;
    let derives = |original: &str, potential: &str| {
        let Some(last) = original.chars().last() else {
            return false;
        };
        let ending = match last {
            'う' => "える",
            'く' => "ける",
            'ぐ' => "げる",
            'す' => "せる",
            'つ' => "てる",
            'ぬ' => "ねる",
            'ぶ' => "べる",
            'む' => "める",
            'る' => "れる",
            'ウ' => "エル",
            'ク' => "ケル",
            'グ' => "ゲル",
            'ス' => "セル",
            'ツ' => "テル",
            'ヌ' => "ネル",
            'ブ' => "ベル",
            'ム' => "メル",
            'ル' => "レル",
            _ => return false,
        };
        potential == format!("{}{ending}", &original[..original.len() - last.len_utf8()])
    };
    let reading = value(token, 6);
    let reading_matches = reading
        .as_deref()
        .zip(value(token, 21).as_deref())
        .is_some_and(|(original, potential)| derives(original, potential));
    if !derives(&lemma, &base) && !reading_matches {
        return None;
    }
    // 词元读法确认派生关系后，原型沿用基本表记中的汉字和假名，如帰れる→帰る。
    let form = if derives(&lemma, &base) {
        lemma
    } else {
        let mut characters = base.chars().collect::<Vec<_>>();
        let ending = characters.pop()?;
        characters.pop()?;
        let mut original = reading.as_deref()?.chars().last()?;
        if ending == 'る' && ('ァ'..='ヶ').contains(&original) {
            original = char::from_u32(original as u32 - 0x60)?;
        }
        characters.push(original);
        characters.into_iter().collect()
    };
    Some(QueryForm {
        kind: "potential_lemma".into(),
        form,
        reading_field: reading.as_ref().map(|_| "lForm".into()),
        reading,
    })
}

/// 先保存原子活用，再按连接形和字符连续性建立所有权。
pub fn collect(
    source_tokens: &[crate::model::ProviderToken],
    morphemes: &[MorphemeToken],
) -> Result<MorphologyArtifact, String> {
    collect_with_external(source_tokens, morphemes, &[])
}

/// 先保留原子词形，再由状态机共同消费词法连接和 GiNZA 结构。
pub fn collect_with_external(
    source_tokens: &[crate::model::ProviderToken],
    morphemes: &[MorphemeToken],
    external: &[crate::external::SourceArtifact],
) -> Result<MorphologyArtifact, String> {
    if source_tokens.len() != morphemes.len() {
        return Err("形态链输入的来源 token 与统一 token 数量不一致".into());
    }
    if source_tokens
        .iter()
        .zip(morphemes)
        .any(|(a, b)| a.char_range != b.char_range || a.surface != b.surface)
    {
        return Err("活用输入的词元范围或正文不一致".into());
    }
    let mut chains = Vec::new();
    for (index, (source, morpheme)) in source_tokens.iter().zip(morphemes).enumerate() {
        let pos_major = value(source, 0).unwrap_or_default();
        if matches!(pos_major.as_str(), "記号" | "補助記号" | "空白") {
            continue;
        }
        let lemma = first(value(source, 7), &morpheme.surface);
        let dictionary_form = first(value(source, 10), &lemma);
        let lemma_form = lemma.clone();
        let potential = potential_lemma(source);
        let lookup_form = potential
            .as_ref()
            .map_or_else(|| dictionary_form.clone(), |query| query.form.clone());
        let role = if matches!(pos_major.as_str(), "助詞" | "助動詞") {
            MorphologyRole::Functional
        } else {
            MorphologyRole::Lexical
        };
        let mut operators = Vec::new();
        for (kind, type_index, form_index, label) in [
            ("conjugation", 4usize, 5usize, "活用"),
            ("initial_alternation", 13usize, 14usize, "词首变化"),
            ("final_alternation", 15usize, 16usize, "词尾变化"),
        ] {
            let type_value = value(source, type_index);
            let form_value = value(source, form_index);
            if type_value.is_none() && form_value.is_none() {
                continue;
            }
            let output_state = form_value
                .clone()
                .or(type_value.clone())
                .unwrap_or_default();
            let detail = [type_value.clone(), form_value.clone()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("/");
            operators.push(MorphologyOperator {
                operator_id: format!("morphology:{index}:{kind}"),
                kind: kind.into(),
                source_morpheme_range: [index, index + 1],
                char_range: morpheme.char_range,
                output_state,
                input_state: lookup_form.clone(),
                concept_id: "morphology.chain".into(),
                confidence: 1.0,
                evidence: if kind == "conjugation" && potential.is_some() {
                    vec![
                        format!("unidic:{label}:{detail}"),
                        "unidic:potential".into(),
                    ]
                } else {
                    vec![format!("unidic:{label}:{detail}")]
                },
                candidates: Vec::new(),
                label: detail.clone(),
                description: format!("UniDic {label}字段：{detail}"),
                state_before: MorphologyState {
                    category: pos_major.clone(),
                    form: if value(source, 4).is_some() {
                        ConnectionForm::Terminal
                    } else {
                        ConnectionForm::Stem
                    },
                    conjugation_type: value(source, 4).unwrap_or_default(),
                    conjugation_form: "基本形".into(),
                    entity_class: crate::morphology_config::entity_class_for_token(source).into(),
                },
                state_after: crate::morphology_machine::state(source),
                normalized_form: None,
                stage: String::new(),
                executor: String::new(),
                rule_group: String::new(),
                input_category: pos_major.clone(),
                output_category: pos_major.clone(),
                terminal_mode: String::new(),
                order_rank: 0,
            });
        }
        let connection_forms = [value(source, 17), value(source, 18)]
            .into_iter()
            .flatten()
            .collect();
        let mut evidence = vec!["provider:unidic".to_owned(), format!("pos1:{pos_major}")];
        if let Some(form) = value(source, 5) {
            evidence.push(format!("cForm:{form}"));
        }
        let mut query_forms = morpheme.query_forms.clone();
        if let Some(query) = potential {
            query_forms.retain(|form| form.form != query.form);
            query_forms.insert(0, query);
        }
        chains.push(MorphologyChain {
            chain_id: format!("morphology:m{index}"),
            anchor_morpheme: index,
            anchor_range: morpheme.char_range,
            morpheme_range: [index, index + 1],
            char_range: morpheme.char_range,
            role,
            base_lexeme: lemma.clone(),
            surface_form: morpheme.surface.clone(),
            display_form: lookup_form.clone(),
            parent_chain_id: None,
            dictionary_form,
            lemma_form,
            lookup_form,
            source_ranges: vec![morpheme.char_range],
            operators,
            connection_forms,
            evidence,
            morpheme_indices: vec![index],
            final_state: crate::morphology_machine::state(source),
            core_morpheme_indices: vec![index],
            query_forms,
            source_evidence: Vec::new(),
            status: "resolved".into(),
            terminal: false,
        });
    }
    Ok(crate::morphology_machine::compose(
        chains,
        source_tokens,
        morphemes,
        external,
    ))
}

pub fn features(token: &crate::model::ProviderToken) -> Vec<String> {
    crate::morphology_config::features(token)
}

pub fn feature_concept(feature: &str) -> &str {
    crate::morphology_config::definition(feature)
        .map_or("morphology.chain", |entry| entry.concept.as_str())
}

pub(crate) fn feature_label(feature: &str) -> &str {
    crate::morphology_config::definition(feature).map_or("活用", |entry| entry.label.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FeatureField, MorphemeToken, ProviderToken};

    fn source(fields: &[(&str, &str)]) -> ProviderToken {
        let mut values = vec![None; 29];
        for (name, value) in fields {
            if let Some(index) = crate::model::FIELD_NAMES
                .iter()
                .position(|item| item == name)
            {
                values[index] = Some((*value).into());
            }
        }
        ProviderToken {
            index: 0,
            surface: "読ん".into(),
            char_range: [0, 2],
            byte_range: [0, 6],
            lexicon_type: "known".into(),
            left_id: 0,
            right_id: 0,
            word_cost: 0,
            total_cost: 0,
            raw_feature: String::new(),
            fields: values
                .into_iter()
                .enumerate()
                .map(|(index, value)| FeatureField {
                    index,
                    name: index.to_string(),
                    label: String::new(),
                    raw: value.clone(),
                    value,
                })
                .collect(),
        }
    }

    #[test]
    fn potential_forms_query_the_lexical_lemma() {
        for (lemma, base) in [
            ("立ち回る", "立ち回れる"),
            ("読む", "読める"),
            ("書く", "書ける"),
            ("泳ぐ", "泳げる"),
            ("話す", "話せる"),
        ] {
            let token = source(&[
                ("pos1", "動詞"),
                ("cType", "下一段-ラ行"),
                ("cForm", "未然形-一般"),
                ("lemma", lemma),
                ("orthBase", base),
            ]);
            let query = potential_lemma(&token).unwrap();
            assert_eq!(query.form, lemma);
            let morpheme = MorphemeToken {
                id: "m0".into(),
                source_index: 0,
                surface: token.surface.clone(),
                char_range: token.char_range,
                pos: [Some("動詞".into()), None, None, None],
                lemma: Some(lemma.into()),
                reading: None,
                query_forms: Vec::new(),
            };
            let artifact = collect(&[token], &[morpheme]).unwrap();
            let chain = &artifact.chains[0];
            assert_eq!(chain.lookup_form, lemma);
            assert_eq!(chain.dictionary_form, base);
            assert_eq!(chain.operators[0].input_state, lemma);
            assert_eq!(chain.query_forms[0].form, lemma);
        }
        for lemma in ["食べる", "見える", "教える"] {
            assert!(potential_lemma(&source(&[
                ("pos1", "動詞"),
                ("cType", "下一段-ア行"),
                ("lemma", lemma),
                ("orthBase", lemma)
            ]))
            .is_none());
        }
        for (lemma, base, reading, base_reading, expected) in [
            ("返る", "帰れる", "カエル", "カエレル", "帰る"),
            ("読む", "よめる", "ヨム", "ヨメル", "よむ"),
        ] {
            let token = source(&[
                ("pos1", "動詞"),
                ("cType", "下一段-ラ行"),
                ("lemma", lemma),
                ("orthBase", base),
                ("lForm", reading),
                ("kanaBase", base_reading),
            ]);
            assert_eq!(potential_lemma(&token).unwrap().form, expected);
        }
    }

    #[test]
    fn preserves_unidic_inflection_and_functional_role() {
        let token = source(&[
            ("pos1", "動詞"),
            ("pos2", "自立"),
            ("pos3", ""),
            ("pos4", ""),
            ("cType", "五段-ラ行"),
            ("cForm", "連用形-撥音便"),
            ("lForm", "ヨム"),
            ("lemma", "読む"),
            ("orth", "読ん"),
            ("orthBase", "読む"),
            ("iConType", "動詞接続"),
        ]);
        let morpheme = MorphemeToken {
            id: "m0".into(),
            source_index: 0,
            surface: "読ん".into(),
            char_range: [0, 2],
            pos: [Some("動詞".into()), None, None, None],
            lemma: Some("読む".into()),
            reading: None,
            query_forms: Vec::new(),
        };
        let artifact = collect(&[token], &[morpheme]).unwrap();
        assert_eq!(artifact.chains[0].dictionary_form, "読む");
        assert_eq!(artifact.chains[0].operators[0].kind, "conjugation");
        assert!(matches!(artifact.chains[0].role, MorphologyRole::Lexical));
    }
}
