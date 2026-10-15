use readux_nlp::{
    external::{Endpoint, Manifest, NodeKind, RelationKind, SourceArtifact, SourceNode, SourceRelation},
    model::{FeatureField, MorphemeToken, ProviderToken, QueryForm, FIELD_LABELS, FIELD_NAMES},
    morphology::MorphologyRole,
};
use serde_json::json;

#[derive(Clone)]
struct Part { surface: &'static str, pos: &'static str, sub: &'static str, ctype: &'static str, cform: &'static str, lemma: &'static str, orth_base: &'static str }

fn parts(text: &str, specs: &[Part]) -> (Vec<ProviderToken>, Vec<MorphemeToken>) {
    let mut source = Vec::new();
    let mut morphemes = Vec::new();
    let mut cursor = 0;
    for (index, spec) in specs.iter().enumerate() {
        let start = cursor;
        cursor += spec.surface.chars().count();
        let mut values = vec![None; FIELD_NAMES.len()];
        for (name, value) in [
            ("pos1", spec.pos), ("pos2", spec.sub), ("cType", spec.ctype), ("cForm", spec.cform),
            ("lemma", spec.lemma), ("orth", spec.surface), ("orthBase", spec.orth_base),
        ] {
            values[FIELD_NAMES.iter().position(|item| *item == name).unwrap()] = Some(value.to_owned());
        }
        let fields = values.into_iter().enumerate().map(|(field, value)| FeatureField {
            index: field, name: FIELD_NAMES[field].into(), label: FIELD_LABELS[field].into(),
            raw: value.clone(), value,
        }).collect();
        source.push(ProviderToken { index, surface: spec.surface.into(), char_range: [start, cursor], byte_range: [0, 0],
            lexicon_type: "known".into(), left_id: 0, right_id: 0, word_cost: 0, total_cost: 0,
            raw_feature: String::new(), fields });
        morphemes.push(MorphemeToken { id: format!("m{index}"), source_index: index, surface: spec.surface.into(),
            char_range: [start, cursor], pos: [Some(spec.pos.into()), Some(spec.sub.into()), None, None],
            lemma: Some(spec.lemma.into()), reading: None,
            query_forms: vec![QueryForm { kind: "base".into(), form: spec.lemma.into(), reading: None, reading_field: None }] });
    }
    assert_eq!(cursor, text.chars().count());
    (source, morphemes)
}

fn ginza(text: &str, source: &[ProviderToken], inserted: &[usize]) -> SourceArtifact {
    let nodes = source.iter().enumerate().map(|(index, token)| SourceNode {
        id: format!("t{index}"), kind: NodeKind::Token, source_ranges: vec![token.char_range],
        source_surface: token.surface.clone(), text_ranges: vec![token.char_range], surface: token.surface.clone(),
        head: None, members: Vec::new(), features: json!({"pos": if token.fields[0].value.as_deref() == Some("助詞") { "SCONJ" } else { "VERB" }}),
    }).collect::<Vec<_>>();
    let mut relations = vec![
        SourceRelation { id: "mark".into(), kind: RelationKind::Dependency, source: Endpoint::Node { id: "t1".into() },
            target: Endpoint::Node { id: "t0".into() }, label: "mark".into(), features: json!({}) },
        SourceRelation { id: "support".into(), kind: RelationKind::Dependency, source: Endpoint::Node { id: format!("t{}", source.len() - 1) },
            target: Endpoint::Node { id: "t0".into() }, label: "advcl".into(), features: json!({}) },
    ];
    for (offset, index) in inserted.iter().enumerate() {
        relations.push(SourceRelation { id: format!("inserted{offset}"), kind: RelationKind::Dependency,
            source: Endpoint::Node { id: format!("t{index}") }, target: Endpoint::Node { id: "t0".into() },
            label: if source[*index].surface == "さえ" { "mark" } else { "case" }.into(), features: json!({}) });
    }
    SourceArtifact {
        schema: "readux.source-analysis.v1".into(),
        provider: Manifest { id: "ginza".into(), version: "test".into(), model: "test".into(), model_version: "test".into(),
            versions: Default::default(), tasks: vec!["dependency".into()], capabilities: vec!["dependency".into()],
            coordinate_system: "unicode_scalar".into(), resources: Vec::new(), resource_digest: String::new(),
            execution: Default::default() },
        text_sha256: readux_nlp::external::text_digest(text), text_characters: text.chars().count(),
        normalized_text: text.into(), normalization_map: text.chars().enumerate().map(|(i, _)| [i, i + 1]).collect(),
        deleted_ranges: Vec::new(), nodes, relations, diagnostics: Vec::new(), raw: None, elapsed_ms: 0.0,
    }
}

fn run(text: &str, specs: &[Part], inserted: &[usize]) -> readux_nlp::morphology::MorphologyArtifact {
    let (source, morphemes) = parts(text, specs);
    let external = if inserted.is_empty() { Vec::new() } else { vec![ginza(text, &source, inserted)] };
    readux_nlp::morphology::collect_with_external(&source, &morphemes, &external).unwrap()
}

fn verb(surface: &'static str, cform: &'static str) -> Part {
    Part { surface, pos: "動詞", sub: "自立", ctype: "五段-マ行", cform, lemma: "読む", orth_base: "読む" }
}

fn te() -> Part { Part { surface: "で", pos: "助詞", sub: "接続助詞", ctype: "", cform: "", lemma: "で", orth_base: "で" } }
fn aux(surface: &'static str, lemma: &'static str, orth_base: &'static str) -> Part {
    Part { surface, pos: "動詞", sub: "非自立可能", ctype: "上一段-ア行", cform: "終止形-一般", lemma, orth_base }
}

#[test]
fn explanatory_no_joins_the_predicate_chain() {
    for (surface, sub) in [("ん", "準体助詞"), ("の", "準体助詞"), ("ん", "終助詞")] {
        let specs = [
            Part { surface: "し", pos: "動詞", sub: "非自立可能", ctype: "サ行変格", cform: "連用形-一般", lemma: "為る", orth_base: "する" },
            Part { surface: "た", pos: "助動詞", sub: "", ctype: "助動詞-タ", cform: "連体形-一般", lemma: "た", orth_base: "た" },
            Part { surface, pos: "助詞", sub, ctype: "", cform: "", lemma: "の", orth_base: surface },
            Part { surface: "だ", pos: "助動詞", sub: "", ctype: "助動詞-ダ", cform: "終止形-一般", lemma: "だ", orth_base: "だ" },
        ];
        let text = format!("した{surface}だ");
        let artifact = run(&text, &specs, &[]);
        assert_eq!(artifact.chains.len(), 1, "{text}");
        let chain = &artifact.chains[0];
        assert_eq!(chain.surface_form, text);
        assert_eq!(chain.lookup_form, "する");
        assert_eq!(chain.core_morpheme_indices, vec![0]);
        assert!(chain.operators.iter().any(|operator| operator.kind == "nominalization"));
    }
    let specs = [verb("読む", "連体形-一般"),
        Part { surface: "の", pos: "助詞", sub: "格助詞", ctype: "", cform: "", lemma: "の", orth_base: "の" }];
    assert_eq!(run("読むの", &specs, &[]).chains.len(), 2);
}

#[test]
fn synthetic_reconstruction_covers_thirty_contract_cases() {
    let ordinary = [
        ("読んだ", vec![verb("読ん", "連用形-撥音便"), Part { surface: "だ", pos: "助動詞", sub: "", ctype: "助動詞-タ", cform: "基本形", lemma: "だ", orth_base: "だ" }], "past"),
        ("読まない", vec![verb("読ま", "未然形-一般"), Part { surface: "ない", pos: "助動詞", sub: "", ctype: "助動詞-ナイ", cform: "基本形", lemma: "ない", orth_base: "ない" }], "negative"),
        ("読ませる", vec![verb("読ま", "未然形-一般"), Part { surface: "せる", pos: "助動詞", sub: "", ctype: "助動詞-セル", cform: "基本形", lemma: "せる", orth_base: "せる" }], "causative"),
        ("読まれる", vec![verb("読ま", "未然形-一般"), Part { surface: "れる", pos: "助動詞", sub: "", ctype: "助動詞-レル", cform: "基本形", lemma: "れる", orth_base: "れる" }], "passive_potential"),
        ("読みます", vec![verb("読み", "連用形-一般"), Part { surface: "ます", pos: "助動詞", sub: "", ctype: "助動詞-マス", cform: "基本形", lemma: "ます", orth_base: "ます" }], "politeness_masu"),
        ("読みたい", vec![verb("読み", "連用形-一般"), Part { surface: "たい", pos: "助動詞", sub: "", ctype: "助動詞-タイ", cform: "基本形", lemma: "たい", orth_base: "たい" }], "desire"),
        ("読めば", vec![verb("読め", "仮定形-一般"), Part { surface: "ば", pos: "助詞", sub: "接続助詞", ctype: "", cform: "", lemma: "ば", orth_base: "ば" }], "ba_connection"),
        ("読もう", vec![verb("読もう", "意志推量形"),], "volitional"),
        ("読め", vec![verb("読め", "命令形"),], "imperative"),
        ("行くな", vec![Part { surface: "行く", pos: "動詞", sub: "自立", ctype: "五段-カ行促音便", cform: "終止形-一般", lemma: "行く", orth_base: "行く" },
            Part { surface: "な", pos: "助詞", sub: "終助詞", ctype: "", cform: "", lemma: "な", orth_base: "な" }], "prohibitive"),
    ];
    for (text, specs, kind) in ordinary {
        let artifact = run(text, &specs, &[]);
        assert!(artifact.chains.iter().any(|chain| chain.operators.iter().any(|operator| operator.kind == kind)), "{text}: {kind}");
    }

    let compounds = [
        ("読み始める", "始める", "inceptive"), ("読み続ける", "続ける", "continuative_aspect"),
        ("読み終わる", "終わる", "terminative"), ("読みまくる", "まくる", "iterative_aspect"), ("読み切る", "切る", "completion"),
    ];
    for (text, surface, kind) in compounds {
        let specs = [verb("読み", "連用形-一般"), Part { surface, pos: "助動詞", sub: "非自立可能", ctype: "", cform: "基本形", lemma: surface, orth_base: surface }];
        let artifact = run(text, &specs, &[]);
        assert!(artifact.chains.iter().any(|chain| chain.role == MorphologyRole::Functional && chain.operators.iter().any(|operator| operator.kind == kind)), "{text}: {kind}");
    }

    let auxiliaries = [
        ("読んでおく", "おく", "置く", "te_oku"), ("読んでいる", "いる", "居る", "te_iru"),
        ("読んでしまう", "しまう", "仕舞う", "te_shimau"), ("読んでみる", "みる", "見る", "te_miru"),
        ("読んでくださる", "くださる", "下さる", "te_kudasaru"),
    ];
    for (text, surface, lemma, kind) in auxiliaries {
        let specs = [verb("読ん", "連用形-撥音便"), te(), aux(surface, lemma, lemma)];
        let artifact = run(text, &specs, &[]);
        let chain = artifact.chains.iter().find(|chain| chain.role == MorphologyRole::Functional).unwrap();
        assert!(chain.operators.iter().any(|operator| operator.kind == kind), "{text}: {:?} {:?}", chain.surface_form, chain.operators.iter().map(|operator| &operator.kind).collect::<Vec<_>>());
        let occurrence = artifact.occurrences.iter().find(|occurrence| occurrence.kind == kind).unwrap();
        assert_eq!(occurrence.hit_ranges.len(), 2);
        assert_eq!(occurrence.char_range[0], 2);
    }

    let inserted = [
        ("読んではおく", "は", 2usize, "おく", "置く", "te_oku"),
        ("読んでもおく", "も", 2, "おく", "置く", "te_oku"),
        ("読んでさえおく", "さえ", 2, "おく", "置く", "te_oku"),
        ("読んではいる", "は", 2, "いる", "居る", "te_iru"),
        ("読んでもみる", "も", 2, "みる", "見る", "te_miru"),
    ];
    for (text, particle, _, surface, lemma, kind) in inserted {
        let particle = Part { surface: particle, pos: "助詞", sub: if particle == "さえ" { "副助詞" } else { "係助詞" },
            ctype: "", cform: "", lemma: particle, orth_base: particle };
        let specs = [verb("読ん", "連用形-撥音便"), te(), particle, aux(surface, lemma, lemma)];
        let artifact = run(text, &specs, &[2]);
        let chain = artifact.chains.iter().find(|chain| chain.role == MorphologyRole::Functional && chain.parent_chain_id.is_some()).unwrap();
        assert_eq!(chain.status, "resolved", "{text} should use the verified GiNZA bridge: {:?}", artifact.diagnostics);
        assert!(chain.operators.iter().any(|operator| operator.kind == kind), "{text}: {} {:?} {:?}", chain.status, chain.surface_form, chain.operators.iter().map(|operator| &operator.kind).collect::<Vec<_>>());
    }

    let lemma_mappings = [
        ("読んで置く", "置く", "te_oku"), ("読んで居る", "居る", "te_iru"),
        ("読んで仕舞う", "仕舞う", "te_shimau"), ("読んで見る", "見る", "te_miru"),
        ("読んで下さる", "下さる", "te_kudasaru"),
    ];
    for (text, lemma, kind) in lemma_mappings {
        let specs = [verb("読ん", "連用形-撥音便"), te(), aux(lemma, lemma, lemma)];
        let artifact = run(text, &specs, &[]);
        let chain = artifact.chains.iter().find(|chain| chain.role == MorphologyRole::Functional && chain.parent_chain_id.is_some()).unwrap();
        assert_eq!(chain.lookup_form, lemma, "{text}");
        assert!(chain.operators.iter().any(|operator| operator.kind == kind), "{text}");
    }
}
