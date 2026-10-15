//! 实际 UniDic 活用与 GiNZA 信息变化的定向验收。
use readux_core::analysis::{AnalysisService, Request, ResourcePaths};
use readux_nlp::{model::{Register, UnifiedDocument}, morphology::MorphologyRole};
use std::path::PathBuf;

fn analyze(service: &AnalysisService, text: &str) -> UnifiedDocument {
    let response = service.dispatch(Request::Analyze { text: text.into(), register: Register::Cwj });
    assert!(response.error.is_none(), "{:?}", response.error);
    serde_json::from_value(response.result.unwrap()).unwrap()
}

#[test]
fn modern_inflections_and_functional_ownership() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for (text, surface, base, kinds) in [
        ("行かせられなかった。", "行かせられなかった", "行く", vec!["causative", "passive_potential", "negative", "past"]),
        ("読みませんでした。", "読みませんでした", "読む", vec!["politeness_masu", "negative", "politeness_desu", "past"]),
        ("食べたかった。", "食べたかった", "食べる", vec!["desire", "past"]),
        ("来なかった。", "来なかった", "来る", vec!["negative", "past"]),
        ("成立した。", "成立した", "成立する", vec!["past"]),
        ("高くなかった。", "高くなかった", "高い", vec!["negative", "past"]),
        ("静かな町だ。", "静かな", "静か", vec!["attributive_na"]),
        ("行くな。", "行くな", "行く", vec!["prohibitive"]),
        ("読め。", "読め", "読む", vec!["imperative"]),
        ("読もう。", "読もう", "読む", vec!["volitional"]),
        ("読めば分かる。", "読めば", "読む", vec!["conditional", "ba_connection"]),
        ("発展せしめる。", "発展せしめる", "発展する", vec!["causative"]),
        ("本である。", "本である", "本", vec!["copula", "copula_aru"]),
        ("降るらしい。", "降るらしい", "降る", vec!["rashii_modality"]),
        ("読めそうだ。", "読めそうだ", "読める", vec!["sou_modality", "copula"]),
        ("行くまい。", "行くまい", "行く", vec!["negative_volitional"]),
        ("読みやすい。", "読みやすい", "読みやすい", vec!["ease"]),
    ] {
        let doc = analyze(&service, text);
        let chain = doc.morphology.chains.iter().find(|c| c.surface_form == surface)
            .unwrap_or_else(|| panic!("{text}: {:?}", doc.morphology.chains.iter().map(|c| (&c.surface_form,&c.final_state)).collect::<Vec<_>>()));
        assert_eq!(chain.dictionary_form, base, "{text}");
        for kind in kinds { assert!(chain.operators.iter().any(|o| o.kind == kind), "{text} 缺少 {kind}"); }
        for op in &chain.operators { assert!(readux_core::grammar_catalog::get(&op.concept_id).is_ok(), "目录引用无效：{}",op.concept_id); }
        assert!(chain.query_forms.iter().any(|q| q.form == base), "{text} 缺少基本形查询");
    }
    for text in ["読んでくださった。", "専有してしまっている。", "読んでいる。", "覗かれてる。", "読んじゃった。", "読んどく。"] {
        let doc = analyze(&service, text);
        let functional: Vec<_> = doc.morphology.chains.iter().filter(|c| c.parent_chain_id.is_some()).collect();
        assert!(!functional.is_empty(), "{text}: {:?}", doc.morphology.chains.iter().map(|c| &c.surface_form).collect::<Vec<_>>());
        for chain in functional {
            assert_eq!(chain.role, MorphologyRole::Functional);
            assert!(doc.morphology.chains.iter().any(|p| Some(&p.chain_id) == chain.parent_chain_id.as_ref()));
        }
    }
    for text in ["そこにいる。", "読んで いる。", "読んで。いる。"] {
        let doc = analyze(&service, text);
        assert!(doc.morphology.chains.iter().all(|c| c.parent_chain_id.is_none()), "{text}");
    }
    for (text,base,expanded) in [("覗かれてる。","いる","ている"),("読んでる。","いる","ている"),
        ("食べちゃった。","しまう","てしまう"),("読んじゃった。","しまう","てしまう"),
        ("食べとく。","おく","ておく"),("読んどく。","おく","ておく")] {
        let doc=analyze(&service,text);
        let chain=doc.morphology.chains.iter().find(|c| c.parent_chain_id.is_some()).unwrap();
        assert_eq!(chain.lookup_form,base);
        assert!(chain.query_forms.iter().any(|q| q.kind=="expanded_auxiliary" && q.form==base));
        assert_eq!(chain.display_form, expanded, "{text}");
        assert!(chain.operators.iter().any(|o| o.normalized_form.as_deref()==Some(expanded)), "{text}");
    }
    let doc=analyze(&service,"読んでいる。");
    let occurrence=doc.morphology.occurrences.iter().find(|o| o.kind=="te_iru").unwrap();
    assert_eq!(occurrence.char_range,[2,5]);
    assert_eq!(occurrence.hit_ranges,vec![[2,3],[3,5]]);
}

#[test]
fn yuku_forms_a_support_chain_after_te() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for text in ["散布されてゆく。", "代わられてゆく。", "広がってゆきました。", "読んでゆく。"] {
        let doc = analyze(&service, text);
        let chain = doc.morphology.chains.iter().find(|chain| chain.operators.iter().any(|op| op.kind == "te_yuku")).unwrap();
        let parent = doc.morphology.chains.iter().find(|parent| Some(&parent.chain_id) == chain.parent_chain_id.as_ref()).unwrap();
        assert_eq!(chain.role, MorphologyRole::Functional);
        assert_eq!(chain.lookup_form, "ゆく");
        assert_eq!(parent.char_range[1], chain.char_range[0]);
        let occurrence = doc.morphology.occurrences.iter().find(|occurrence| occurrence.kind == "te_yuku").unwrap();
        assert_eq!(occurrence.hit_ranges.len(), 2);
        assert_eq!(occurrence.context_range[0], parent.char_range[0]);
        let op = chain.operators.iter().find(|op| op.kind == "te_yuku").unwrap();
        assert_eq!(op.normalized_form.as_deref(), Some("てゆく"));
        assert!(readux_core::grammar_catalog::get(&op.concept_id).is_ok());
    }
    for text in ["学校へゆく。", "散布されて、ゆく。", "散布されて ゆく。"] {
        let doc = analyze(&service, text);
        assert!(doc.morphology.chains.iter().all(|chain| !chain.operators.iter().any(|op| op.kind == "te_yuku")), "{text}");
    }
}

#[test]
fn tara_and_dara_are_complete_conditional_forms() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for (text, ending) in [("なれたら。", "たら"), ("食べたら。", "たら"), ("読んだら。", "だら"),
        ("静かだったら。", "たら"), ("なれたらいいな。", "たら")] {
        let doc = analyze(&service, text);
        let occurrence = doc.morphology.occurrences.iter().find(|occurrence| occurrence.kind == "tara_condition").unwrap();
        let surface: String = doc.text.chars().skip(occurrence.char_range[0]).take(occurrence.char_range[1] - occurrence.char_range[0]).collect();
        assert_eq!(surface, ending);
        assert_eq!(occurrence.hit_ranges, vec![occurrence.char_range]);
        let operators = doc.morphology.chains.iter().flat_map(|chain| &chain.operators)
            .filter(|op| op.char_range == occurrence.char_range).collect::<Vec<_>>();
        assert_eq!(operators.len(), 1, "{text}");
        assert_eq!(operators[0].normalized_form.as_deref(), Some(ending));
    }
    for (text, kind) in [("読んだ。", "past"), ("読めば。", "ba_connection")] {
        let doc = analyze(&service, text);
        assert!(doc.morphology.occurrences.iter().any(|occurrence| occurrence.kind == kind));
        assert!(!doc.morphology.occurrences.iter().any(|occurrence| occurrence.kind == "tara_condition"));
    }
}

#[test]
fn sentence_final_no_is_outside_the_predicate_chain() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for text in ["なにそれ食べられるの？", "食べられるの", "読んだん？", "読んだの。", "「食べるの？」", "食べるのね。"] {
        let doc = analyze(&service, text);
        let index = doc.morphemes.iter().position(|token| token.lemma.as_deref() == Some("の")).unwrap();
        let chain = doc.morphology.chains.iter().find(|chain| chain.morpheme_indices.contains(&index)).unwrap();
        assert_eq!(chain.morpheme_indices, vec![index], "{text}");
        assert!(chain.parent_chain_id.is_none());
        assert!(chain.operators.is_empty());
        assert!(!doc.morphology.occurrences.iter().any(|occurrence| occurrence.morpheme_indices.contains(&index)));
        assert!(doc.grammar.occurrences.iter().any(|occurrence| occurrence.morpheme_indices.contains(&index)));
    }
    for text in ["読んだのだ。", "読んだんです。", "読んだのが好き。", "読むのをやめた。"] {
        let doc = analyze(&service, text);
        assert!(doc.morphology.chains.iter().any(|chain| chain.surface_form.starts_with("読")
            && chain.operators.iter().any(|op| op.kind == "nominalization")), "{text}");
    }
}

#[test]
fn ni_na_and_da_preserve_their_distinct_functions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for (text, surface, kind, form, display) in [
        ("夢みたいに見える。", "に", "adverbial_ni", "に", "みたいに"),
        ("夢みたいな話。", "な", "attributive_na", "な", "みたいな"),
        ("夢みたいだ。", "だ", "copula", "だ", "みたいだ"),
        ("鳥のように飛ぶ。", "に", "adverbial_ni", "に", "ように"),
        ("鳥のような姿。", "な", "attributive_na", "な", "ような"),
        ("鳥のようだ。", "だ", "copula", "だ", "ようだ"),
        ("静かに歩く。", "に", "adverbial_ni", "に", "静かに"),
        ("静かな町。", "な", "attributive_na", "な", "静かな"),
        ("静かだ。", "だ", "copula", "だ", "静かだ"),
    ] {
        let doc = analyze(&service, text);
        let token = doc.morphemes.iter().find(|token| token.surface == surface).unwrap();
        let chain = doc.morphology.chains.iter().find(|chain| chain.operators.iter().any(|op| op.kind == kind && op.char_range == token.char_range)).unwrap();
        let operators = chain.operators.iter().filter(|op| op.char_range == token.char_range).collect::<Vec<_>>();
        assert_eq!(operators.len(), 1, "{text}");
        assert_eq!(operators[0].normalized_form.as_deref(), Some(form));
        assert!(readux_core::grammar_catalog::get(&operators[0].concept_id).is_ok());
        assert!(chain.display_form.ends_with(display), "{text}: {}", chain.display_form);
        assert_eq!(doc.morphology.occurrences.iter().filter(|occurrence| occurrence.char_range == token.char_range).count(), 1);
    }
    let doc = analyze(&service, "駅に行くな。");
    assert!(!doc.morphology.occurrences.iter().any(|occurrence| matches!(occurrence.kind.as_str(), "adverbial_ni" | "attributive_na")));
    assert!(doc.morphology.occurrences.iter().any(|occurrence| occurrence.kind == "prohibitive"));
}

#[test]
fn ginza_changes_core_boundaries_and_functional_status() {
    use readux_nlp::{external::{SourceArtifact, Endpoint, RelationKind, SourceRelation}, syntax::*};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    let doc = analyze(&service, "読んでいる。");
    let specs = [("t0", [0,2], "VERB"), ("t1", [2,3], "SCONJ"), ("t2", [3,5], "AUX")];
    let syntax = SyntaxArtifact { schema: SCHEMA.into(), segment_id: None, text_characters: doc.characters,
        text_sha256: readux_nlp::external::text_digest(&doc.text), provider: SyntaxProviderDescriptor { id: "ginza".into(), version: None, capabilities: Vec::new(), license: None },
        spans: specs.iter().map(|(id, range, pos)| SyntaxSpan { id: (*id).into(), kind: "token".into(), char_range: *range,
            head_char_range: None, source_id: "fixture".into(), surface: None, labels: vec![format!("pos:{pos}")] }).collect() };
    let mut source = SourceArtifact::from_syntax(&syntax, &doc.text).unwrap();
    source.relations.push(SourceRelation { id: "aux2".into(), kind: RelationKind::Dependency, source: Endpoint::Node { id: "t2".into() },
        target: Endpoint::Node { id: "t0".into() }, label: "aux".into(), features: serde_json::json!({}) });
    let with = readux_nlp::morphology::collect_with_external(&doc.source.tokens, &doc.morphemes, &[source.clone()]).unwrap();
    assert_eq!(with.chains.iter().find(|c| c.surface_form=="いる").unwrap().status, "resolved");
    source.relations.clear();
    let without = readux_nlp::morphology::collect_with_external(&doc.source.tokens, &doc.morphemes, &[source]).unwrap();
    assert_eq!(without.chains.iter().find(|c| c.surface_form=="いる").unwrap().status, "candidate");

    let doc = analyze(&service, "情報処理。");
    let mut syntax = syntax; syntax.text_characters=doc.characters; syntax.text_sha256=readux_nlp::external::text_digest(&doc.text);
    syntax.spans=vec![SyntaxSpan { id: "whole".into(), kind:"token".into(), char_range:[0,4], head_char_range:None, source_id:"fixture".into(),surface:None,labels:vec!["pos:NOUN".into()] }];
    let source=SourceArtifact::from_syntax(&syntax,&doc.text).unwrap();
    let with=readux_nlp::morphology::collect_with_external(&doc.source.tokens,&doc.morphemes,&[source]).unwrap();
    assert!(with.chains.iter().any(|c| c.surface_form=="情報処理" && c.core_morpheme_indices.len()==2));
    assert!(doc.morphology.chains.iter().all(|c| c.core_morpheme_indices.len()==1));
}

#[test]
fn formation_uses_dependencies_and_preserves_partial_coverage() {
    use readux_nlp::{external::*, formation::*, syntax::*};
    let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service=AnalysisService::new(ResourcePaths::development(&root));
    let doc=analyze(&service,"情報処理。");
    let mut syntax=SyntaxArtifact { schema: readux_nlp::syntax::SCHEMA.into(),segment_id:None,
        provider:SyntaxProviderDescriptor {id:"ginza".into(),version:None,capabilities:vec![],license:None},
        text_sha256:text_digest(&doc.text),text_characters:doc.characters,spans:vec![] };
    for (id,range) in [("a",[0,2]),("b",[2,4])] {
        syntax.spans.push(SyntaxSpan {id:id.into(),kind:"token".into(),char_range:range,head_char_range:None,source_id:"fixture".into(),surface:None,labels:vec!["pos:NOUN".into()]});
    }
    let mut source=SourceArtifact::from_syntax(&syntax,&doc.text).unwrap();
    source.relations.push(SourceRelation {id:"compound-a-b".into(),kind:RelationKind::Dependency,source:Endpoint::Node {id:"a".into()},target:Endpoint::Node {id:"b".into()},label:"compound".into(),features:serde_json::json!({})});
    let mut artifact=collect_formations_with_sources(&doc.text,&doc.morphemes,&doc.structure,&[source]).unwrap();
    attach_words(&doc.text,&doc.morphemes,&doc.morphology,&mut artifact);
    let word=artifact.nodes.iter().find(|n| n.char_range==[0,4]).unwrap().word.as_ref().unwrap();
    assert_eq!(word.source_relation_ids,vec!["compound-a-b"]);
    assert_eq!(word.head_morpheme,Some(1));
    assert_eq!(word.component_candidate_ids.len(),2);
    assert_eq!(word.query_forms[0].reading.as_deref(),Some("ジョウホウショリ"));
    let mut uncertain=doc.morphology.clone();
    uncertain.chains[0].status="candidate".into();
    attach_words(&doc.text,&doc.morphemes,&uncertain,&mut artifact);
    assert_eq!(artifact.nodes[0].status,FormationStatus::Candidate);
    assert_eq!(artifact.nodes[0].word.as_ref().unwrap().reason,"morphology_core_candidate");
    let mut tokens=doc.morphemes.clone();
    tokens[1].query_forms.push(readux_nlp::model::QueryForm {kind:"base".into(),form:"処理する".into(),reading:None,reading_field:None});
    attach_words(&doc.text,&tokens,&doc.morphology,&mut artifact);
    let restored=artifact.nodes[0].word.as_ref().unwrap().query_forms.iter().find(|q| q.form=="情報処理する").unwrap();
    assert!(restored.reading.is_none(),"基本形缺少读音时应保留缺失状态");
    syntax.spans[0].char_range=[0,1]; syntax.spans[0].kind="compound".into(); syntax.spans.truncate(1);
    let (structure,_)=readux_nlp::structure::merge_external(readux_nlp::structure::local_candidates(&doc.text),&syntax,&doc.text).unwrap();
    let mut partial=collect_formations(&doc.text,&doc.morphemes,&structure).unwrap();
    attach_words(&doc.text,&doc.morphemes,&doc.morphology,&mut partial);
    assert_eq!(partial.nodes[0].status,FormationStatus::Pending);
    assert!(partial.nodes[0].word.as_ref().unwrap().query_forms[0].reading.is_none());
}
