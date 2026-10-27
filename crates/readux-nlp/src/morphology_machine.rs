//! 词法连接状态机。GiNZA 参与复合核心、功能所有权和来源冲突判定。
use crate::{
    external::{SourceArtifact, SourceNode},
    linguistic_context::Context,
    model::{MorphemeToken, ProviderToken, QueryForm},
    morphology::*,
};

use crate::morphology_config::{self as config, field, identity, Action, CONFIG};

pub(crate) fn state(t: &ProviderToken) -> MorphologyState {
    let raw = field(t, 5);
    let classical = field(t, 4).starts_with("文語")
        || config::classical_paradigm_form(t).is_some()
        || config::classical_auxiliary(t).is_some();
    let form = if classical {
        config::classical_connection_form(raw)
    } else {
        None
    }
    .unwrap_or_else(|| {
        CONFIG
            .forms
            .iter()
            .find(|rule| rule.prefixes.iter().any(|prefix| raw.starts_with(prefix)))
            .map(|rule| rule.form.clone())
            .unwrap_or_else(|| {
                if raw.is_empty() {
                    ConnectionForm::Stem
                } else {
                    ConnectionForm::Other
                }
            })
    });
    let category = if classical {
        "文語用言"
    } else if field(t, 0) == "接尾辞" && field(t, 1) == "名詞的" {
        "名詞"
    } else {
        field(t, 0)
    };
    MorphologyState {
        category: category.into(),
        form,
        conjugation_type: field(t, 4).into(),
        conjugation_form: raw.into(),
        entity_class: if classical {
            "文言活用类"
        } else {
            config::entity_class_for_token(t)
        }
        .into(),
    }
}

fn semantic(t: &ProviderToken) -> Vec<String> {
    config::features(t)
}

/// 连接条件读取紧邻的语法步骤；插入语保持原接续许可。
fn follows_operator(previous: &MorphologyChain, kinds: &[String]) -> bool {
    previous
        .operators
        .iter()
        .rev()
        .find(|op| op.executor != "insertion" || kinds.contains(&op.kind))
        .is_some_and(|op| kinds.contains(&op.kind))
}

fn terminal_mode(kind: &str) -> &'static str {
    config::rule_metadata(kind).map_or("continue", |metadata| metadata.terminal_mode.as_str())
}

fn may_stop(kind: &str, state: &MorphologyState) -> bool {
    if config::rule_metadata(kind).is_some_and(|metadata| {
        matches!(
            metadata.executor.as_str(),
            "classical_conjugation" | "na_adjective_judgement"
        )
    }) {
        return matches!(
            state.form,
            ConnectionForm::Terminal
                | ConnectionForm::Attributive
                | ConnectionForm::Conditional
                | ConnectionForm::Imperative
                | ConnectionForm::Continuative
        ) || kind == "negative_zuni" && state.form == ConnectionForm::Other;
    }
    matches!(
        terminal_mode(kind),
        "sentence_or_pause" | "pause_or_continue"
    )
}

fn accepts_input(state: &MorphologyState, metadata: &config::RuleMetadata) -> bool {
    let category = &state.category;
    let accepted_category = if metadata.input_categories.is_empty() {
        match metadata.input_category.as_str() {
            "動詞" | "动词" => category == "動詞",
            "形容詞" => category == "形容詞",
            "形状詞" => category == "形状詞",
            "名詞" => category == "名詞",
            "動詞／助動詞" => matches!(category.as_str(), "動詞" | "助動詞"),
            "用言" => matches!(
                category.as_str(),
                "動詞" | "形容詞" | "形状詞" | "助動詞" | "文語用言"
            ),
            "已成立链" | "" => true,
            _ => false,
        }
    } else {
        metadata.input_categories.contains(category)
    };
    accepted_category
        && (metadata.input_forms.is_empty() || metadata.input_forms.contains(&state.form))
        && !metadata
            .exclude_input_ctypes
            .iter()
            .any(|ctype| state.conjugation_type == *ctype)
}

enum Transition {
    Core,
    Attach(Vec<String>),
    Support(&'static str),
    Insert(&'static str),
    Boundary,
    Stop,
}

fn declared_output(previous: &MorphologyChain, atom: &mut MorphologyChain, kind: &str) {
    let Some(rule) = config::definition(kind) else {
        return;
    };
    if rule.preserve_state {
        atom.final_state = previous.final_state.clone();
    }
    if let Some(category) = &rule.output_category {
        atom.final_state.category = category.clone();
    }
    if let Some(form) = &rule.output_form {
        atom.final_state.form = form.clone();
    }
}

/// 动词循环类执行器：只接受动词范畴内的有限转移，词性转换交给其他执行器。
struct VerbCycleExecutor;

fn connection_form_name(form: &ConnectionForm) -> &'static str {
    match form {
        ConnectionForm::Stem => "stem",
        ConnectionForm::Irrealis => "irrealis",
        ConnectionForm::Continuative => "continuative",
        ConnectionForm::Terminal => "terminal",
        ConnectionForm::Attributive => "attributive",
        ConnectionForm::Conditional => "conditional",
        ConnectionForm::Imperative => "imperative",
        ConnectionForm::Volitional => "volitional",
        ConnectionForm::Te => "te",
        ConnectionForm::Other => "other",
    }
}

impl VerbCycleExecutor {
    fn accepts(kind: &str) -> bool {
        matches!(
            config::rule_metadata(kind).map(|metadata| metadata.executor.as_str()),
            Some("verb_cycle") | Some("verb_cycle+i_adjective_conversion")
        )
    }

    fn apply(
        previous: &MorphologyChain,
        atom: &mut MorphologyChain,
        kind: &str,
        diagnostics: &mut Vec<String>,
        index: usize,
    ) {
        if !Self::accepts(kind) {
            return;
        }
        declared_output(previous, atom, kind);
        let metadata = config::rule_metadata(kind).expect("动词循环规则缺少元数据");
        if !accepts_input(&previous.final_state, metadata) {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "verb_cycle_input_category:{}:{}:{}",
                previous.chain_id, kind, index
            ));
        }
        if metadata.output_category == "動詞" || metadata.output_category == "动词" {
            atom.final_state.category = "動詞".into();
            atom.final_state.entity_class = "动词循环类".into();
        } else if metadata.output_category == "用言" {
            atom.final_state.category = previous.final_state.category.clone();
            atom.final_state.entity_class = previous.final_state.entity_class.clone();
        }
    }
}

/// イ形容词转换执行器：完成动词或助动词到イ形容词性整体的切换。
struct IAdjectiveConversionExecutor;

impl IAdjectiveConversionExecutor {
    fn accepts(kind: &str) -> bool {
        matches!(
            config::rule_metadata(kind).map(|metadata| metadata.executor.as_str()),
            Some("i_adjective_conversion") | Some("verb_cycle+i_adjective_conversion")
        )
    }

    fn apply(
        previous: &MorphologyChain,
        atom: &mut MorphologyChain,
        kind: &str,
        diagnostics: &mut Vec<String>,
        index: usize,
    ) {
        if !Self::accepts(kind) {
            return;
        }
        declared_output(previous, atom, kind);
        let metadata = config::rule_metadata(kind).expect("イ形容词转换规则缺少元数据");
        if !accepts_input(&previous.final_state, metadata) {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "i_adjective_input_category:{}:{}:{}",
                previous.chain_id, kind, index
            ));
        }
        atom.final_state.category = "形容詞".into();
        atom.final_state.entity_class = "イ形容词转换类".into();
        if kind == "rashii_characteristic" {
            atom.evidence
                .push("judgement_usage:lexical_characteristic".into());
        }
    }

    fn continuation(previous: &MorphologyChain, next: &ProviderToken) -> Option<Vec<String>> {
        if previous.final_state.category != "形容詞"
            || previous.final_state.entity_class != "イ形容词转换类"
            || previous.final_state.form != ConnectionForm::Continuative
        {
            return None;
        }
        let rules: Vec<_> = semantic(next)
            .into_iter()
            .filter(|kind| matches!(kind.as_str(), "negative" | "past"))
            .collect();
        (!rules.is_empty()).then_some(rules)
    }
}

/// ナ形容词判断执行器：把判断词干交给だ／な／で等后续收尾规则。
struct NaAdjectiveJudgementExecutor;

impl NaAdjectiveJudgementExecutor {
    fn accepts(kind: &str) -> bool {
        matches!(
            config::rule_metadata(kind).map(|metadata| metadata.executor.as_str()),
            Some("na_adjective_judgement")
        )
    }

    fn apply(
        previous: &MorphologyChain,
        atom: &mut MorphologyChain,
        kind: &str,
        diagnostics: &mut Vec<String>,
        _index: usize,
    ) {
        if !Self::accepts(kind) {
            return;
        }
        declared_output(previous, atom, kind);
        let metadata = config::rule_metadata(kind).expect("ナ形容词判断规则缺少元数据");
        if !accepts_input(&previous.final_state, metadata) {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "na_judgement_input_state:{}:{}:{}:{}",
                previous.chain_id,
                kind,
                previous.final_state.category,
                connection_form_name(&previous.final_state.form)
            ));
        }
        let output_category = config::definition(kind)
            .and_then(|definition| definition.output_category.as_deref())
            .unwrap_or(metadata.output_category.as_str());
        atom.final_state.category = match output_category {
            "形状詞" => "形状詞".into(),
            "名詞" => "名詞".into(),
            "用言" | "保持前项" | "" => previous.final_state.category.clone(),
            category => category.into(),
        };
        atom.final_state.entity_class = if atom.final_state.category == "形容詞" {
            "イ形容词转换类".into()
        } else {
            "ナ形容词判断类".into()
        };
        atom.evidence.push(format!(
            "judgement_state:{}:{}->{}:{}",
            previous.final_state.category,
            connection_form_name(&previous.final_state.form),
            atom.final_state.category,
            connection_form_name(&atom.final_state.form)
        ));
        let usage = match kind {
            "sou_modality"
                if previous.final_state.form == ConnectionForm::Continuative
                    && matches!(previous.final_state.category.as_str(), "動詞" | "形容詞")
                    || kind == "sou_modality"
                        && previous.final_state.form == ConnectionForm::Stem =>
            {
                "appearance"
            }
            "sou_modality" => "hearsay",
            "you_modality" | "mitai_modality" => "simile",
            "obligation" => "normative",
            "hazu_modality" => "expectation",
            "rashii_modality" => "inference",
            "copula_de" => "copula_continuative",
            "copula_past" => "copula_past",
            "copula" | "copula_aru" => "copula",
            _ => "construction",
        };
        atom.evidence.push(format!("judgement_usage:{usage}"));
    }
}

struct NominalizationExecutor;

impl NominalizationExecutor {
    fn apply(
        previous: &MorphologyChain,
        atom: &mut MorphologyChain,
        kind: &str,
        diagnostics: &mut Vec<String>,
        index: usize,
    ) {
        if config::rule_metadata(kind).map(|metadata| metadata.executor.as_str())
            != Some("nominalization")
        {
            return;
        }
        declared_output(previous, atom, kind);
        let metadata = config::rule_metadata(kind).unwrap();
        if !accepts_input(&previous.final_state, metadata) {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "nominalization_input_category:{}:{}:{}",
                previous.chain_id, kind, index
            ));
        }
        atom.final_state.category = "名詞".into();
        atom.final_state.entity_class.clear();
    }
}

/// 补助动词执行器：验证连接器、建立父子链，并允许补助层继续递归。
struct AuxiliaryVerbExecutor;

impl AuxiliaryVerbExecutor {
    fn accepts(kind: &str) -> bool {
        config::rule_metadata(kind).is_some_and(|metadata| metadata.executor == "auxiliary_verb")
    }

    fn apply(
        previous: &MorphologyChain,
        atom: &mut MorphologyChain,
        kind: &str,
        source: &[ProviderToken],
        index: usize,
        depth: usize,
        diagnostics: &mut Vec<String>,
    ) {
        if !Self::accepts(kind) {
            return;
        }
        declared_output(previous, atom, kind);
        let metadata = config::rule_metadata(kind).expect("补助动词规则缺少元数据");
        if !accepts_input(&previous.final_state, metadata) {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "auxiliary_input_category:{}:{}:{}",
                previous.chain_id, kind, index
            ));
        }
        let support_group = config::definition(kind)
            .and_then(|definition| definition.support.as_ref())
            .map(|support| support.group.as_str());
        let connector = previous
            .operators
            .iter()
            .rev()
            .find(|operator| operator.executor != "insertion");
        let has_te_connector = previous.final_state.form == ConnectionForm::Te;
        let has_connector = match support_group {
            Some("te") => has_te_connector,
            Some("contracted") => {
                has_te_connector || previous.final_state.form == ConnectionForm::Continuative
            }
            Some("continuative") => previous.final_state.form == ConnectionForm::Continuative,
            Some("enumerative") => connector.is_some_and(|op| op.kind == "enumerative"),
            Some("volitional") => connector.is_some_and(|op| op.kind == "volitional_to_connection"),
            Some("nominal_change") => connector
                .is_some_and(|op| matches!(op.kind.as_str(), "nominal_change_ni" | "adverbial_ni")),
            Some("adjectival_change") => previous.final_state.form == ConnectionForm::Continuative,
            _ => false,
        };
        if !has_connector {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "auxiliary_missing_connector:{}:{}",
                previous.chain_id, index
            ));
        }
        let contiguous_or_inserted = previous.morpheme_range[1] <= index
            && source
                .get(previous.morpheme_range[1]..index)
                .is_some_and(|skipped| {
                    skipped.iter().all(insertion_token)
                        && std::iter::once(previous.char_range[1])
                            .chain(skipped.iter().map(|token| token.char_range[1]))
                            .zip(
                                skipped
                                    .iter()
                                    .chain(std::iter::once(&source[index]))
                                    .map(|token| token.char_range[0]),
                            )
                            .all(|(end, start)| end == start)
                });
        if !contiguous_or_inserted {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "auxiliary_non_contiguous:{}:{}",
                previous.chain_id, index
            ));
        }
        atom.role = MorphologyRole::Functional;
        atom.parent_chain_id = Some(previous.chain_id.clone());
        // 后项保留自身活用型和活用形；缩约助动词按展开后的动词身份继续，
        // ほしい与整体否定构式则进入形容词活用。
        atom.final_state.category = match metadata.output_category.as_str() {
            "动词" => "動詞",
            category => category,
        }
        .into();
        atom.final_state.entity_class = if atom.final_state.category == "形容詞" {
            "イ形容词转换类"
        } else {
            "补助动词类"
        }
        .into();
        atom.evidence
            .push(format!("connector:{}", metadata.rule_group));
        if let Some(connector) =
            config::definition(kind).and_then(|definition| definition.connector.as_deref())
        {
            atom.evidence.push(format!("connector_rule:{connector}"));
        }
        if depth >= CONFIG.max_auxiliary_depth {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "auxiliary_depth_limit:{}:{}",
                previous.chain_id, index
            ));
        }
    }
}

fn auxiliary_support_depth(chains: &[MorphologyChain], previous_index: usize) -> usize {
    let mut depth = 0;
    let mut current = Some(previous_index);
    while let Some(index) = current {
        let chain = &chains[index];
        depth += chain
            .operators
            .iter()
            .filter(|operator| {
                config::definition(&operator.kind)
                    .is_some_and(|definition| definition.support.is_some())
            })
            .count();
        current = chain.parent_chain_id.as_ref().and_then(|parent_id| {
            chains
                .iter()
                .position(|candidate| &candidate.chain_id == parent_id)
        });
    }
    depth
}

/// 文言活用执行器：使用独立的文言活用表，不与现代判断结构共用状态。
struct ClassicalConjugationExecutor;

impl ClassicalConjugationExecutor {
    fn accepts(kind: &str) -> bool {
        config::rule_metadata(kind)
            .is_some_and(|metadata| metadata.executor == "classical_conjugation")
    }

    fn apply(
        previous: &MorphologyChain,
        atom: &mut MorphologyChain,
        kind: &str,
        token: &ProviderToken,
        diagnostics: &mut Vec<String>,
        index: usize,
    ) {
        if !Self::accepts(kind) {
            return;
        }
        if kind == "classical_gotokunari"
            && previous
                .operators
                .iter()
                .any(|operator| operator.kind == "classical_gotoshi")
            && previous.final_state.form == ConnectionForm::Continuative
            && field(token, 4).starts_with("文語助動詞-ナリ")
        {
            atom.final_state.category = "文語用言".into();
            atom.final_state.entity_class = "文言活用类".into();
            atom.final_state.form =
                config::classical_connection_form(&atom.final_state.conjugation_form)
                    .unwrap_or_else(|| atom.final_state.form.clone());
            atom.evidence.extend([
                "classical_composition:ごとし+なり".into(),
                "classical_lemma:ごとくなり".into(),
            ]);
            return;
        }
        if kind == "classical_adjective" {
            let Some((entry, form_name)) = config::classical_paradigm_match(
                &[atom.base_lexeme.as_str(), atom.dictionary_form.as_str()],
                &atom.surface_form,
                &atom.final_state.conjugation_type,
                &atom.final_state.conjugation_form,
            ) else {
                atom.status = "candidate".into();
                diagnostics.push(format!(
                    "classical_paradigm_unmapped:{}:{}",
                    atom.chain_id, index
                ));
                return;
            };
            atom.final_state.category = "文語用言".into();
            atom.final_state.entity_class = "文言活用类".into();
            atom.evidence.extend([
                format!("classical_paradigm:{}:{}", entry.class_name, form_name),
                format!("classical_lemma:{}", entry.lemma),
                format!("classical_connection:{}", entry.connection.join("／")),
            ]);
            return;
        }
        // UniDic 在「泣かざりし」等真实文言输入中偶尔将「し」标为接续助词，
        // 词元失去助动词 cType。前项已由「ざり」确认文言否定时，保留受限的
        // き连体形接续，避免把普通接续助词泛化为文言过去。
        if kind == "classical_ki"
            && token.surface == "し"
            && field(token, 0) == "助詞"
            && field(token, 1) == "接続助詞"
            && previous
                .operators
                .iter()
                .any(|operator| operator.kind == "negative_zuni")
        {
            atom.final_state.category = "文語用言".into();
            atom.final_state.entity_class = "文言活用类".into();
            atom.final_state.form = ConnectionForm::Attributive;
            atom.evidence.extend([
                "classical_paradigm:特殊型:連体形".into(),
                "classical_lemma:き".into(),
                "classical_connection:連用形".into(),
                "classical_unidic_pos_fallback".into(),
            ]);
            return;
        }
        config::rule_metadata(kind).expect("文言规则缺少元数据");
        let Some((entry, form_name)) = CONFIG
            .classical_auxiliaries
            .iter()
            .filter(|entry| entry.rule == kind)
            .find_map(|entry| {
                entry
                    .form(
                        identity(token),
                        &token.surface,
                        field(token, 4),
                        field(token, 5),
                    )
                    .map(|form| (entry, form))
            })
        else {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "classical_form_unmapped:{}:{}:{}",
                previous.chain_id, kind, index
            ));
            return;
        };
        if !config::classical_accepts(entry, &previous.final_state) {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "classical_input_state:{}:{}:{}",
                previous.chain_id, kind, index
            ));
        }
        atom.final_state.entity_class = "文言活用类".into();
        atom.final_state.category = "文語用言".into();
        atom.final_state.form =
            config::classical_connection_form(&atom.final_state.conjugation_form)
                .unwrap_or_else(|| atom.final_state.form.clone());
        if kind == "negative_zuni" && atom.morpheme_range[1] > atom.morpheme_range[0] + 1 {
            atom.final_state.form = ConnectionForm::Other;
        }
        atom.evidence.extend([
            format!("classical_paradigm:{}:{}", entry.class_name, form_name),
            format!("classical_connection:{}", entry.connection.join("／")),
            format!(
                "classical_input:{}:{}",
                previous.final_state.category,
                connection_form_name(&previous.final_state.form)
            ),
            format!("classical_lemma:{}", entry.lemma),
        ]);
    }
}

/// 按规则元数据选择唯一的状态转换执行器。
fn apply_rule_executor(
    previous: &MorphologyChain,
    atom: &mut MorphologyChain,
    kind: &str,
    token: &ProviderToken,
    diagnostics: &mut Vec<String>,
    index: usize,
) {
    let Some(metadata) = config::rule_metadata(kind) else {
        return;
    };
    match metadata.executor.as_str() {
        "verb_cycle" => VerbCycleExecutor::apply(previous, atom, kind, diagnostics, index),
        "i_adjective_conversion" => {
            IAdjectiveConversionExecutor::apply(previous, atom, kind, diagnostics, index)
        }
        "verb_cycle+i_adjective_conversion"
            if metadata.output_category == "形容詞"
                || previous.final_state.category == "形容詞" =>
        {
            IAdjectiveConversionExecutor::apply(previous, atom, kind, diagnostics, index)
        }
        "verb_cycle+i_adjective_conversion" => {
            VerbCycleExecutor::apply(previous, atom, kind, diagnostics, index)
        }
        "na_adjective_judgement" => {
            NaAdjectiveJudgementExecutor::apply(previous, atom, kind, diagnostics, index)
        }
        "nominalization" => NominalizationExecutor::apply(previous, atom, kind, diagnostics, index),
        "auxiliary_verb" if kind == "te_connection" => declared_output(previous, atom, kind),
        "classical_conjugation" => {
            ClassicalConjugationExecutor::apply(previous, atom, kind, token, diagnostics, index)
        }
        "insertion" => InsertionExecutor::apply(previous, atom, kind, token, diagnostics, index),
        executor => {
            atom.status = "candidate".into();
            diagnostics.push(format!("unknown_rule_executor:{kind}:{executor}:{index}"));
        }
    }
}

fn insertion_token(token: &ProviderToken) -> bool {
    config::is_insertion(token)
}

/// 按已声明的连接器查找插入范围，补助词始终连接原用言链。
fn bridge_support(
    previous: &MorphologyChain,
    index: usize,
    source: &[ProviderToken],
    context: &Context<'_>,
) -> bool {
    CONFIG.insertion_positions.iter().any(|position| {
        !position.connector.is_empty()
            && bridge_support_for_position(previous, index, position, source, context)
    })
}

fn bridge_support_for_position(
    previous: &MorphologyChain,
    index: usize,
    position: &config::InsertionPosition,
    source: &[ProviderToken],
    context: &Context<'_>,
) -> bool {
    if previous.morpheme_range[1] > index
        || config::support_kind(&position.support_group, &source[index]).is_none()
    {
        return false;
    }
    let Some(connector) = previous.operators.iter().rev().find(|operator| {
        operator.kind == position.connector && operator.state_after == previous.final_state
    }) else {
        return false;
    };
    let start = connector.source_morpheme_range[1];
    let inserted = &source[start..index];
    if inserted.is_empty()
        || !source[start - 1..=index]
            .windows(2)
            .all(|pair| pair[0].char_range[1] == pair[1].char_range[0])
    {
        return false;
    }
    let mut member = start;
    while member < index {
        if insertion_token(&source[member]) {
            member += 1;
        } else if let Some(operator) = previous.operators.iter().find(|operator| {
            operator.executor == "insertion"
                && operator.source_morpheme_range[0] == member
                && operator.source_morpheme_range[1] > member
                && operator.source_morpheme_range[1] <= index
        }) {
            member = operator.source_morpheme_range[1];
        } else {
            return false;
        }
    }
    context.confirms_inserted_support(
        source[previous.core_morpheme_indices[0]].char_range,
        connector.char_range,
        &inserted
            .iter()
            .map(|token| token.char_range)
            .collect::<Vec<_>>(),
        source[index].char_range,
    ) || context.ginza.is_none()
}

/// 插入语执行器：按位置声明保留词性、活用状态、阶段和停机许可。
struct InsertionExecutor;

impl InsertionExecutor {
    fn position(
        previous: &MorphologyChain,
        next: &ProviderToken,
        context: &Context<'_>,
        source: &[ProviderToken],
    ) -> Option<&'static config::InsertionPosition> {
        if previous.morpheme_range[1] != next.index
            || previous.char_range[1] != next.char_range[0]
            || !insertion_token(next)
        {
            return None;
        }
        CONFIG.insertion_positions.iter().find(|position| {
            (position.categories.is_empty()
                || position.categories.contains(&previous.final_state.category))
                && (position.forms.is_empty()
                    || position.forms.contains(&previous.final_state.form))
                && (position.bases.is_empty()
                    || position.bases.iter().any(|base| base == identity(next)))
                && if position.connector.is_empty() {
                    true
                } else {
                    let mut end = next.index + 1;
                    while source.get(end).is_some_and(insertion_token) {
                        end += 1;
                    }
                    source.get(end).is_some()
                        && bridge_support_for_position(previous, end, position, source, context)
                }
        })
    }

    fn apply(
        previous: &MorphologyChain,
        atom: &mut MorphologyChain,
        kind: &str,
        token: &ProviderToken,
        diagnostics: &mut Vec<String>,
        index: usize,
    ) {
        let metadata = config::rule_metadata(kind).expect("插入语规则缺少元数据");
        let declared = config::definition(kind).expect("插入语规则缺少定义");
        assert_eq!(metadata.executor, "insertion");
        assert!(declared.preserve_state);
        if !accepts_input(&previous.final_state, metadata)
            || kind == "insertion" && !insertion_token(token)
            || kind == "exemplification"
                && !atom
                    .evidence
                    .iter()
                    .any(|evidence| evidence == "phenomenon:insertion")
        {
            atom.status = "candidate".into();
            diagnostics.push(format!(
                "insertion_input_state:{}:{kind}:{index}",
                previous.chain_id
            ));
        }
        atom.final_state = previous.final_state.clone();
        atom.terminal = previous.terminal;
        atom.evidence.extend([
            "phenomenon:insertion".into(),
            format!("insertion_owner:{}", previous.chain_id),
            format!(
                "insertion_state_preserved:{}:{}",
                previous.final_state.category,
                connection_form_name(&previous.final_state.form)
            ),
        ]);
    }
}

fn transition(
    previous: &MorphologyChain,
    next: &ProviderToken,
    context: &Context<'_>,
    source: &[ProviderToken],
) -> Transition {
    if let Some((entry, _)) = config::classical_auxiliary(next) {
        if entry.rule == "classical_nari_copula"
            && previous.final_state.form == ConnectionForm::Continuative
            && previous
                .operators
                .iter()
                .any(|operator| operator.kind == "classical_gotoshi")
        {
            return Transition::Attach(vec!["classical_gotokunari".into()]);
        }
        // べき后接现代判断词尾时，作为现代规范判断词干参与同一条链。
        if next.surface == "べき"
            && source.get(next.index + 1).is_some_and(|following| {
                field(following, 0) == "助動詞"
                    && matches!(field(following, 4), "助動詞-ダ" | "助動詞-デス")
            })
        {
            return Transition::Attach(vec!["obligation".into()]);
        }
        return Transition::Attach(vec![config::classical_rule_for_input(
            next,
            &previous.final_state,
        )
        .unwrap_or(&entry.rule)
        .to_owned()]);
    }
    if let Some(position) = InsertionExecutor::position(previous, next, context, source) {
        return Transition::Insert(position.id.as_str());
    }
    if let Some(rules) = IAdjectiveConversionExecutor::continuation(previous, next) {
        return Transition::Attach(rules);
    }
    let last = &source[previous.morpheme_range[1] - 1];
    let following = source.get(next.index + 1);
    let aux_relation = context
        .relation(previous.char_range, next.char_range, &["aux", "cop"])
        .is_some();
    for rule in &CONFIG.connections {
        let matched = rule.when.iter().any(|condition| {
            condition.next.matches(next)
                && condition.last.matches(last)
                && condition
                    .following
                    .as_ref()
                    .is_none_or(|pattern| following.is_some_and(|token| pattern.matches(token)))
                && (!condition.at_end || following.is_none())
                && (condition.forms.is_empty()
                    || condition.forms.contains(&previous.final_state.form))
                && (condition.categories.is_empty()
                    || condition
                        .categories
                        .contains(&previous.final_state.category))
                && (condition.operators.is_empty()
                    || follows_operator(previous, &condition.operators))
                && (condition.suffixes.is_empty()
                    || condition
                        .suffixes
                        .iter()
                        .any(|suffix| previous.surface_form.ends_with(suffix)))
                && (!condition.auxiliary_relation || aux_relation)
                && (!condition.functional
                    || field(next, 1).starts_with("非自立")
                    || context.auxiliary(next.char_range)
                    || context.functional_position(next.char_range)
                    || aux_relation)
                && (!condition.sahen
                    || (field(last, 0) == "名詞"
                        && (field(last, 2).contains("サ変")
                            || context
                                .tag(last.char_range)
                                .is_some_and(|tag| tag.contains("サ変")))))
        });
        if !matched {
            continue;
        }
        match rule.action {
            Action::Core => return Transition::Core,
            Action::Stop => return Transition::Boundary,
            Action::Attach => {
                return Transition::Attach(if rule.kinds.is_empty() {
                    semantic(next)
                } else {
                    rule.kinds.clone()
                })
            }
            Action::Support => {
                if let Some(kind) = config::support_kind(&rule.support_group, next) {
                    return Transition::Support(kind);
                }
            }
        }
    }
    Transition::Stop
}

fn operator(
    chain: &MorphologyChain,
    atom: &MorphologyChain,
    kind: String,
    token: &ProviderToken,
) -> MorphologyOperator {
    let metadata = config::rule_metadata(&kind);
    let definition = config::definition(&kind);
    MorphologyOperator {
        operator_id: format!("morphology:{}:{kind}", atom.anchor_morpheme),
        source_morpheme_range: atom.morpheme_range,
        char_range: atom.char_range,
        input_state: chain.surface_form.clone(),
        output_state: format!("{}{}", chain.surface_form, atom.surface_form),
        state_before: chain.final_state.clone(),
        state_after: atom.final_state.clone(),
        normalized_form: definition
            .and_then(|entry| entry.normalized.clone())
            .or_else(|| config::prototype(&kind, token)),
        concept_id: feature_concept(&kind).to_owned(),
        confidence: 1.0,
        evidence: atom.evidence.clone(),
        candidates: config::definition(&kind)
            .map_or_else(Vec::new, |entry| entry.candidates.clone()),
        label: feature_label(&kind).to_owned(),
        description: format!("{}：{}", feature_label(&kind), atom.surface_form),
        kind,
        stage: metadata.map_or_else(String::new, |m| m.stage.clone()),
        executor: metadata.map_or_else(String::new, |m| m.executor.clone()),
        rule_group: metadata.map_or_else(String::new, |m| m.rule_group.clone()),
        input_category: metadata.map_or_else(String::new, |m| m.input_category.clone()),
        output_category: metadata.map_or_else(String::new, |m| m.output_category.clone()),
        terminal_mode: metadata.map_or_else(String::new, |m| m.terminal_mode.clone()),
        order_rank: metadata.map_or(0, |m| m.order_rank),
        connection: None,
    }
}

fn merge(chain: &mut MorphologyChain, atom: MorphologyChain, core: bool) {
    if atom.status == "candidate" {
        chain.status = "candidate".into();
    }
    if core {
        let prefix = chain.surface_form.clone();
        let prefix_reading = chain
            .query_forms
            .iter()
            .find(|f| f.kind == "observed")
            .and_then(|f| f.reading.clone());
        let observed_reading = prefix_reading
            .as_ref()
            .zip(
                atom.query_forms
                    .iter()
                    .find(|f| f.kind == "observed")
                    .and_then(|f| f.reading.as_ref()),
            )
            .map(|(a, b)| format!("{a}{b}"));
        chain.dictionary_form = format!("{}{}", prefix, atom.dictionary_form);
        chain.lookup_form = chain.dictionary_form.clone();
        chain.display_form = chain.dictionary_form.clone();
        chain.lemma_form = format!("{}{}", prefix, atom.lemma_form);
        chain
            .core_morpheme_indices
            .extend(&atom.core_morpheme_indices);
        chain.query_forms = atom
            .query_forms
            .iter()
            .filter(|f| f.kind != "observed")
            .map(|f| QueryForm {
                kind: f.kind.clone(),
                form: format!("{prefix}{}", f.form),
                reading: prefix_reading
                    .as_ref()
                    .zip(f.reading.as_ref())
                    .map(|(a, b)| format!("{a}{b}")),
                reading_field: Some("composed".into()),
            })
            .collect();
        chain.query_forms.insert(
            0,
            QueryForm {
                kind: "observed".into(),
                form: format!("{prefix}{}", atom.surface_form),
                reading_field: observed_reading.as_ref().map(|_| "composed_kana".into()),
                reading: observed_reading,
            },
        );
        if !chain
            .query_forms
            .iter()
            .any(|f| f.form == chain.dictionary_form)
        {
            chain.query_forms.push(QueryForm {
                kind: "base".into(),
                form: chain.dictionary_form.clone(),
                reading: None,
                reading_field: None,
            });
        }
    }
    chain.surface_form.push_str(&atom.surface_form);
    chain.morpheme_range[1] = atom.morpheme_range[1];
    chain.char_range[1] = atom.char_range[1];
    chain.source_ranges.extend(atom.source_ranges);
    chain.morpheme_indices.extend(atom.morpheme_indices);
    chain.operators.extend(
        atom.operators
            .into_iter()
            .filter(|o| o.kind != "conjugation"),
    );
    chain.connection_forms.extend(atom.connection_forms);
    chain.evidence.extend(atom.evidence);
    chain.final_state = atom.final_state;
}

fn merge_verb_core(
    chain: &mut MorphologyChain,
    atom: MorphologyChain,
    current: &ProviderToken,
    node: &SourceNode,
) {
    let lemma = node.features["lemma"].as_str().unwrap();
    let (conjugation_type, conjugation_form) = node.features["morph"]["Inflection"]
        .as_str()
        .unwrap()
        .split_once(';')
        .unwrap();
    let reading = node.features["morph"]["Reading"]
        .as_str()
        .map(str::to_owned);
    let mut supplemented = current.clone();
    supplemented.fields[0].value = Some("動詞".into());
    supplemented.fields[4].value = Some(conjugation_type.into());
    supplemented.fields[5].value = Some(conjugation_form.into());
    let mut verb_state = state(&supplemented);
    // GiNZA 已确认这是词汇动词核心；即使其 cType 使用文言范式，
    // 词汇核心仍属于动词循环类，文言类别留给文言助动词接续。
    verb_state.category = "動詞".into();
    verb_state.entity_class = "动词循环类".into();
    let mut conjugation = operator(chain, &atom, "conjugation".into(), current);
    conjugation.input_state = lemma.into();
    conjugation.output_state = conjugation_form.into();
    conjugation.label = format!("{conjugation_type}/{conjugation_form}");
    conjugation.description = format!("GiNZA 活用字段：{}", conjugation.label);
    conjugation.evidence = vec![format!("ginza:{}:Inflection", node.id)];
    conjugation.source_morpheme_range = [chain.anchor_morpheme, atom.morpheme_range[1]];
    conjugation.char_range = [chain.char_range[0], atom.char_range[1]];
    conjugation.state_before = MorphologyState {
        category: "動詞".into(),
        form: ConnectionForm::Terminal,
        conjugation_type: conjugation_type.into(),
        conjugation_form: "基本形".into(),
        entity_class: "动词循环类".into(),
    };
    conjugation.state_after = verb_state.clone();
    chain.source_evidence.extend(atom.source_evidence.clone());
    chain
        .source_evidence
        .push(crate::linguistic_context::SourceEvidence {
            provider: "ginza".into(),
            node_id: Some(node.id.clone()),
            relation_id: None,
            reason: "verb_core".into(),
        });
    merge(chain, atom, true);
    chain.dictionary_form = lemma.into();
    chain.lemma_form = lemma.into();
    chain.lookup_form = lemma.into();
    chain.display_form = lemma.into();
    chain.final_state = verb_state;
    chain.query_forms = vec![QueryForm {
        kind: "observed".into(),
        form: chain.surface_form.clone(),
        reading: reading.clone(),
        reading_field: reading.as_ref().map(|_| "ginza:morph.Reading".into()),
    }];
    if lemma != chain.surface_form {
        chain.query_forms.push(QueryForm {
            kind: "base".into(),
            form: lemma.into(),
            reading: None,
            reading_field: None,
        });
    }
    chain.operators.push(conjugation);
    chain.evidence.push(format!("ginza:{}:verb_core", node.id));
}

fn suffix_sequence(
    previous: &MorphologyChain,
    index: usize,
    source: &[ProviderToken],
    context: &Context<'_>,
) -> Option<&'static config::Sequence> {
    // 从连接器扫描整个插入区间，确认后再让固定词尾让位。
    if let Transition::Attach(kinds) = transition(previous, &source[index], context, source) {
        let mut end = index + 1;
        while source.get(end).is_some_and(insertion_token) {
            end += 1;
        }
        if end > index + 1
            && end < source.len()
            && source[index..=end]
                .windows(2)
                .all(|pair| pair[0].char_range[1] == pair[1].char_range[0])
            && CONFIG.insertion_positions.iter().any(|position| {
                !position.connector.is_empty()
                    && kinds.contains(&position.connector)
                    && config::support_kind(&position.support_group, &source[end]).is_some()
                    && (context.ginza.is_none()
                        || context.confirms_inserted_support(
                            source[previous.core_morpheme_indices[0]].char_range,
                            source[index].char_range,
                            &source[index + 1..end]
                                .iter()
                                .map(|token| token.char_range)
                                .collect::<Vec<_>>(),
                            source[end].char_range,
                        ))
            })
        {
            return None;
        }
    }
    CONFIG.sequences.iter().find(|rule| {
        let end = index + rule.tokens.len();
        end <= source.len()
            && previous.morpheme_range[1] == index
            && previous.char_range[1] == source[index].char_range[0]
            && rule.forms.contains(&previous.final_state.form)
            && (rule.categories.is_empty()
                || rule.categories.contains(&previous.final_state.category))
            && (rule.operators.is_empty() || follows_operator(previous, &rule.operators))
            && rule
                .tokens
                .iter()
                .zip(&source[index..end])
                .all(|(pattern, token)| pattern.matches(token))
            && source[index..end]
                .windows(2)
                .all(|pair| pair[0].char_range[1] == pair[1].char_range[0])
            && !rule
                .exclude_following
                .as_ref()
                .is_some_and(|pattern| source.get(end).is_some_and(|token| pattern.matches(token)))
            && (!rule.terminal_only
                || source.get(end).is_none_or(|token| {
                    field(token, 0) == "補助記号" && matches!(field(token, 1), "句点" | "括弧閉")
                }))
    })
}

/// 后项先完成自身活用，再按完整否定证据确认义务或禁止构式。
fn link_predicates(chains: &mut [MorphologyChain], source: &[ProviderToken]) {
    for index in 1..chains.len() {
        let (before, after) = chains.split_at_mut(index);
        let left = &mut before[index - 1];
        let right = &mut after[0];
        if right.parent_chain_id.is_some()
            || left.char_range[1] != right.char_range[0]
            || left.morpheme_range[1] != right.morpheme_range[0]
        {
            continue;
        }
        for rule in &CONFIG.predicate_links {
            let Some(prefix) = left
                .operators
                .iter()
                .find(|op| op.kind == rule.prefix && op.char_range[1] == left.char_range[1])
                .cloned()
            else {
                continue;
            };
            if !rule.next.matches(&source[right.anchor_morpheme]) {
                continue;
            }
            let body_end = if !rule.required_operators.is_empty() {
                let Some(op) = right
                    .operators
                    .iter()
                    .find(|op| rule.required_operators.contains(&op.kind))
                else {
                    continue;
                };
                op.source_morpheme_range[1]
            } else {
                right.anchor_morpheme + 1
            };
            let mut construction = operator(
                left,
                right,
                rule.kind.clone(),
                &source[prefix.source_morpheme_range[0]],
            );
            construction.source_morpheme_range = [prefix.source_morpheme_range[0], body_end];
            construction.char_range = [prefix.char_range[0], source[body_end - 1].char_range[1]];
            construction.state_before = prefix.state_before;
            construction.state_after = state(&source[body_end - 1]);
            construction.evidence.extend(prefix.evidence);
            construction
                .evidence
                .push("connection:predicate_construction".into());
            construction.output_state = format!("{}{}", left.surface_form, right.surface_form);
            left.operators
                .retain(|op| op.operator_id != prefix.operator_id);
            right.operators.retain(|op| {
                op.source_morpheme_range[1] > body_end
                    || config::definition(&op.kind).is_some_and(|entry| entry.output_form.is_some())
                    || !config::is_inflection(&op.kind)
                        && !rule.required_operators.contains(&op.kind)
            });
            right.operators.insert(0, construction);
            right.role = MorphologyRole::Functional;
            right.parent_chain_id = Some(left.chain_id.clone());
            break;
        }
    }
}

pub(crate) fn compose(
    atoms: Vec<MorphologyChain>,
    source: &[ProviderToken],
    morphemes: &[MorphemeToken],
    external: &[SourceArtifact],
) -> MorphologyArtifact {
    let context = Context::new(external);
    let mut chains: Vec<MorphologyChain> = Vec::new();
    let mut diagnostics = Vec::new();
    let mut atoms: std::collections::VecDeque<_> = atoms.into();
    while let Some(mut atom) = atoms.pop_front() {
        let index = atom.anchor_morpheme;
        let current = &source[index];
        if let Some(previous) = chains.last_mut().filter(|previous|
            previous.morpheme_range[1] == index && previous.char_range[1] == atom.char_range[0]
                && previous.morpheme_indices.len() == 1) {
            if let Some(recovery) = CONFIG.input_recoveries.iter().find(|rule| {
                rule.previous.matches(&source[index - 1]) && rule.next.matches(current)
                    && index + 1 + rule.following.len() <= source.len()
                    && rule.following.iter().zip(&source[index + 1..])
                        .all(|(pattern, token)| pattern.matches(token))
                    && source[index - 1..index + 1 + rule.following.len()].windows(2)
                        .all(|pair| pair[0].char_range[1] == pair[1].char_range[0])
            }) {
                atom.final_state = MorphologyState {
                    category: "形容詞".into(), form: ConnectionForm::Irrealis,
                    conjugation_type: recovery.ctype.clone(), conjugation_form: "未然形-一般".into(),
                    entity_class: "イ形容词转换类".into(),
                };
                merge(previous, atom, true);
                previous.base_lexeme = recovery.lemma.clone();
                previous.dictionary_form = recovery.lemma.clone();
                previous.lemma_form = recovery.lemma.clone();
                previous.lookup_form = recovery.lemma.clone();
                previous.display_form = recovery.lemma.clone();
                previous.query_forms = vec![
                    QueryForm { kind: "base".into(), form: recovery.lemma.clone(), reading: Some(recovery.reading.clone()), reading_field: Some("configured_recovery".into()) },
                    QueryForm { kind: "observed".into(), form: previous.surface_form.clone(), reading: Some(recovery.surface_reading.clone()), reading_field: Some("configured_recovery".into()) },
                ];
                previous.evidence.push(format!("input_recovery:{}", recovery.id));
                continue;
            }
        }
        let sequence = chains
            .last()
            .and_then(|previous| suffix_sequence(previous, index, source, &context));
        if let Some(rule) = sequence {
            let first_state = atom.final_state.clone();
            for _ in 1..rule.tokens.len() {
                merge(
                    &mut atom,
                    atoms.pop_front().expect("连续词尾成员缺失"),
                    false,
                );
            }
            atom.operators.clear();
            atom.evidence.extend(rule.phenomena.iter().cloned());
            if rule.kind == "negative_zuni" {
                atom.final_state = first_state;
            }
        }
        let rule_token = sequence.map_or(current, |rule| &source[index + rule.head]);
        atom.source_evidence = context.evidence(atom.char_range);
        if atom.final_state.conjugation_form.is_empty() {
            if let Some((ctype, cform)) = context
                .inflection(atom.char_range)
                .and_then(|f| f.split_once(';'))
            {
                let mut supplemented = current.clone();
                if supplemented.fields.len() >= 6 {
                    supplemented.fields[4].value = Some(ctype.into());
                    supplemented.fields[5].value = Some(cform.into());
                    atom.final_state = state(&supplemented);
                    atom.evidence
                        .push("ginza:Inflection:missing_unidic_form".into());
                }
            }
        }
        let mut linked = false;
        let previous_index = chains.len().checked_sub(1).filter(|&i| {
            chains[i].char_range[1] == atom.char_range[0] && chains[i].morpheme_range[1] == index
        });
        if let Some(previous_index) = previous_index {
            let auxiliary_depth = auxiliary_support_depth(&chains, previous_index);
            let previous = &mut chains[previous_index];
            let first = previous.core_morpheme_indices[0];
            let verb_core = (previous.morpheme_indices.len() == 1
                && field(&source[first], 0) == "名詞"
                && field(current, 0) == "助動詞")
                .then(|| {
                    context.verb_core(
                        [previous.char_range[0], atom.char_range[1]],
                        &source[first].surface,
                    )
                })
                .flatten();
            if let Some(node) = verb_core.filter(|_| sequence.is_none()) {
                merge_verb_core(previous, atom, current, node);
                continue;
            }
            let members = context.core_members(morphemes, first);
            let local_step = sequence.map_or_else(
                || transition(previous, current, &context, source),
                |rule| {
                    if config::definition(&rule.kind)
                        .is_some_and(|definition| definition.support.is_some())
                    {
                        Transition::Support(rule.kind.as_str())
                    } else {
                        Transition::Attach(vec![rule.kind.clone()])
                    }
                },
            );
            let nominalization = matches!(&local_step, Transition::Attach(kinds) if kinds.iter().any(|kind|
                config::rule_metadata(kind).is_some_and(|metadata| metadata.executor == "nominalization")));
            let semantic_connection = nominalization
                || matches!(&local_step, Transition::Support(_) | Transition::Insert(_));
            let step = if sequence.is_none()
                && members.contains(&index)
                && !semantic_connection
                && !matches!(local_step, Transition::Boundary)
            {
                Transition::Core
            } else {
                local_step
            };
            match step {
                Transition::Core => {
                    previous.evidence.push("connection:lexical_core".into());
                    let prior = &source[previous.morpheme_range[1] - 1];
                    if field(prior, 0) == "動詞" && field(current, 0) == "名詞" {
                        previous.status = "candidate".into();
                        diagnostics.push(format!(
                            "lexical_core_pos_conflict:{}:{index}",
                            previous.chain_id
                        ));
                    }
                    previous
                        .source_evidence
                        .extend(atom.source_evidence.clone());
                    let mut rule_input = previous.clone();
                    for kind in semantic(current) {
                        apply_rule_executor(
                            &rule_input,
                            &mut atom,
                            &kind,
                            current,
                            &mut diagnostics,
                            index,
                        );
                        previous
                            .operators
                            .push(operator(&rule_input, &atom, kind, current));
                        rule_input.final_state = atom.final_state.clone();
                    }
                    merge(previous, atom, true);
                    continue;
                }
                Transition::Attach(mut kinds) => {
                    let before = previous.final_state.clone();
                    for kind in &kinds {
                        if let Some(entry) = config::definition(kind) {
                            if before.category == "形状詞" {
                                if let Some(suffix) = &entry.display_suffix {
                                    previous.display_form =
                                        format!("{}{suffix}", previous.surface_form);
                                }
                            }
                        }
                    }
                    if kinds.is_empty() {
                        kinds.push("auxiliary".into());
                    }
                    let mut rule_input = previous.clone();
                    let mut attached = false;
                    for kind in kinds {
                        apply_rule_executor(
                            &rule_input,
                            &mut atom,
                            &kind,
                            rule_token,
                            &mut diagnostics,
                            index,
                        );
                        attached |= atom.status != "candidate";
                        previous
                            .operators
                            .push(operator(&rule_input, &atom, kind, rule_token));
                        rule_input.final_state = atom.final_state.clone();
                        if terminal_mode(&previous.operators.last().unwrap().kind) != "preserve" {
                            previous.terminal = may_stop(
                                &previous.operators.last().unwrap().kind,
                                &atom.final_state,
                            );
                        }
                    }
                    if attached && previous.status == "pending" {
                        previous.status = "resolved".into();
                    }
                    if let Some(e) = context.relation(
                        previous.char_range,
                        atom.char_range,
                        &["aux", "cop", "mark"],
                    ) {
                        previous.source_evidence.push(e);
                    }
                    if context.ginza.is_some()
                        && !context.same_bunsetsu(previous.char_range, atom.char_range)
                    {
                        diagnostics.push(format!("cross_bunsetsu:{}:{}", previous.chain_id, index));
                    }
                    previous
                        .source_evidence
                        .extend(atom.source_evidence.clone());
                    merge(previous, atom, false);
                    continue;
                }
                Transition::Support(kind) => {
                    AuxiliaryVerbExecutor::apply(
                        previous,
                        &mut atom,
                        kind,
                        source,
                        index,
                        auxiliary_depth,
                        &mut diagnostics,
                    );
                    atom.operators
                        .push(operator(previous, &atom, kind.into(), current));
                    if let Some(connector) = config::definition(kind)
                        .and_then(|definition| definition.connector.as_deref())
                    {
                        atom.evidence.push(format!("connector_rule:{connector}"));
                    }
                    atom.terminal = matches!(
                        terminal_mode(&kind),
                        "sentence_or_pause" | "pause_or_continue"
                    );
                    let relation = context.relation(
                        previous.char_range,
                        atom.char_range,
                        &["aux", "cop", "advcl", "xcomp"],
                    );
                    let bridged = bridge_support(previous, index, source, &context);
                    // 连接器、输入类别、连续范围和递归深度是补助构式的必要条件。
                    // GiNZA 依存边和文节范围只提供外部证据，缺少外部结果时不能否定已通过的词法连接。
                    let structural_evidence = bridged
                        || relation.is_some()
                        || context.same_word(previous.char_range, atom.char_range)
                        || context.same_bunsetsu(previous.char_range, atom.char_range);
                    if atom.status != "candidate" {
                        atom.status = "resolved".into();
                    }
                    if let Some(e) = relation {
                        atom.source_evidence.push(e);
                    }
                    if bridged {
                        atom.source_evidence
                            .extend(previous.source_evidence.clone());
                    }
                    if let Some(source) = context.ginza {
                        for node in source.nodes.iter().filter(|n| {
                            n.kind == crate::external::NodeKind::Bunsetsu
                                && n.text_ranges.len() == 1
                                && n.text_ranges[0][0] <= previous.char_range[0]
                                && atom.char_range[1] <= n.text_ranges[0][1]
                        }) {
                            atom.source_evidence
                                .push(crate::linguistic_context::SourceEvidence {
                                    provider: "ginza".into(),
                                    node_id: Some(node.id.clone()),
                                    relation_id: None,
                                    reason: "support_bunsetsu".into(),
                                });
                        }
                    }
                    if let Some([base, reading]) = config::definition(kind)
                        .and_then(|entry| entry.support.as_ref())
                        .and_then(|support| support.expanded.as_ref())
                    {
                        atom.display_form = atom
                            .operators
                            .last()
                            .and_then(|op| op.normalized_form.clone())
                            .unwrap();
                        atom.dictionary_form = base.clone();
                        atom.lookup_form = base.clone();
                        atom.query_forms.push(QueryForm {
                            kind: "expanded_auxiliary".into(),
                            form: base.clone(),
                            reading: Some(reading.clone()),
                            reading_field: Some("contraction_rule".into()),
                        });
                    }
                    if !structural_evidence || context.ginza.is_none() {
                        atom.evidence.push("support_lexical_connection_only".into());
                    }
                    linked = true;
                }
                Transition::Insert(position) => {
                    atom.evidence.push(format!("insertion_position:{position}"));
                    apply_rule_executor(
                        previous,
                        &mut atom,
                        "insertion",
                        current,
                        &mut diagnostics,
                        index,
                    );
                    if let Some(relation) = context.depends_on(
                        atom.char_range,
                        source[previous.core_morpheme_indices[0]].char_range,
                        &["case", "mark"],
                    ) {
                        atom.evidence
                            .push(format!("insertion_relation:{}", relation.reason));
                        atom.source_evidence.push(relation);
                    }
                    previous
                        .operators
                        .push(operator(previous, &atom, "insertion".into(), current));
                    previous
                        .source_evidence
                        .extend(atom.source_evidence.clone());
                    merge(previous, atom, false);
                    continue;
                }
                Transition::Boundary | Transition::Stop => {
                    if context
                        .relation(previous.char_range, atom.char_range, &["aux", "cop"])
                        .is_some()
                    {
                        diagnostics.push(format!(
                            "source_connection_conflict:{}:{}",
                            previous.chain_id, index
                        ));
                        atom.status = "candidate".into();
                    }
                }
            }
        }
        if let Some((entry, form)) = config::classical_paradigm_form(current) {
            atom.evidence
                .push(format!("classical_paradigm:{}:{form}", entry.class_name));
            atom.evidence
                .push(format!("classical_lemma:{}", entry.lemma));
            atom.evidence.push(format!(
                "classical_connection:{}",
                entry.connection.join("／")
            ));
        }
        for kind in semantic(current) {
            if kind == "classical_adjective" {
                let input = atom.clone();
                apply_rule_executor(&input, &mut atom, &kind, current, &mut diagnostics, index);
                atom.terminal = may_stop(&kind, &atom.final_state);
            }
            let mut op = operator(&atom, &atom, kind, current);
            op.input_state = atom.dictionary_form.clone();
            op.output_state = atom.surface_form.clone();
            atom.operators.push(op);
        }
        if !linked && field(current, 0) == "助動詞" {
            atom.status = "pending".into();
        }
        chains.push(atom);
    }
    for chain in &chains {
        if chain.status == "pending" && chain.role == MorphologyRole::Functional {
            diagnostics.push(format!("unattached_auxiliary:{}", chain.anchor_morpheme));
        }
    }
    link_predicates(&mut chains, source);
    identify_connection_paths(&mut chains, source, &mut diagnostics);
    let occurrences = chains
        .iter()
        .flat_map(|chain| {
            chain
                .operators
                .iter()
                .filter(|op| !config::is_inflection(&op.kind))
                .map(|op| {
                    let mut hit_ranges = vec![op.char_range];
                    let mut members: Vec<_> =
                        (op.source_morpheme_range[0]..op.source_morpheme_range[1]).collect();
                    let mut operator_ids = vec![op.operator_id.clone()];
                    let mut context_range = chain.char_range;
                    if let Some(connector_kind) = config::definition(&op.kind)
                        .and_then(|entry| entry.connector.as_ref())
                        .filter(|_| chain.parent_chain_id.is_some())
                    {
                        if let Some(parent) = chains
                            .iter()
                            .find(|c| Some(&c.chain_id) == chain.parent_chain_id.as_ref())
                        {
                            if let Some(connector) =
                                parent.operators.iter().find(|o| &o.kind == connector_kind)
                            {
                                hit_ranges.insert(0, connector.char_range);
                                members.splice(
                                    0..0,
                                    connector.source_morpheme_range[0]
                                        ..connector.source_morpheme_range[1],
                                );
                                operator_ids.insert(0, connector.operator_id.clone());
                                for inserted in parent.operators.iter().filter(|inserted| {
                                    inserted.executor == "insertion"
                                        && inserted.source_morpheme_range[0]
                                            >= connector.source_morpheme_range[1]
                                        && inserted.source_morpheme_range[1]
                                            <= op.source_morpheme_range[0]
                                }) {
                                    hit_ranges.push(inserted.char_range);
                                    members.extend(
                                        inserted.source_morpheme_range[0]
                                            ..inserted.source_morpheme_range[1],
                                    );
                                    operator_ids.insert(
                                        operator_ids.len() - 1,
                                        inserted.operator_id.clone(),
                                    );
                                }
                                hit_ranges.sort_unstable_by_key(|range| range[0]);
                                members.sort_unstable();
                                members.dedup();
                            }
                        }
                    }
                    let mut parent = chain.parent_chain_id.as_ref();
                    while let Some(id) = parent {
                        let Some(ancestor) = chains.iter().find(|c| &c.chain_id == id) else {
                            break;
                        };
                        context_range[0] = ancestor.char_range[0];
                        parent = ancestor.parent_chain_id.as_ref();
                    }
                    let char_range = if hit_ranges.len() > 1 {
                        [hit_ranges[0][0], op.char_range[1]]
                    } else {
                        op.char_range
                    };
                    MorphologyOccurrence {
                        id: format!("occurrence:{}", op.operator_id),
                        chain_id: chain.chain_id.clone(),
                        operator_ids,
                        kind: op.kind.clone(),
                        char_range,
                        context_range,
                        hit_ranges,
                        morpheme_indices: members,
                        candidates: op.candidates.clone(),
                        status: chain.status.clone(),
                        source_evidence: chain.source_evidence.clone(),
                    }
                })
        })
        .collect();
    MorphologyArtifact {
        schema: SCHEMA.into(),
        chains,
        occurrences,
        diagnostics,
    }
}

/// 只回看路径声明的语素和当前输入状态，保留全部匹配以检测规则冲突。
fn identify_connection_paths(
    chains: &mut [MorphologyChain],
    source: &[ProviderToken],
    diagnostics: &mut Vec<String>,
) {
    for chain in chains.iter_mut() {
        for index in 0..chain.operators.len() {
            let op = &chain.operators[index];
            let Some((rule_id, rule)) = CONFIG.connection_rules.iter()
                .find(|(_, rule)| rule.operators.contains(&op.kind)) else { continue };
            let start = op.source_morpheme_range[0];
            // 独立词条没有前接对象，其词法边界由原分析保留。
            if start == 0 || (start == chain.morpheme_range[0] && chain.parent_chain_id.is_none())
                || source[start - 1].char_range[1] != op.char_range[0]
                || op.input_state == op.output_state { continue; }
            let state = &op.state_before;
            let matched: Vec<_> = rule.paths.iter().filter(|path| {
                (path.categories.is_empty() || path.categories.contains(&state.category))
                    && (path.forms.is_empty() || path.forms.contains(&state.form))
                    && (path.ctypes.is_empty() || path.ctypes.iter().any(|v| state.conjugation_type.starts_with(v)))
                    && !path.exclude_ctypes.iter().any(|v| state.conjugation_type.starts_with(v))
                    && (path.cform_contains.is_empty() || path.cform_contains.iter().any(|v| state.conjugation_form.contains(v)))
                    && !path.exclude_cform_contains.iter().any(|v| state.conjugation_form.contains(v))
                    && (path.previous_operators.is_empty() || chain.operators[..index].iter().rev()
                        .find(|p| p.executor != "insertion").is_some_and(|p| path.previous_operators.contains(&p.kind)))
                    && !chain.operators[..index].iter().rev().find(|p| p.executor != "insertion")
                        .is_some_and(|p| path.exclude_previous_operators.contains(&p.kind))
                    && path.next.matches(&source[start])
                    && start >= path.predecessors.len()
                    && path.predecessors.iter().rev().zip(source[..start].iter().rev())
                        .all(|(pattern, token)| pattern.matches(token))
                    && source[start.saturating_sub(path.predecessors.len().max(1))..=start]
                        .windows(2).all(|pair| pair[0].char_range[1] == pair[1].char_range[0])
            }).collect();
            let mut messages = Vec::new();
            if matched.len() != 1 {
                let message = format!("connection_path_count:{}:{}:{}", op.operator_id, rule_id, matched.len());
                diagnostics.push(message.clone());
                messages.push(message);
            }
            let count = matched.first().map_or(1, |path| path.predecessors.len().max(1));
            let range = [source[start.saturating_sub(count)].char_range[0], op.char_range[0]];
            let connection = MorphologyConnection {
                rule: rule_id.clone(),
                path: (matched.len() == 1).then(|| matched[0].id.clone()),
                matched_paths: matched.iter().map(|path| path.id.clone()).collect(),
                predecessor_range: range,
                predecessor_surface: source[start.saturating_sub(count)..start].iter().map(|t| t.surface.as_str()).collect(),
                connector_operator_id: None,
                diagnostics: messages,
            };
            if connection.path.is_none() { chain.status = "candidate".into(); }
            chain.operators[index].connection = Some(connection);
        }
    }
    // 补助条目复用实际父链连接器的命中结果。
    let all_chains = &*chains;
    let inherited: Vec<_> = all_chains.iter().enumerate().flat_map(|(ci, chain)| {
        chain.operators.iter().enumerate().filter_map(move |(oi, op)| {
            let connector = config::definition(&op.kind)?.connector.as_ref()?;
            let parent = all_chains.iter().find(|p| Some(&p.chain_id) == chain.parent_chain_id.as_ref())?;
            let source = parent.operators.iter().rev().find(|p| &p.kind == connector)?;
            let mut connection = source.connection.clone()?;
            connection.connector_operator_id = Some(source.operator_id.clone());
            Some((ci, oi, connection))
        })
    }).collect();
    for (ci, oi, connection) in inherited { chains[ci].operators[oi].connection = Some(connection); }
}
