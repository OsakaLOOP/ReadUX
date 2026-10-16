//! 词法连接状态机。GiNZA 参与复合核心、功能所有权和来源冲突判定。
use crate::{external::{SourceArtifact, SourceNode}, linguistic_context::Context, model::{MorphemeToken, ProviderToken, QueryForm}, morphology::*};

use crate::morphology_config::{self as config, Action, CONFIG, field, identity};

pub(crate) fn state(t: &ProviderToken) -> MorphologyState {
    let raw = field(t, 5);
    let form = CONFIG.forms.iter().find(|rule| rule.prefixes.iter().any(|prefix| raw.starts_with(prefix)))
        .map(|rule| rule.form.clone()).unwrap_or_else(|| if raw.is_empty() { ConnectionForm::Stem } else { ConnectionForm::Other });
    let category = if field(t, 0) == "接尾辞" && field(t, 1) == "名詞的" { "名詞" } else { field(t, 0) };
    MorphologyState { category: category.into(), form, conjugation_type: field(t, 4).into(), conjugation_form: raw.into() }
}

fn semantic(t: &ProviderToken) -> Vec<String> { config::features(t) }

enum Transition { Core, Attach(Vec<String>), Support(&'static str), Boundary, Stop }

fn insertion_token(token: &ProviderToken) -> bool {
    CONFIG.insertion.iter().any(|pattern| pattern.matches(token))
}

fn bridge_support(previous: &MorphologyChain, index: usize, source: &[ProviderToken], context: &Context<'_>) -> bool {
    if previous.final_state.form != ConnectionForm::Te || previous.morpheme_range[1] >= index { return false; }
    let Some(connector) = previous.operators.iter().rev().find(|operator| operator.kind == "te_connection").map(|operator| operator.char_range) else { return false; };
    let inserted = (previous.morpheme_range[1]..index).map(|i| &source[i]).collect::<Vec<_>>();
    if inserted.is_empty() || inserted.iter().any(|token| !insertion_token(token)) { return false; }
    let Some(predicate_index) = previous.core_morpheme_indices.first() else { return false; };
    let Some(predicate) = source.get(*predicate_index).map(|token| token.char_range) else { return false; };
    context.confirms_inserted_support(predicate, connector,
        &inserted.iter().map(|token| token.char_range).collect::<Vec<_>>(), source[index].char_range)
}

fn transition(previous: &MorphologyChain, next: &ProviderToken, context: &Context<'_>, source: &[ProviderToken]) -> Transition {
    let last = &source[previous.morpheme_range[1] - 1];
    let following = source.get(next.index + 1);
    let aux_relation = context.relation(previous.char_range, next.char_range, &["aux", "cop"]).is_some();
    for rule in &CONFIG.connections {
        let matched = rule.when.iter().any(|condition| {
            condition.next.matches(next) && condition.last.matches(last)
                && condition.following.as_ref().is_none_or(|pattern| following.is_some_and(|token| pattern.matches(token)))
                && (!condition.at_end || following.is_none())
                && (condition.forms.is_empty() || condition.forms.contains(&previous.final_state.form))
                && (condition.categories.is_empty() || condition.categories.contains(&previous.final_state.category))
                && (condition.operators.is_empty() || previous.operators.iter().any(|op| condition.operators.contains(&op.kind)))
                && (condition.suffixes.is_empty() || condition.suffixes.iter().any(|suffix| previous.surface_form.ends_with(suffix)))
                && (!condition.auxiliary_relation || aux_relation)
                && (!condition.functional || field(next, 1).starts_with("非自立") || context.auxiliary(next.char_range)
                    || context.functional_position(next.char_range) || aux_relation)
                && (!condition.sahen || (field(last, 0) == "名詞" && (field(last, 2).contains("サ変")
                    || context.tag(last.char_range).is_some_and(|tag| tag.contains("サ変")))))
        });
        if !matched { continue; }
        match rule.action {
            Action::Core => return Transition::Core,
            Action::Stop => return Transition::Boundary,
            Action::Attach => return Transition::Attach(if rule.kinds.is_empty() { semantic(next) } else { rule.kinds.clone() }),
            Action::Support => {
                if let Some(kind) = config::support_kind(&rule.support_group, next) {
                    return Transition::Support(kind);
                }
            }
        }
    }
    Transition::Stop
}

fn operator(chain: &MorphologyChain, atom: &MorphologyChain, kind: String, token: &ProviderToken) -> MorphologyOperator {
    MorphologyOperator { operator_id: format!("morphology:{}:{kind}", atom.anchor_morpheme),
        source_morpheme_range: atom.morpheme_range, char_range: atom.char_range,
        input_state: chain.surface_form.clone(), output_state: format!("{}{}", chain.surface_form, atom.surface_form),
        state_before: chain.final_state.clone(), state_after: atom.final_state.clone(),
        normalized_form: config::definition(&kind).and_then(|entry| entry.normalized.clone()).or_else(|| config::prototype(&kind, token)),
        concept_id: feature_concept(&kind).into(), confidence: 1.0, evidence: atom.evidence.clone(),
        candidates: config::definition(&kind).map_or_else(Vec::new, |entry| entry.candidates.clone()),
        label: feature_label(&kind).into(), description: format!("{}：{}", feature_label(&kind), atom.surface_form), kind }
}

fn merge(chain: &mut MorphologyChain, atom: MorphologyChain, core: bool) {
    if core {
        let prefix = chain.surface_form.clone();
        let prefix_reading = chain.query_forms.iter().find(|f| f.kind == "observed").and_then(|f| f.reading.clone());
        let observed_reading = prefix_reading.as_ref().zip(atom.query_forms.iter().find(|f| f.kind == "observed").and_then(|f| f.reading.as_ref())).map(|(a,b)| format!("{a}{b}"));
        chain.dictionary_form = format!("{}{}", prefix, atom.dictionary_form);
        chain.lookup_form = chain.dictionary_form.clone(); chain.display_form = chain.dictionary_form.clone();
        chain.lemma_form = format!("{}{}", prefix, atom.lemma_form);
        chain.core_morpheme_indices.extend(&atom.core_morpheme_indices);
        chain.query_forms = atom.query_forms.iter().filter(|f| f.kind != "observed").map(|f| QueryForm {
            kind: f.kind.clone(), form: format!("{prefix}{}", f.form),
            reading: prefix_reading.as_ref().zip(f.reading.as_ref()).map(|(a,b)| format!("{a}{b}")),
            reading_field: Some("composed".into()),
        }).collect();
        chain.query_forms.insert(0, QueryForm { kind: "observed".into(), form: format!("{prefix}{}", atom.surface_form), reading_field: observed_reading.as_ref().map(|_| "composed_kana".into()), reading: observed_reading });
        if !chain.query_forms.iter().any(|f| f.form == chain.dictionary_form) {
            chain.query_forms.push(QueryForm { kind: "base".into(), form: chain.dictionary_form.clone(), reading: None, reading_field: None });
        }
    }
    chain.surface_form.push_str(&atom.surface_form);
    chain.morpheme_range[1] = atom.morpheme_range[1]; chain.char_range[1] = atom.char_range[1];
    chain.source_ranges.extend(atom.source_ranges); chain.morpheme_indices.extend(atom.morpheme_indices);
    chain.operators.extend(atom.operators.into_iter().filter(|o| o.kind != "conjugation")); chain.connection_forms.extend(atom.connection_forms);
    chain.evidence.extend(atom.evidence); chain.final_state = atom.final_state;
}

fn merge_verb_core(chain: &mut MorphologyChain, atom: MorphologyChain, current: &ProviderToken, node: &SourceNode) {
    let lemma = node.features["lemma"].as_str().unwrap();
    let (conjugation_type, conjugation_form) = node.features["morph"]["Inflection"].as_str().unwrap().split_once(';').unwrap();
    let reading = node.features["morph"]["Reading"].as_str().map(str::to_owned);
    let mut supplemented = current.clone();
    supplemented.fields[0].value = Some("動詞".into());
    supplemented.fields[4].value = Some(conjugation_type.into());
    supplemented.fields[5].value = Some(conjugation_form.into());
    let verb_state = state(&supplemented);
    let mut conjugation = operator(chain, &atom, "conjugation".into(), current);
    conjugation.input_state = lemma.into();
    conjugation.output_state = conjugation_form.into();
    conjugation.label = format!("{conjugation_type}/{conjugation_form}");
    conjugation.description = format!("GiNZA 活用字段：{}", conjugation.label);
    conjugation.evidence = vec![format!("ginza:{}:Inflection", node.id)];
    conjugation.source_morpheme_range = [chain.anchor_morpheme, atom.morpheme_range[1]];
    conjugation.char_range = [chain.char_range[0], atom.char_range[1]];
    conjugation.state_before = MorphologyState { category: "動詞".into(), form: ConnectionForm::Terminal,
        conjugation_type: conjugation_type.into(), conjugation_form: "基本形".into() };
    conjugation.state_after = verb_state.clone();
    chain.source_evidence.extend(atom.source_evidence.clone());
    chain.source_evidence.push(crate::linguistic_context::SourceEvidence {
        provider: "ginza".into(), node_id: Some(node.id.clone()), relation_id: None, reason: "verb_core".into(),
    });
    merge(chain, atom, true);
    chain.dictionary_form = lemma.into();
    chain.lemma_form = lemma.into();
    chain.lookup_form = lemma.into();
    chain.display_form = lemma.into();
    chain.final_state = verb_state;
    chain.query_forms = vec![QueryForm { kind: "observed".into(), form: chain.surface_form.clone(),
        reading: reading.clone(), reading_field: reading.as_ref().map(|_| "ginza:morph.Reading".into()) }];
    if lemma != chain.surface_form {
        chain.query_forms.push(QueryForm { kind: "base".into(), form: lemma.into(), reading: None, reading_field: None });
    }
    chain.operators.push(conjugation);
    chain.evidence.push(format!("ginza:{}:verb_core", node.id));
}

fn suffix_sequence(previous: &MorphologyChain, index: usize, source: &[ProviderToken], context: &Context<'_>) -> Option<&'static config::Sequence> {
    CONFIG.sequences.iter().find(|rule| {
        let end = index + rule.tokens.len();
        end <= source.len() && previous.morpheme_range[1] == index && previous.char_range[1] == source[index].char_range[0]
            && rule.forms.contains(&previous.final_state.form)
            && (rule.categories.is_empty() || rule.categories.contains(&previous.final_state.category))
            && (rule.operators.is_empty() || previous.operators.iter().any(|op| rule.operators.contains(&op.kind)))
            && rule.tokens.iter().zip(&source[index..end]).all(|(pattern, token)| pattern.matches(token))
            && source[index..end].windows(2).all(|pair| pair[0].char_range[1] == pair[1].char_range[0])
            // 首项能形成て形时，优先保留经句法确认的助词插入补助连接。
            && !(end > index + 1 && source[index + 1..end].iter().all(insertion_token)
                && matches!(transition(previous, &source[index], context, source),
                    Transition::Attach(kinds) if kinds.iter().any(|kind| kind == "te_connection"))
                && source.get(end).is_some_and(|next| CONFIG.operators.values().any(|entry|
                    entry.support.as_ref().is_some_and(|support| support.bases.iter().any(|base| base == identity(next))))
                    && context.confirms_inserted_support(source[previous.core_morpheme_indices[0]].char_range,
                        source[index].char_range, &source[index + 1..end].iter().map(|token| token.char_range).collect::<Vec<_>>(), next.char_range)))
            && (!rule.terminal_only || source.get(end).is_none_or(|token|
                field(token, 0) == "補助記号" && matches!(field(token, 1), "句点" | "括弧閉")))
    })
}

/// 后项先完成自身活用，再按完整否定证据确认义务或禁止构式。
fn link_predicates(chains: &mut [MorphologyChain], source: &[ProviderToken]) {
    for index in 1..chains.len() {
        let (before, after) = chains.split_at_mut(index);
        let left = &mut before[index - 1];
        let right = &mut after[0];
        if right.parent_chain_id.is_some() || left.char_range[1] != right.char_range[0]
            || left.morpheme_range[1] != right.morpheme_range[0] { continue; }
        for rule in &CONFIG.predicate_links {
            let Some(prefix) = left.operators.iter().find(|op| op.kind == rule.prefix && op.char_range[1] == left.char_range[1]).cloned() else { continue; };
            if !rule.next.matches(&source[right.anchor_morpheme]) { continue; }
            let body_end = if !rule.required_operators.is_empty() {
                let Some(op) = right.operators.iter().find(|op| rule.required_operators.contains(&op.kind)) else { continue; };
                op.source_morpheme_range[1]
            } else { right.anchor_morpheme + 1 };
            let mut construction = operator(left, right, rule.kind.clone(), &source[prefix.source_morpheme_range[0]]);
            construction.source_morpheme_range = [prefix.source_morpheme_range[0], body_end];
            construction.char_range = [prefix.char_range[0], source[body_end - 1].char_range[1]];
            construction.state_before = prefix.state_before;
            construction.state_after = state(&source[body_end - 1]);
            construction.evidence.extend(prefix.evidence);
            construction.evidence.push("connection:predicate_construction".into());
            construction.output_state = format!("{}{}", left.surface_form, right.surface_form);
            left.operators.retain(|op| op.operator_id != prefix.operator_id);
            right.operators.retain(|op| op.source_morpheme_range[1] > body_end
                || config::definition(&op.kind).is_some_and(|entry| entry.output_form.is_some())
                || !config::is_inflection(&op.kind) && !rule.required_operators.contains(&op.kind));
            right.operators.insert(0, construction);
            right.role = MorphologyRole::Functional;
            right.parent_chain_id = Some(left.chain_id.clone());
            break;
        }
    }
}

pub(crate) fn compose(atoms: Vec<MorphologyChain>, source: &[ProviderToken], morphemes: &[MorphemeToken], external: &[SourceArtifact]) -> MorphologyArtifact {
    let context = Context::new(external);
    let mut chains: Vec<MorphologyChain> = Vec::new();
    let mut diagnostics = Vec::new();
    let mut atoms: std::collections::VecDeque<_> = atoms.into();
    while let Some(mut atom) = atoms.pop_front() {
        let index = atom.anchor_morpheme;
        let current = &source[index];
        let sequence = chains.last().and_then(|previous| suffix_sequence(previous, index, source, &context));
        if let Some(rule) = sequence {
            for _ in 1..rule.tokens.len() { merge(&mut atom, atoms.pop_front().expect("连续词尾成员缺失"), false); }
            atom.operators.clear();
            atom.evidence.extend(rule.phenomena.iter().cloned());
        }
        atom.source_evidence = context.evidence(atom.char_range);
        if atom.final_state.conjugation_form.is_empty() {
            if let Some((ctype, cform)) = context.inflection(atom.char_range).and_then(|f| f.split_once(';')) {
                let mut supplemented = current.clone();
                if supplemented.fields.len() >= 6 {
                    supplemented.fields[4].value = Some(ctype.into()); supplemented.fields[5].value = Some(cform.into());
                    atom.final_state = state(&supplemented);
                    atom.evidence.push("ginza:Inflection:missing_unidic_form".into());
                }
            }
        }
        let mut linked = false;
        let previous_index = chains.iter().enumerate().rev()
            .find(|(_, chain)| bridge_support(chain, index, source, &context)).map(|(i, _)| i)
            .or_else(|| chains.len().checked_sub(1)
                .filter(|&i| chains[i].char_range[1] == atom.char_range[0] && chains[i].morpheme_range[1] == index));
        if let Some(previous_index) = previous_index {
            let previous = &mut chains[previous_index];
            let first = previous.core_morpheme_indices[0];
            let verb_core = (previous.morpheme_indices.len() == 1 && field(&source[first], 0) == "名詞"
                && field(current, 0) == "助動詞")
                .then(|| context.verb_core([previous.char_range[0], atom.char_range[1]], &source[first].surface))
                .flatten();
            if let Some(node) = verb_core.filter(|_| sequence.is_none()) {
                merge_verb_core(previous, atom, current, node);
                continue;
            }
            let members = context.core_members(morphemes, first);
            let local_step = sequence.map_or_else(|| transition(previous, current, &context, source),
                |rule| Transition::Attach(vec![rule.kind.clone()]));
            let nominalization = matches!(&local_step, Transition::Attach(kinds) if kinds.iter().any(|kind| kind == "nominalization"));
            let semantic_connection = nominalization || matches!(&local_step, Transition::Support(_));
            let step = if sequence.is_none() && members.contains(&index) && !semantic_connection && !matches!(local_step, Transition::Boundary) { Transition::Core } else { local_step };
            match step {
                Transition::Core => {
                    previous.evidence.push("connection:lexical_core".into());
                    let prior = &source[previous.morpheme_range[1]-1];
                    if field(prior,0)=="動詞" && field(current,0)=="名詞" {
                        previous.status="candidate".into();
                        diagnostics.push(format!("lexical_core_pos_conflict:{}:{index}",previous.chain_id));
                    }
                    previous.source_evidence.extend(atom.source_evidence.clone());
                    for kind in semantic(current) { previous.operators.push(operator(previous, &atom, kind, current)); }
                    merge(previous, atom, true); continue;
                }
                Transition::Attach(mut kinds) => {
                    let before = previous.final_state.clone();
                    for entry in kinds.iter().filter_map(|kind| config::definition(kind)) {
                        if let Some(category) = &entry.output_category { atom.final_state.category = category.clone(); }
                        if let Some(form) = &entry.output_form { atom.final_state.form = form.clone(); }
                        if before.category == "形状詞" {
                            if let Some(suffix) = &entry.display_suffix { previous.display_form = format!("{}{suffix}", previous.dictionary_form); }
                        }
                    }
                    if kinds.is_empty() { kinds.push("auxiliary".into()); }
                    for kind in kinds { previous.operators.push(operator(previous, &atom, kind, current)); }
                    if let Some(e) = context.relation(previous.char_range, atom.char_range, &["aux", "cop", "mark"]) { previous.source_evidence.push(e); }
                    if context.ginza.is_some() && !context.same_bunsetsu(previous.char_range, atom.char_range) {
                        diagnostics.push(format!("cross_bunsetsu:{}:{}", previous.chain_id, index));
                    }
                    previous.source_evidence.extend(atom.source_evidence.clone());
                    merge(previous, atom, false); continue;
                }
                Transition::Support(kind) => {
                    atom.role = MorphologyRole::Functional;
                    atom.parent_chain_id = Some(previous.chain_id.clone());
                    atom.operators.push(operator(previous, &atom, kind.into(), current));
                    let relation = context.relation(previous.char_range, atom.char_range, &["aux", "cop", "advcl", "xcomp"]);
                    let bridged = bridge_support(previous, index, source, &context);
                    let supported = bridged || relation.is_some() || context.same_word(previous.char_range, atom.char_range)
                        || context.same_bunsetsu(previous.char_range, atom.char_range);
                    atom.status = if supported { "resolved" } else { "candidate" }.into();
                    if let Some(e) = relation { atom.source_evidence.push(e); }
                    if let Some(source)=context.ginza {
                        for node in source.nodes.iter().filter(|n| n.kind==crate::external::NodeKind::Bunsetsu && n.text_ranges.len()==1
                            && n.text_ranges[0][0]<=previous.char_range[0] && atom.char_range[1]<=n.text_ranges[0][1]) {
                            atom.source_evidence.push(crate::linguistic_context::SourceEvidence {provider:"ginza".into(),node_id:Some(node.id.clone()),relation_id:None,reason:"support_bunsetsu".into()});
                        }
                    }
                    if let Some([base, reading]) = config::definition(kind).and_then(|entry| entry.support.as_ref())
                        .and_then(|support| support.expanded.as_ref()) {
                        atom.display_form = atom.operators.last().and_then(|op| op.normalized_form.clone()).unwrap();
                        atom.dictionary_form = base.clone(); atom.lookup_form = base.clone();
                        atom.query_forms.push(QueryForm { kind: "expanded_auxiliary".into(), form: base.clone(), reading: Some(reading.clone()), reading_field: Some("contraction_rule".into()) });
                    }
                    if !supported { diagnostics.push(format!("support_ownership_candidate:{}:{}", previous.chain_id, index)); }
                    linked = true;
                }
                Transition::Boundary | Transition::Stop => {
                    if context.relation(previous.char_range, atom.char_range, &["aux", "cop"]).is_some() {
                        diagnostics.push(format!("source_connection_conflict:{}:{}", previous.chain_id, index));
                        atom.status = "candidate".into();
                    }
                }
            }
        }
        for kind in semantic(current) {
            let mut op = operator(&atom, &atom, kind, current);
            op.input_state = atom.dictionary_form.clone(); op.output_state = atom.surface_form.clone();
            atom.operators.push(op);
        }
        if !linked && field(current, 0) == "助動詞" {
            atom.status = "pending".into(); diagnostics.push(format!("unattached_auxiliary:{index}"));
        }
        chains.push(atom);
    }
    link_predicates(&mut chains, source);
    let occurrences = chains.iter().flat_map(|chain| chain.operators.iter().filter(|op|
        !config::is_inflection(&op.kind))
        .map(|op| {
            let mut hit_ranges=vec![op.char_range];
            let mut members:Vec<_>=(op.source_morpheme_range[0]..op.source_morpheme_range[1]).collect();
            let mut operator_ids=vec![op.operator_id.clone()];
            let mut context_range=chain.char_range;
            if let Some(connector_kind) = config::definition(&op.kind).and_then(|entry| entry.connector.as_ref()).filter(|_| chain.parent_chain_id.is_some()) {
                if let Some(parent)=chains.iter().find(|c| Some(&c.chain_id)==chain.parent_chain_id.as_ref()) {
                    if let Some(connector)=parent.operators.iter().find(|o| &o.kind == connector_kind) {
                        hit_ranges.insert(0,connector.char_range);
                        members.splice(0..0,connector.source_morpheme_range[0]..connector.source_morpheme_range[1]);
                        operator_ids.insert(0,connector.operator_id.clone());
                        members.sort_unstable();
                    }
                }
            }
            let mut parent=chain.parent_chain_id.as_ref();
            while let Some(id)=parent {
                let Some(ancestor)=chains.iter().find(|c| &c.chain_id==id) else { break; };
                context_range[0]=ancestor.char_range[0]; parent=ancestor.parent_chain_id.as_ref();
            }
            let char_range = if hit_ranges.len() > 1 { [hit_ranges[0][0], op.char_range[1]] } else { op.char_range };
            MorphologyOccurrence { id: format!("occurrence:{}", op.operator_id), chain_id: chain.chain_id.clone(), operator_ids,
                kind: op.kind.clone(), char_range, context_range, hit_ranges,
                morpheme_indices: members, candidates: op.candidates.clone(), status: chain.status.clone(), source_evidence: chain.source_evidence.clone() }
        })).collect();
    MorphologyArtifact { schema: SCHEMA.into(), chains, occurrences, diagnostics }
}
