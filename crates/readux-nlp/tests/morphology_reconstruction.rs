use readux_nlp::{
    external::{
        Endpoint, Manifest, NodeKind, RelationKind, SourceArtifact, SourceNode, SourceRelation,
    },
    model::{FeatureField, MorphemeToken, ProviderToken, QueryForm, FIELD_LABELS, FIELD_NAMES},
    morphology::MorphologyRole,
};
use serde_json::json;

#[derive(Clone)]
struct Part {
    surface: &'static str,
    pos: &'static str,
    sub: &'static str,
    ctype: &'static str,
    cform: &'static str,
    lemma: &'static str,
    orth_base: &'static str,
}

#[test]
fn enumerative_toka_accepts_single_and_split_tokens_without_changing_state() {
    for inserted in [
        vec![focus_particle("とか", "副助詞")],
        vec![
            focus_particle("と", "格助詞"),
            focus_particle("か", "副助詞"),
        ],
    ] {
        let mut specs = vec![
            verb("読ん", "連用形-撥音便"),
            focus_particle("だり", "副助詞"),
        ];
        specs.extend(inserted);
        specs.push(Part {
            surface: "し",
            pos: "動詞",
            sub: "非自立可能",
            ctype: "サ行変格",
            cform: "未然形-一般",
            lemma: "為る",
            orth_base: "する",
        });
        specs.push(Part {
            surface: "ない",
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-ナイ",
            cform: "終止形-一般",
            lemma: "ない",
            orth_base: "ない",
        });
        let text: String = specs.iter().map(|part| part.surface).collect();
        let artifact = run(&text, &specs, &[]);
        assert!(
            artifact.diagnostics.is_empty(),
            "{:?}",
            artifact.diagnostics
        );
        assert_eq!(artifact.chains.len(), 2);
        assert!(artifact
            .chains
            .iter()
            .all(|chain| chain.status == "resolved"));
        let insertion = artifact.chains[0]
            .operators
            .iter()
            .find(|op| op.kind == "exemplification")
            .unwrap();
        assert_eq!(insertion.state_before, insertion.state_after);
        assert_eq!(artifact.chains[0].surface_form, "読んだりとか");
        assert_eq!(
            artifact.chains[1].parent_chain_id.as_deref(),
            Some("morphology:m0")
        );
        let occurrence = artifact
            .occurrences
            .iter()
            .find(|op| op.kind == "tari_suru")
            .unwrap();
        assert_eq!(occurrence.hit_ranges, vec![[2, 4], [4, 6], [6, 7]]);
        assert_eq!(occurrence.context_range, [0, 9]);
    }
}

#[test]
fn local_input_constraints_reject_invalid_reinflection_with_source_edges() {
    for (tail, lemma, ctype, kind) in [
        ("たい", "たい", "助動詞-タイ", "desire"),
        ("ます", "ます", "助動詞-マス", "politeness_masu"),
        ("せる", "せる", "助動詞-セル", "causative"),
    ] {
        let polite = Part {
            surface: if kind == "causative" {
                "ませ"
            } else {
                "まし"
            },
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-マス",
            cform: if kind == "causative" {
                "未然形-一般"
            } else {
                "連用形-一般"
            },
            lemma: "ます",
            orth_base: "ます",
        };
        let specs = [
            verb("読み", "連用形-一般"),
            polite,
            Part {
                surface: tail,
                pos: "助動詞",
                sub: "",
                ctype,
                cform: "終止形-一般",
                lemma,
                orth_base: lemma,
            },
        ];
        let text: String = specs.iter().map(|part| part.surface).collect();
        let artifact = run_with_support_edges(&text, &specs, &[(2, 0)]);
        let owner = artifact
            .chains
            .iter()
            .find(|chain| chain.operators.iter().any(|op| op.kind == kind))
            .unwrap();
        assert_eq!(
            owner.status, "candidate",
            "{text}: {:?}",
            artifact.diagnostics
        );
        assert!(
            artifact
                .diagnostics
                .iter()
                .any(|item| item.contains("input_category")),
            "{text}: {:?}",
            artifact.diagnostics
        );
    }
}

fn parts(text: &str, specs: &[Part]) -> (Vec<ProviderToken>, Vec<MorphemeToken>) {
    let mut source = Vec::new();
    let mut morphemes = Vec::new();
    let mut cursor = 0;
    for (index, spec) in specs.iter().enumerate() {
        let start = cursor;
        cursor += spec.surface.chars().count();
        let mut values = vec![None; FIELD_NAMES.len()];
        for (name, value) in [
            ("pos1", spec.pos),
            ("pos2", spec.sub),
            ("cType", spec.ctype),
            ("cForm", spec.cform),
            ("lemma", spec.lemma),
            ("orth", spec.surface),
            ("orthBase", spec.orth_base),
        ] {
            values[FIELD_NAMES.iter().position(|item| *item == name).unwrap()] =
                Some(value.to_owned());
        }
        let fields = values
            .into_iter()
            .enumerate()
            .map(|(field, value)| FeatureField {
                index: field,
                name: FIELD_NAMES[field].into(),
                label: FIELD_LABELS[field].into(),
                raw: value.clone(),
                value,
            })
            .collect();
        source.push(ProviderToken {
            index,
            surface: spec.surface.into(),
            char_range: [start, cursor],
            byte_range: [0, 0],
            lexicon_type: "known".into(),
            left_id: 0,
            right_id: 0,
            word_cost: 0,
            total_cost: 0,
            raw_feature: String::new(),
            fields,
        });
        morphemes.push(MorphemeToken {
            id: format!("m{index}"),
            source_index: index,
            surface: spec.surface.into(),
            char_range: [start, cursor],
            pos: [Some(spec.pos.into()), Some(spec.sub.into()), None, None],
            lemma: Some(spec.lemma.into()),
            reading: None,
            query_forms: vec![QueryForm {
                kind: "base".into(),
                form: spec.lemma.into(),
                reading: None,
                reading_field: None,
            }],
        });
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
        SourceRelation {
            id: "mark".into(),
            kind: RelationKind::Dependency,
            source: Endpoint::Node { id: "t1".into() },
            target: Endpoint::Node { id: "t0".into() },
            label: "mark".into(),
            features: json!({}),
        },
        SourceRelation {
            id: "support".into(),
            kind: RelationKind::Dependency,
            source: Endpoint::Node {
                id: format!("t{}", source.len() - 1),
            },
            target: Endpoint::Node { id: "t0".into() },
            label: "advcl".into(),
            features: json!({}),
        },
    ];
    for (offset, index) in inserted.iter().enumerate() {
        relations.push(SourceRelation {
            id: format!("inserted{offset}"),
            kind: RelationKind::Dependency,
            source: Endpoint::Node {
                id: format!("t{index}"),
            },
            target: Endpoint::Node { id: "t0".into() },
            label: if source[*index].surface == "さえ" {
                "mark"
            } else {
                "case"
            }
            .into(),
            features: json!({}),
        });
    }
    SourceArtifact {
        schema: "readux.source-analysis.v1".into(),
        provider: Manifest {
            id: "ginza".into(),
            version: "test".into(),
            model: "test".into(),
            model_version: "test".into(),
            versions: Default::default(),
            tasks: vec!["dependency".into()],
            capabilities: vec!["dependency".into()],
            coordinate_system: "unicode_scalar".into(),
            resources: Vec::new(),
            resource_digest: String::new(),
            execution: Default::default(),
        },
        text_sha256: readux_nlp::external::text_digest(text),
        text_characters: text.chars().count(),
        normalized_text: text.into(),
        normalization_map: text.chars().enumerate().map(|(i, _)| [i, i + 1]).collect(),
        deleted_ranges: Vec::new(),
        nodes,
        relations,
        diagnostics: Vec::new(),
        raw: None,
        elapsed_ms: 0.0,
    }
}

fn run(
    text: &str,
    specs: &[Part],
    inserted: &[usize],
) -> readux_nlp::morphology::MorphologyArtifact {
    let (source, morphemes) = parts(text, specs);
    let external = if inserted.is_empty() {
        Vec::new()
    } else {
        vec![ginza(text, &source, inserted)]
    };
    readux_nlp::morphology::collect_with_external(&source, &morphemes, &external).unwrap()
}

fn run_with_support_edges(
    text: &str,
    specs: &[Part],
    support_edges: &[(usize, usize)],
) -> readux_nlp::morphology::MorphologyArtifact {
    let (source, morphemes) = parts(text, specs);
    let mut external = ginza(text, &source, &[]);
    external
        .relations
        .retain(|relation| relation.id != "support");
    for (offset, (support, parent)) in support_edges.iter().enumerate() {
        external.relations.push(SourceRelation {
            id: format!("support{offset}"),
            kind: RelationKind::Dependency,
            source: Endpoint::Node {
                id: format!("t{support}"),
            },
            target: Endpoint::Node {
                id: format!("t{parent}"),
            },
            label: "advcl".into(),
            features: json!({}),
        });
    }
    readux_nlp::morphology::collect_with_external(&source, &morphemes, &[external]).unwrap()
}

fn verb(surface: &'static str, cform: &'static str) -> Part {
    Part {
        surface,
        pos: "動詞",
        sub: "自立",
        ctype: "五段-マ行",
        cform,
        lemma: "読む",
        orth_base: "読む",
    }
}

fn te() -> Part {
    te_token("で")
}

fn te_token(surface: &'static str) -> Part {
    Part {
        surface,
        pos: "助詞",
        sub: "接続助詞",
        ctype: "",
        cform: "",
        lemma: surface,
        orth_base: surface,
    }
}

fn aux(surface: &'static str, lemma: &'static str, orth_base: &'static str) -> Part {
    aux_form(surface, lemma, orth_base, "終止形-一般")
}

fn aux_form(
    surface: &'static str,
    lemma: &'static str,
    orth_base: &'static str,
    cform: &'static str,
) -> Part {
    Part {
        surface,
        pos: "動詞",
        sub: "非自立可能",
        ctype: "上一段-ア行",
        cform,
        lemma,
        orth_base,
    }
}

#[test]
fn explanatory_no_joins_the_predicate_chain() {
    for (surface, sub) in [("ん", "準体助詞"), ("の", "準体助詞"), ("ん", "終助詞")] {
        let specs = [
            Part {
                surface: "し",
                pos: "動詞",
                sub: "非自立可能",
                ctype: "サ行変格",
                cform: "連用形-一般",
                lemma: "為る",
                orth_base: "する",
            },
            Part {
                surface: "た",
                pos: "助動詞",
                sub: "",
                ctype: "助動詞-タ",
                cform: "連体形-一般",
                lemma: "た",
                orth_base: "た",
            },
            Part {
                surface,
                pos: "助詞",
                sub,
                ctype: "",
                cform: "",
                lemma: "の",
                orth_base: surface,
            },
            Part {
                surface: "だ",
                pos: "助動詞",
                sub: "",
                ctype: "助動詞-ダ",
                cform: "終止形-一般",
                lemma: "だ",
                orth_base: "だ",
            },
        ];
        let text = format!("した{surface}だ");
        let artifact = run(&text, &specs, &[]);
        assert_eq!(artifact.chains.len(), 1, "{text}");
        let chain = &artifact.chains[0];
        assert_eq!(chain.surface_form, text);
        assert_eq!(chain.lookup_form, "する");
        assert_eq!(chain.core_morpheme_indices, vec![0]);
        assert!(chain
            .operators
            .iter()
            .any(|operator| operator.kind == "nominalization"));
    }
    let specs = [
        verb("読む", "連体形-一般"),
        Part {
            surface: "の",
            pos: "助詞",
            sub: "格助詞",
            ctype: "",
            cform: "",
            lemma: "の",
            orth_base: "の",
        },
    ];
    assert_eq!(run("読むの", &specs, &[]).chains.len(), 2);
}

#[test]
fn final_no_stays_separate_even_when_tagged_as_a_nominalizer() {
    for sub in ["準体助詞", "終助詞"] {
        let specs = [
            verb("読む", "連体形-一般"),
            Part {
                surface: "の",
                pos: "助詞",
                sub,
                ctype: "",
                cform: "",
                lemma: "の",
                orth_base: "の",
            },
        ];
        let artifact = run("読むの", &specs, &[]);
        assert_eq!(artifact.chains.len(), 2);
        assert_eq!(artifact.chains[0].surface_form, "読む");
        assert!(artifact.chains[1].operators.is_empty());
    }
}

#[test]
fn nominalizing_sa_joins_the_negative_auxiliary_stem() {
    let specs = [
        Part {
            surface: "底",
            pos: "名詞",
            sub: "普通名詞",
            ctype: "",
            cform: "",
            lemma: "底",
            orth_base: "底",
        },
        Part {
            surface: "知れ",
            pos: "動詞",
            sub: "一般",
            ctype: "下一段-ラ行",
            cform: "未然形-一般",
            lemma: "知れる",
            orth_base: "知れる",
        },
        Part {
            surface: "な",
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-ナイ",
            cform: "語幹-一般",
            lemma: "ない",
            orth_base: "ない",
        },
        Part {
            surface: "さ",
            pos: "接尾辞",
            sub: "名詞的",
            ctype: "",
            cform: "",
            lemma: "さ",
            orth_base: "さ",
        },
    ];
    let artifact = run("底知れなさ", &specs, &[]);
    let chain = artifact
        .chains
        .iter()
        .find(|chain| chain.core_morpheme_indices.contains(&1))
        .unwrap();
    assert_eq!(chain.surface_form, "知れなさ");
    assert_eq!(chain.morpheme_indices, vec![1, 2, 3]);
    assert_eq!(chain.core_morpheme_indices, vec![1]);
    assert_eq!(chain.lookup_form, "知れる");
    assert_eq!(chain.final_state.category, "名詞");
    assert!(chain.operators.iter().any(|op| op.kind == "negative"));
    let occurrence = artifact
        .occurrences
        .iter()
        .find(|occurrence| occurrence.kind == "nominalization")
        .unwrap();
    assert_eq!(occurrence.chain_id, chain.chain_id);
    assert_eq!(occurrence.hit_ranges, vec![[4, 5]]);
    assert_eq!(occurrence.context_range, [1, 5]);
}

#[test]
fn deshou_is_conjectural_after_predicates_and_nominals() {
    for prefix in [
        verb("読む", "終止形-一般"),
        Part {
            surface: "明日",
            pos: "名詞",
            sub: "普通名詞",
            ctype: "",
            cform: "",
            lemma: "明日",
            orth_base: "明日",
        },
        Part {
            surface: "高い",
            pos: "形容詞",
            sub: "一般",
            ctype: "形容詞",
            cform: "終止形-一般",
            lemma: "高い",
            orth_base: "高い",
        },
    ] {
        for surface in ["でしょう", "でしょ"] {
            let text = format!("{}{surface}", prefix.surface);
            let specs = [
                prefix.clone(),
                Part {
                    surface,
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-デス",
                    cform: "意志推量形",
                    lemma: "です",
                    orth_base: "です",
                },
            ];
            let artifact = run(&text, &specs, &[]);
            assert_eq!(artifact.chains.len(), 1, "{text}");
            let operators = artifact.chains[0]
                .operators
                .iter()
                .filter(|op| op.source_morpheme_range == [1, 2])
                .collect::<Vec<_>>();
            assert_eq!(operators.len(), 1, "{text}");
            assert_eq!(operators[0].kind, "deshou_modality");
            assert_eq!(operators[0].normalized_form.as_deref(), Some("でしょう"));
            assert_eq!(operators[0].concept_id, "grammar.modality.deshou");
        }
    }
}

#[test]
fn classical_rules_keep_their_own_entities_and_concepts() {
    let cases = [
        (
            "行かずに",
            vec![
                Part {
                    surface: "行か",
                    pos: "動詞",
                    sub: "自立",
                    ctype: "五段-カ行",
                    cform: "未然形-一般",
                    lemma: "行く",
                    orth_base: "行く",
                },
                Part {
                    surface: "ず",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-ヌ",
                    cform: "連用形-一般",
                    lemma: "ず",
                    orth_base: "ず",
                },
                Part {
                    surface: "に",
                    pos: "助詞",
                    sub: "格助詞",
                    ctype: "",
                    cform: "",
                    lemma: "に",
                    orth_base: "に",
                },
            ],
            "negative_zuni",
            "grammar.classical.zu",
        ),
        (
            "夢のごとし",
            vec![
                Part {
                    surface: "夢",
                    pos: "名詞",
                    sub: "普通名詞",
                    ctype: "",
                    cform: "",
                    lemma: "夢",
                    orth_base: "夢",
                },
                Part {
                    surface: "の",
                    pos: "助詞",
                    sub: "格助詞",
                    ctype: "",
                    cform: "",
                    lemma: "の",
                    orth_base: "の",
                },
                Part {
                    surface: "ごとし",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-ゴトシ",
                    cform: "終止形-一般",
                    lemma: "ごとし",
                    orth_base: "ごとし",
                },
            ],
            "classical_gotoshi",
            "grammar.classical.gotoshi",
        ),
        (
            "行くべし",
            vec![
                Part {
                    surface: "行く",
                    pos: "動詞",
                    sub: "自立",
                    ctype: "五段-カ行",
                    cform: "終止形-一般",
                    lemma: "行く",
                    orth_base: "行く",
                },
                Part {
                    surface: "べし",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-ベシ",
                    cform: "終止形-一般",
                    lemma: "べし",
                    orth_base: "べし",
                },
            ],
            "classical_beshi",
            "grammar.classical.beshi",
        ),
    ];
    for (text, specs, kind, concept) in cases {
        let artifact = run(text, &specs, &[]);
        let chain = artifact
            .chains
            .iter()
            .find(|chain| chain.operators.iter().any(|operator| operator.kind == kind))
            .unwrap_or_else(|| {
                panic!(
                    "{text}: {:?}",
                    artifact
                        .chains
                        .iter()
                        .map(|chain| (&chain.surface_form, &chain.final_state, &chain.operators))
                        .collect::<Vec<_>>()
                )
            });
        let operator = chain
            .operators
            .iter()
            .find(|operator| operator.kind == kind)
            .unwrap();
        assert_eq!(operator.concept_id, concept, "{text}");
        assert_eq!(operator.executor, "classical_conjugation", "{text}");
        assert_eq!(chain.final_state.entity_class, "文言活用类", "{text}");
    }

    let modern = run(
        "夢のようだ",
        &[
            Part {
                surface: "夢",
                pos: "名詞",
                sub: "普通名詞",
                ctype: "",
                cform: "",
                lemma: "夢",
                orth_base: "夢",
            },
            Part {
                surface: "の",
                pos: "助詞",
                sub: "格助詞",
                ctype: "",
                cform: "",
                lemma: "の",
                orth_base: "の",
            },
            Part {
                surface: "よう",
                pos: "名詞",
                sub: "非自立",
                ctype: "",
                cform: "",
                lemma: "よう",
                orth_base: "よう",
            },
            Part {
                surface: "だ",
                pos: "助動詞",
                sub: "",
                ctype: "助動詞-ダ",
                cform: "終止形-一般",
                lemma: "だ",
                orth_base: "だ",
            },
        ],
        &[],
    );
    assert!(modern
        .chains
        .iter()
        .flat_map(|chain| &chain.operators)
        .any(|operator| operator.kind == "copula"));
    assert!(!modern
        .chains
        .iter()
        .flat_map(|chain| &chain.operators)
        .any(|operator| operator.executor == "classical_conjugation"));
}

#[test]
fn synthetic_reconstruction_covers_thirty_contract_cases() {
    let ordinary = [
        (
            "読んだ",
            vec![
                verb("読ん", "連用形-撥音便"),
                Part {
                    surface: "だ",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-タ",
                    cform: "基本形",
                    lemma: "だ",
                    orth_base: "だ",
                },
            ],
            "past",
        ),
        (
            "読まない",
            vec![
                verb("読ま", "未然形-一般"),
                Part {
                    surface: "ない",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-ナイ",
                    cform: "基本形",
                    lemma: "ない",
                    orth_base: "ない",
                },
            ],
            "negative",
        ),
        (
            "読ませる",
            vec![
                verb("読ま", "未然形-一般"),
                Part {
                    surface: "せる",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-セル",
                    cform: "基本形",
                    lemma: "せる",
                    orth_base: "せる",
                },
            ],
            "causative",
        ),
        (
            "読まれる",
            vec![
                verb("読ま", "未然形-一般"),
                Part {
                    surface: "れる",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-レル",
                    cform: "基本形",
                    lemma: "れる",
                    orth_base: "れる",
                },
            ],
            "passive_potential",
        ),
        (
            "読みます",
            vec![
                verb("読み", "連用形-一般"),
                Part {
                    surface: "ます",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-マス",
                    cform: "基本形",
                    lemma: "ます",
                    orth_base: "ます",
                },
            ],
            "politeness_masu",
        ),
        (
            "読みたい",
            vec![
                verb("読み", "連用形-一般"),
                Part {
                    surface: "たい",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-タイ",
                    cform: "基本形",
                    lemma: "たい",
                    orth_base: "たい",
                },
            ],
            "desire",
        ),
        (
            "読めば",
            vec![
                verb("読め", "仮定形-一般"),
                Part {
                    surface: "ば",
                    pos: "助詞",
                    sub: "接続助詞",
                    ctype: "",
                    cform: "",
                    lemma: "ば",
                    orth_base: "ば",
                },
            ],
            "ba_connection",
        ),
        ("読もう", vec![verb("読もう", "意志推量形")], "volitional"),
        ("読め", vec![verb("読め", "命令形")], "imperative"),
        (
            "行くな",
            vec![
                Part {
                    surface: "行く",
                    pos: "動詞",
                    sub: "自立",
                    ctype: "五段-カ行促音便",
                    cform: "終止形-一般",
                    lemma: "行く",
                    orth_base: "行く",
                },
                Part {
                    surface: "な",
                    pos: "助詞",
                    sub: "終助詞",
                    ctype: "",
                    cform: "",
                    lemma: "な",
                    orth_base: "な",
                },
            ],
            "prohibitive",
        ),
    ];
    for (text, specs, kind) in ordinary {
        let artifact = run(text, &specs, &[]);
        assert!(
            artifact
                .chains
                .iter()
                .any(|chain| chain.operators.iter().any(|operator| operator.kind == kind)),
            "{text}: {kind}"
        );
    }

    let adjective_chains = [
        (
            "読みたくない",
            vec![
                verb("読み", "連用形-一般"),
                Part {
                    surface: "たく",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-タイ",
                    cform: "連用形-一般",
                    lemma: "たい",
                    orth_base: "たい",
                },
                Part {
                    surface: "ない",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-ナイ",
                    cform: "基本形",
                    lemma: "ない",
                    orth_base: "ない",
                },
            ],
        ),
        (
            "読みたかった",
            vec![
                verb("読み", "連用形-一般"),
                Part {
                    surface: "たかっ",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-タイ",
                    cform: "連用形-促音便",
                    lemma: "たい",
                    orth_base: "たい",
                },
                Part {
                    surface: "た",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-タ",
                    cform: "基本形",
                    lemma: "た",
                    orth_base: "た",
                },
            ],
        ),
        (
            "読まなかった",
            vec![
                verb("読ま", "未然形-一般"),
                Part {
                    surface: "なかっ",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-ナイ",
                    cform: "連用形-促音便",
                    lemma: "ない",
                    orth_base: "ない",
                },
                Part {
                    surface: "た",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-タ",
                    cform: "基本形",
                    lemma: "た",
                    orth_base: "た",
                },
            ],
        ),
    ];
    for (text, specs) in adjective_chains {
        let artifact = run(text, &specs, &[]);
        let chain = artifact
            .chains
            .iter()
            .find(|chain| chain.surface_form == text)
            .unwrap_or_else(|| {
                panic!(
                    "{text}: {:?}",
                    artifact
                        .chains
                        .iter()
                        .map(|chain| (&chain.surface_form, &chain.final_state, &chain.operators))
                        .collect::<Vec<_>>()
                )
            });
        assert!(
            chain
                .operators
                .iter()
                .any(|operator| operator.kind == "negative" || operator.kind == "desire"),
            "{text}"
        );
        assert_eq!(chain.final_state.category, "形容詞", "{text}");
        assert_eq!(chain.final_state.entity_class, "イ形容词转换类", "{text}");
        let conversion = chain
            .operators
            .iter()
            .find(|operator| operator.kind == "desire" || operator.kind == "negative")
            .unwrap();
        assert_eq!(conversion.state_after.category, "形容詞", "{text}");
        assert_eq!(
            conversion.state_after.entity_class, "イ形容词转换类",
            "{text}"
        );
        if text == "読みたくない" || text == "読まなかった" {
            assert!(chain
                .operators
                .iter()
                .any(|operator| operator.kind == "negative"));
        }
        if text == "読みたかった" || text == "読まなかった" {
            assert!(chain
                .operators
                .iter()
                .any(|operator| operator.kind == "past"));
        }
    }

    let compounds = [
        ("読みすぎる", "すぎる", "excessive"),
        ("読み始める", "始める", "inceptive"),
        ("読み続ける", "続ける", "continuative_aspect"),
        ("読み終わる", "終わる", "terminative"),
        ("読みまくる", "まくる", "iterative_aspect"),
        ("読み出す", "出す", "inchoative"),
        ("読み直す", "直す", "corrective"),
        ("読みかける", "かける", "incomplete"),
        ("読み切る", "切る", "completion"),
        ("読みかねる", "かねる", "reluctance"),
        ("読み損なう", "損なう", "failure"),
        ("読みそびれる", "そびれる", "missed_opportunity"),
    ];
    for (text, surface, kind) in compounds {
        let specs = [
            verb("読み", "連用形-一般"),
            Part {
                surface,
                pos: "助動詞",
                sub: "非自立可能",
                ctype: "",
                cform: "基本形",
                lemma: surface,
                orth_base: surface,
            },
        ];
        let artifact = run(text, &specs, &[]);
        assert!(
            artifact
                .chains
                .iter()
                .any(|chain| chain.role == MorphologyRole::Functional
                    && chain.operators.iter().any(|operator| operator.kind == kind)),
            "{text}: {kind}"
        );
    }

    let auxiliaries = [
        ("読んでおく", "おく", "置く", "te_oku"),
        ("読んでいる", "いる", "居る", "te_iru"),
        ("読んでしまう", "しまう", "仕舞う", "te_shimau"),
        ("読んでみる", "みる", "見る", "te_miru"),
        ("読んでくださる", "くださる", "下さる", "te_kudasaru"),
    ];
    for (text, surface, lemma, kind) in auxiliaries {
        let specs = [
            verb("読ん", "連用形-撥音便"),
            te(),
            aux(surface, lemma, lemma),
        ];
        let artifact = run(text, &specs, &[]);
        let chain = artifact
            .chains
            .iter()
            .find(|chain| chain.role == MorphologyRole::Functional)
            .unwrap();
        assert!(
            chain.operators.iter().any(|operator| operator.kind == kind),
            "{text}: {:?} {:?}",
            chain.surface_form,
            chain
                .operators
                .iter()
                .map(|operator| &operator.kind)
                .collect::<Vec<_>>()
        );
        let occurrence = artifact
            .occurrences
            .iter()
            .find(|occurrence| occurrence.kind == kind)
            .unwrap();
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
        let particle = Part {
            surface: particle,
            pos: "助詞",
            sub: if particle == "さえ" {
                "副助詞"
            } else {
                "係助詞"
            },
            ctype: "",
            cform: "",
            lemma: particle,
            orth_base: particle,
        };
        let specs = [
            verb("読ん", "連用形-撥音便"),
            te(),
            particle,
            aux(surface, lemma, lemma),
        ];
        let artifact = run(text, &specs, &[2]);
        let chain = artifact
            .chains
            .iter()
            .find(|chain| {
                chain.role == MorphologyRole::Functional && chain.parent_chain_id.is_some()
            })
            .unwrap();
        assert_eq!(
            chain.status, "resolved",
            "{text} should use the verified GiNZA bridge: {:?}",
            artifact.diagnostics
        );
        assert!(
            chain.operators.iter().any(|operator| operator.kind == kind),
            "{text}: {} {:?} {:?}",
            chain.status,
            chain.surface_form,
            chain
                .operators
                .iter()
                .map(|operator| &operator.kind)
                .collect::<Vec<_>>()
        );
        assert!(
            chain
                .evidence
                .iter()
                .any(|item| item == "connector_rule:te_connection"),
            "{text}: {:?}",
            chain.evidence
        );
    }

    let lemma_mappings = [
        ("読んで置く", "置く", "te_oku"),
        ("読んで居る", "居る", "te_iru"),
        ("読んで仕舞う", "仕舞う", "te_shimau"),
        ("読んで見る", "見る", "te_miru"),
        ("読んで下さる", "下さる", "te_kudasaru"),
    ];
    for (text, lemma, kind) in lemma_mappings {
        let specs = [
            verb("読ん", "連用形-撥音便"),
            te(),
            aux(lemma, lemma, lemma),
        ];
        let artifact = run(text, &specs, &[]);
        let chain = artifact
            .chains
            .iter()
            .find(|chain| {
                chain.role == MorphologyRole::Functional && chain.parent_chain_id.is_some()
            })
            .unwrap();
        assert_eq!(chain.lookup_form, lemma, "{text}");
        assert!(
            chain.operators.iter().any(|operator| operator.kind == kind),
            "{text}"
        );
    }
}

#[test]
fn auxiliary_support_owns_inflection_recursion_and_depth_diagnostics() {
    let cases = [
        (
            "読んでいない",
            vec![
                verb("読ん", "連用形-撥音便"),
                te(),
                aux_form("い", "居る", "居る", "未然形-一般"),
                Part {
                    surface: "ない",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-ナイ",
                    cform: "基本形",
                    lemma: "ない",
                    orth_base: "ない",
                },
            ],
            &["te_iru", "negative"][..],
            false,
        ),
        (
            "読んでいなかった",
            vec![
                verb("読ん", "連用形-撥音便"),
                te(),
                aux_form("い", "居る", "居る", "未然形-一般"),
                Part {
                    surface: "なかっ",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-ナイ",
                    cform: "連用形-促音便",
                    lemma: "ない",
                    orth_base: "ない",
                },
                Part {
                    surface: "た",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-タ",
                    cform: "基本形",
                    lemma: "た",
                    orth_base: "た",
                },
            ],
            &["te_iru", "negative", "past"][..],
            true,
        ),
        (
            "読んでいました",
            vec![
                verb("読ん", "連用形-撥音便"),
                te(),
                aux_form("い", "居る", "居る", "連用形-一般"),
                Part {
                    surface: "まし",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-マス",
                    cform: "連用形-一般",
                    lemma: "ます",
                    orth_base: "ます",
                },
                Part {
                    surface: "た",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-タ",
                    cform: "基本形",
                    lemma: "た",
                    orth_base: "た",
                },
            ],
            &["te_iru", "politeness_masu", "past"][..],
            true,
        ),
    ];

    for (text, specs, expected_kinds, terminal) in cases {
        let artifact = run_with_support_edges(text, &specs, &[(2, 0)]);
        let child = artifact
            .chains
            .iter()
            .find(|chain| {
                chain
                    .operators
                    .iter()
                    .any(|operator| operator.kind == "te_iru")
            })
            .unwrap_or_else(|| panic!("{text}: {:?}", artifact.diagnostics));
        assert_eq!(
            child.parent_chain_id.as_deref(),
            Some("morphology:m0"),
            "{text}"
        );
        assert_eq!(child.lookup_form, "居る", "{text}");
        assert_eq!(
            child.status, "resolved",
            "{text}: {:?}",
            artifact.diagnostics
        );
        assert_eq!(child.terminal, terminal, "{text}");
        for kind in expected_kinds {
            assert!(
                child
                    .operators
                    .iter()
                    .any(|operator| operator.kind == *kind),
                "{text}: {kind} not in {:?}",
                child
                    .operators
                    .iter()
                    .map(|operator| &operator.kind)
                    .collect::<Vec<_>>()
            );
        }
        assert!(
            child
                .evidence
                .iter()
                .any(|item| item == "connector_rule:te_connection"),
            "{text}: {:?}",
            child.evidence
        );
        let occurrence = artifact
            .occurrences
            .iter()
            .find(|occurrence| occurrence.kind == "te_iru")
            .unwrap();
        assert_eq!(occurrence.hit_ranges, vec![[2, 3], [3, 4]], "{text}");
        assert_eq!(occurrence.char_range, [2, 4], "{text}");
    }

    let tari_specs = [
        verb("読ん", "連用形-撥音便"),
        Part {
            surface: "だり",
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-タ",
            cform: "連用形-一般",
            lemma: "たり",
            orth_base: "たり",
        },
        Part {
            surface: "する",
            pos: "動詞",
            sub: "自立",
            ctype: "サ行変格",
            cform: "終止形-一般",
            lemma: "する",
            orth_base: "する",
        },
    ];
    let tari = run_with_support_edges("読んだりする", &tari_specs, &[(2, 0)]);
    let tari_child = tari
        .chains
        .iter()
        .find(|chain| {
            chain
                .operators
                .iter()
                .any(|operator| operator.kind == "tari_suru")
        })
        .unwrap_or_else(|| panic!("{:?}", tari.diagnostics));
    assert_eq!(tari_child.parent_chain_id.as_deref(), Some("morphology:m0"));
    assert_eq!(tari_child.lookup_form, "する");
    assert_eq!(tari_child.status, "resolved", "{:?}", tari.diagnostics);
    assert!(tari_child
        .evidence
        .iter()
        .any(|item| item == "connector_rule:enumerative"));

    let invalid_specs = [
        Part {
            surface: "高く",
            pos: "形容詞",
            sub: "一般",
            ctype: "形容詞",
            cform: "連用形-一般",
            lemma: "高い",
            orth_base: "高い",
        },
        te_token("て"),
        aux("いる", "居る", "居る"),
    ];
    let invalid = run_with_support_edges("高くている", &invalid_specs, &[(2, 0)]);
    let invalid_child = invalid
        .chains
        .iter()
        .find(|chain| {
            chain
                .operators
                .iter()
                .any(|operator| operator.kind == "te_iru")
        })
        .unwrap();
    assert_eq!(invalid_child.status, "candidate");
    assert!(invalid
        .diagnostics
        .iter()
        .any(|item| item.starts_with("auxiliary_input_category:")));

    let mut recursive_specs = vec![verb("読ん", "連用形-撥音便"), te()];
    let mut recursive_text = String::from("読んで");
    let mut support_edges = Vec::new();
    let mut parent_index = 0;
    for layer in 0..9 {
        let support_index = recursive_specs.len();
        recursive_specs.push(aux_form("い", "居る", "居る", "連用形-一般"));
        recursive_text.push('い');
        support_edges.push((support_index, parent_index));
        parent_index = support_index;
        if layer < 8 {
            recursive_specs.push(te_token("て"));
            recursive_text.push('て');
        }
    }
    let recursive = run_with_support_edges(&recursive_text, &recursive_specs, &support_edges);
    let support_chains = recursive
        .chains
        .iter()
        .filter(|chain| {
            chain
                .operators
                .iter()
                .any(|operator| operator.kind == "te_iru")
        })
        .collect::<Vec<_>>();
    assert_eq!(support_chains.len(), 9);
    for (index, chain) in support_chains.iter().enumerate() {
        let expected = if index < 8 { "resolved" } else { "candidate" };
        assert_eq!(chain.status, expected, "depth {}", index + 1);
    }
    assert!(recursive
        .diagnostics
        .iter()
        .any(|item| item.starts_with("auxiliary_depth_limit:")));
}

/// 阶段七用例共用来源节点，依存关系按各用例显式声明。
fn stage_seven_artifact(
    specs: &[Part],
    edges: Option<&[(usize, usize, &str)]>,
) -> readux_nlp::morphology::MorphologyArtifact {
    let text = specs.iter().map(|part| part.surface).collect::<String>();
    let (source, morphemes) = parts(&text, specs);
    let external = edges.map(|edges| {
        let mut artifact = ginza(&text, &source, &[]);
        artifact.relations = edges
            .iter()
            .enumerate()
            .map(|(index, &(dependent, head, label))| SourceRelation {
                id: format!("edge{index}"),
                kind: RelationKind::Dependency,
                source: Endpoint::Node {
                    id: format!("t{dependent}"),
                },
                target: Endpoint::Node {
                    id: format!("t{head}"),
                },
                label: label.into(),
                features: json!({}),
            })
            .collect();
        artifact
    });
    readux_nlp::morphology::collect_with_external(
        &source,
        &morphemes,
        &external.into_iter().collect::<Vec<_>>(),
    )
    .unwrap()
}

fn focus_particle(surface: &'static str, sub: &'static str) -> Part {
    Part {
        surface,
        pos: "助詞",
        sub,
        ctype: "",
        cform: "",
        lemma: surface,
        orth_base: surface,
    }
}

#[test]
fn stage_seven_whitelist_preserves_owner_state_and_complete_support_evidence() {
    for (surface, sub) in [
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
        let specs = [
            verb("読ん", "連用形-撥音便"),
            te(),
            focus_particle(surface, sub),
            aux("いる", "居る", "居る"),
        ];
        let artifact = stage_seven_artifact(
            &specs,
            Some(&[(1, 0, "mark"), (2, 0, "case"), (3, 0, "advcl")]),
        );
        assert_eq!(
            artifact.chains.len(),
            2,
            "{surface}: {:?}",
            artifact.diagnostics
        );
        let owner = &artifact.chains[0];
        let child = &artifact.chains[1];
        let insertion = owner
            .operators
            .iter()
            .find(|op| op.kind == "insertion")
            .unwrap();
        let connector = owner
            .operators
            .iter()
            .find(|op| op.kind == "te_connection")
            .unwrap();
        assert_eq!(owner.lookup_form, "読む", "{surface}");
        assert_eq!(owner.core_morpheme_indices, vec![0], "{surface}");
        assert_eq!(owner.morpheme_indices, vec![0, 1, 2], "{surface}");
        assert_eq!(owner.parent_chain_id, None, "{surface}");
        assert_eq!(owner.final_state, connector.state_after, "{surface}");
        assert_eq!(insertion.state_before, connector.state_after, "{surface}");
        assert_eq!(insertion.state_before, insertion.state_after, "{surface}");
        assert_eq!(insertion.input_state, "読んで", "{surface}");
        assert_eq!(insertion.output_state, format!("読んで{surface}"));
        assert_eq!(insertion.executor, "insertion");
        assert_eq!(insertion.stage, "insertion");
        assert_eq!(insertion.order_rank, 0);
        assert_eq!(insertion.terminal_mode, "preserve");
        assert!(owner.terminal, "{surface}: て形中顿许可应保持");
        assert!(insertion
            .evidence
            .iter()
            .any(|e| e == "insertion_position:te_support"));
        assert!(owner
            .source_evidence
            .iter()
            .any(|e| { e.relation_id.as_deref() == Some("edge1") && e.reason == "case" }));
        assert_eq!(
            child.parent_chain_id.as_deref(),
            Some(owner.chain_id.as_str())
        );
        assert_eq!(child.lookup_form, "居る");
        assert_eq!(
            child.status, "resolved",
            "{surface}: {:?}",
            artifact.diagnostics
        );
        let support = artifact
            .occurrences
            .iter()
            .find(|o| o.kind == "te_iru")
            .unwrap();
        assert_eq!(support.morpheme_indices, vec![1, 2, 3], "{surface}");
        assert_eq!(
            support.hit_ranges,
            vec![
                connector.char_range,
                insertion.char_range,
                child.anchor_range
            ],
            "{surface}",
        );
        assert_eq!(
            support.operator_ids,
            vec![
                connector.operator_id.clone(),
                insertion.operator_id.clone(),
                child
                    .operators
                    .iter()
                    .find(|o| o.kind == "te_iru")
                    .unwrap()
                    .operator_id
                    .clone()
            ],
            "{surface}",
        );
        assert!(support.source_evidence.iter().any(|e| e.reason == "case"));
        let occurrence = artifact
            .occurrences
            .iter()
            .find(|o| o.kind == "insertion")
            .unwrap();
        assert_eq!(occurrence.chain_id, owner.chain_id);
        assert_eq!(occurrence.char_range, insertion.char_range);
        assert_eq!(occurrence.context_range, owner.char_range);
        assert_eq!(occurrence.morpheme_indices, vec![2]);
        assert_eq!(owner.char_range[1], insertion.char_range[1]);
    }
}

#[test]
fn stage_seven_rejects_undeclared_particles_and_incomplete_syntax_bridges() {
    for particle in [
        focus_particle("に", "格助詞"),
        focus_particle("を", "格助詞"),
        focus_particle("が", "格助詞"),
        focus_particle("と", "格助詞"),
        focus_particle("ばかり", "副助詞"),
        focus_particle("は", "格助詞"),
    ] {
        let specs = [
            verb("読ん", "連用形-撥音便"),
            te(),
            particle.clone(),
            aux("いる", "居る", "居る"),
        ];
        let artifact = stage_seven_artifact(
            &specs,
            Some(&[(1, 0, "mark"), (2, 0, "case"), (3, 0, "advcl")]),
        );
        assert!(
            !artifact
                .chains
                .iter()
                .any(|chain| chain.operators.iter().any(|op| op.kind == "insertion")),
            "{}",
            particle.surface
        );
        assert!(
            !artifact
                .chains
                .iter()
                .any(|chain| chain.operators.iter().any(|op| op.kind == "te_iru")),
            "{}",
            particle.surface
        );
    }
    let specs = [
        verb("読ん", "連用形-撥音便"),
        te(),
        focus_particle("は", "係助詞"),
        aux("いる", "居る", "居る"),
    ];
    for edges in [
        vec![],
        vec![(1, 0, "mark"), (3, 0, "advcl")],
        vec![(1, 0, "mark"), (2, 3, "case"), (3, 0, "advcl")],
        vec![(1, 0, "mark"), (2, 0, "obj"), (3, 0, "advcl")],
        vec![(1, 0, "mark"), (2, 0, "case")],
        vec![(0, 1, "mark"), (2, 0, "case"), (3, 0, "advcl")],
    ] {
        let artifact = stage_seven_artifact(&specs, Some(&edges));
        assert!(
            !artifact
                .chains
                .iter()
                .any(|chain| chain.operators.iter().any(|op| op.kind == "insertion")),
            "{edges:?}"
        );
    }
    for separator in [
        Part {
            surface: " ",
            pos: "空白",
            sub: "",
            ctype: "",
            cform: "",
            lemma: " ",
            orth_base: " ",
        },
        Part {
            surface: "。",
            pos: "補助記号",
            sub: "句点",
            ctype: "",
            cform: "",
            lemma: "。",
            orth_base: "。",
        },
    ] {
        let artifact = stage_seven_artifact(
            &[
                specs[0].clone(),
                specs[1].clone(),
                specs[2].clone(),
                separator,
                specs[3].clone(),
            ],
            Some(&[(1, 0, "mark"), (2, 0, "case"), (4, 0, "advcl")]),
        );
        assert!(!artifact
            .chains
            .iter()
            .any(|chain| chain.operators.iter().any(|op| op.kind == "insertion")));
    }
}

#[test]
fn stage_seven_consecutive_insertions_keep_original_parent_and_following_inflection() {
    let specs = [
        verb("読ん", "連用形-撥音便"),
        te(),
        focus_particle("だけ", "副助詞"),
        focus_particle("は", "係助詞"),
        aux_form("い", "居る", "居る", "連用形-一般"),
        Part {
            surface: "まし",
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-マス",
            cform: "連用形-一般",
            lemma: "ます",
            orth_base: "ます",
        },
        Part {
            surface: "た",
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-タ",
            cform: "終止形-一般",
            lemma: "た",
            orth_base: "た",
        },
    ];
    let artifact = stage_seven_artifact(
        &specs,
        Some(&[
            (1, 0, "mark"),
            (2, 0, "case"),
            (3, 0, "mark"),
            (4, 0, "advcl"),
        ]),
    );
    assert_eq!(artifact.chains.len(), 2, "{:?}", artifact.diagnostics);
    let owner = &artifact.chains[0];
    let child = &artifact.chains[1];
    let insertions = owner
        .operators
        .iter()
        .filter(|op| op.kind == "insertion")
        .collect::<Vec<_>>();
    assert_eq!(insertions.len(), 2);
    assert_eq!(insertions[0].state_after, insertions[1].state_before);
    assert_eq!(insertions[1].state_after, owner.final_state);
    assert_eq!(insertions[0].input_state, "読んで");
    assert_eq!(insertions[1].input_state, "読んでだけ");
    assert_eq!(insertions[1].output_state, "読んでだけは");
    assert_eq!(owner.morpheme_indices, vec![0, 1, 2, 3]);
    assert_eq!(owner.core_morpheme_indices, vec![0]);
    assert_eq!(
        child.parent_chain_id.as_deref(),
        Some(owner.chain_id.as_str())
    );
    assert_eq!(child.surface_form, "いました");
    assert_eq!(child.lookup_form, "居る");
    assert_eq!(child.status, "resolved", "{:?}", artifact.diagnostics);
    assert!(child.terminal);
    for kind in ["te_iru", "politeness_masu", "past"] {
        assert!(child.operators.iter().any(|op| op.kind == kind), "{kind}");
    }
    let occurrence = artifact
        .occurrences
        .iter()
        .find(|o| o.kind == "te_iru")
        .unwrap();
    assert_eq!(occurrence.hit_ranges, vec![[2, 3], [3, 5], [5, 6], [6, 7]]);
    assert_eq!(occurrence.morpheme_indices, vec![1, 2, 3, 4]);
    assert_eq!(occurrence.context_range, [0, 10]);
    assert_eq!(
        occurrence
            .source_evidence
            .iter()
            .filter(|e| matches!(e.reason.as_str(), "case" | "mark"))
            .count(),
        3
    );
}

#[test]
fn stage_seven_general_insertions_preserve_state_terminal_and_nominalization_boundaries() {
    let noun = Part {
        surface: "本",
        pos: "名詞",
        sub: "普通名詞",
        ctype: "",
        cform: "",
        lemma: "本",
        orth_base: "本",
    };
    let high = Part {
        surface: "高く",
        pos: "形容詞",
        sub: "一般",
        ctype: "形容詞",
        cform: "連用形-一般",
        lemma: "高い",
        orth_base: "高い",
    };
    let nai = Part {
        surface: "ない",
        pos: "助動詞",
        sub: "",
        ctype: "助動詞-ナイ",
        cform: "終止形-一般",
        lemma: "ない",
        orth_base: "ない",
    };
    for prefix in [
        vec![noun.clone()],
        vec![verb("読む", "連体形-一般")],
        vec![verb("読み", "連用形-一般")],
        vec![high.clone()],
        vec![
            verb("読ん", "連用形-撥音便"),
            Part {
                surface: "だ",
                pos: "助動詞",
                sub: "",
                ctype: "助動詞-タ",
                cform: "終止形-一般",
                lemma: "た",
                orth_base: "た",
            },
        ],
    ] {
        let baseline = stage_seven_artifact(&prefix, None);
        let mut specs = prefix;
        specs.push(focus_particle("は", "係助詞"));
        let artifact = stage_seven_artifact(&specs, None);
        assert_eq!(artifact.chains.len(), 1, "{:?}", artifact.diagnostics);
        let chain = &artifact.chains[0];
        assert_eq!(chain.final_state, baseline.chains[0].final_state);
        assert_eq!(chain.terminal, baseline.chains[0].terminal);
        assert_eq!(chain.lookup_form, baseline.chains[0].lookup_form);
        assert_eq!(
            chain.core_morpheme_indices,
            baseline.chains[0].core_morpheme_indices
        );
        assert_eq!(chain.status, "resolved", "{:?}", artifact.diagnostics);
        assert!(chain.operators.iter().any(|op| op.executor == "insertion"));
    }
    let contrast = stage_seven_artifact(&[high, focus_particle("は", "係助詞"), nai], None);
    assert_eq!(contrast.chains.len(), 1);
    assert_eq!(contrast.chains[0].surface_form, "高くはない");
    assert_eq!(contrast.chains[0].final_state.category, "形容詞");
    assert_eq!(contrast.chains[0].status, "resolved");
    let nominalized = stage_seven_artifact(
        &[
            verb("読む", "連体形-一般"),
            focus_particle("だけ", "副助詞"),
            focus_particle("の", "準体助詞"),
            Part {
                surface: "だ",
                pos: "助動詞",
                sub: "",
                ctype: "助動詞-ダ",
                cform: "終止形-一般",
                lemma: "だ",
                orth_base: "だ",
            },
        ],
        None,
    );
    assert_eq!(nominalized.chains.len(), 1, "{:?}", nominalized.chains);
    assert_eq!(
        nominalized.chains[0]
            .operators
            .iter()
            .filter(|op| op.kind == "insertion")
            .count(),
        1
    );
    assert!(nominalized.chains[0]
        .operators
        .iter()
        .any(|op| op.kind == "nominalization"));

    for parts in [
        vec![noun.clone(), focus_particle("とか", "副助詞")],
        vec![
            noun,
            focus_particle("と", "格助詞"),
            focus_particle("か", "副助詞"),
        ],
    ] {
        let artifact = stage_seven_artifact(&parts, None);
        assert_eq!(artifact.chains.len(), 1);
        assert_eq!(artifact.chains[0].final_state.category, "名詞");
        assert_eq!(
            artifact.chains[0].status, "resolved",
            "{:?}",
            artifact.diagnostics
        );
        let operator = artifact.chains[0]
            .operators
            .iter()
            .find(|op| op.kind == "exemplification")
            .unwrap();
        assert_eq!(operator.state_before, operator.state_after);
        assert_eq!(operator.executor, "insertion");
        assert!(!artifact
            .diagnostics
            .iter()
            .any(|e| e.starts_with("unknown_rule_executor:")));
    }
}

#[test]
fn stage_seven_reenters_recursive_support_without_increasing_insertion_depth() {
    let specs = [
        verb("読ん", "連用形-撥音便"),
        te(),
        focus_particle("は", "係助詞"),
        aux_form("い", "居る", "居る", "連用形-一般"),
        te_token("て"),
        focus_particle("も", "係助詞"),
        aux("みる", "見る", "見る"),
    ];
    let artifact = stage_seven_artifact(
        &specs,
        Some(&[
            (1, 0, "mark"),
            (2, 0, "case"),
            (3, 0, "advcl"),
            (4, 3, "mark"),
            (5, 3, "case"),
            (6, 3, "aux"),
        ]),
    );
    assert_eq!(artifact.chains.len(), 3, "{:?}", artifact.diagnostics);
    assert!(
        artifact
            .chains
            .iter()
            .all(|chain| chain.status == "resolved"),
        "{:?}: {:?}",
        artifact.diagnostics,
        artifact
            .chains
            .iter()
            .map(|chain| (&chain.surface_form, &chain.status, &chain.final_state))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        artifact.chains[1].parent_chain_id.as_deref(),
        Some(artifact.chains[0].chain_id.as_str())
    );
    assert_eq!(
        artifact.chains[2].parent_chain_id.as_deref(),
        Some(artifact.chains[1].chain_id.as_str())
    );
    assert_eq!(
        artifact.chains[1]
            .operators
            .iter()
            .filter(|op| op.kind == "insertion")
            .count(),
        1
    );
    assert_eq!(artifact.chains[2].lookup_form, "見る");
    let occurrence = artifact
        .occurrences
        .iter()
        .find(|o| o.kind == "te_miru")
        .unwrap();
    assert_eq!(occurrence.context_range, [0, 9]);
    assert_eq!(occurrence.morpheme_indices, vec![4, 5, 6]);
    assert!(!artifact
        .diagnostics
        .iter()
        .any(|e| e.starts_with("auxiliary_depth_limit:")));
}

#[test]
fn stage_seven_restriction_continues_into_judgement_and_explanatory_nominalization() {
    let specs = [
        Part {
            surface: "気付き",
            pos: "動詞",
            sub: "一般",
            ctype: "五段-カ行",
            cform: "連用形-一般",
            lemma: "気付く",
            orth_base: "気付く",
        },
        Part {
            surface: "たく",
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-タイ",
            cform: "連用形-一般",
            lemma: "たい",
            orth_base: "たい",
        },
        Part {
            surface: "なかっ",
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-ナイ",
            cform: "連用形-促音便",
            lemma: "ない",
            orth_base: "ない",
        },
        Part {
            surface: "た",
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-タ",
            cform: "終止形-一般",
            lemma: "た",
            orth_base: "た",
        },
        focus_particle("だけ", "副助詞"),
        Part {
            surface: "な",
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-ダ",
            cform: "連体形-一般",
            lemma: "だ",
            orth_base: "だ",
        },
        focus_particle("の", "準体助詞"),
        Part {
            surface: "だ",
            pos: "助動詞",
            sub: "",
            ctype: "助動詞-ダ",
            cform: "終止形-一般",
            lemma: "だ",
            orth_base: "だ",
        },
    ];
    let artifact = stage_seven_artifact(&specs, None);
    assert_eq!(
        artifact.chains.len(),
        1,
        "{:?}: {:?}",
        artifact.diagnostics,
        artifact
            .chains
            .iter()
            .map(|chain| (
                &chain.surface_form,
                &chain.final_state,
                chain
                    .operators
                    .iter()
                    .map(|op| &op.kind)
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>()
    );
    let owner = &artifact.chains[0];
    assert_eq!(owner.surface_form, "気付きたくなかっただけなのだ");
    assert_eq!(owner.lookup_form, "気付く");
    assert_eq!(owner.core_morpheme_indices, vec![0]);
    assert_eq!(owner.status, "resolved", "{:?}", artifact.diagnostics);
    let insertion = owner
        .operators
        .iter()
        .find(|op| op.kind == "insertion")
        .unwrap();
    assert_eq!(insertion.state_before.category, "形容詞");
    assert_eq!(insertion.state_before, insertion.state_after);
    assert_eq!(
        insertion.state_before.form,
        readux_nlp::morphology::ConnectionForm::Terminal
    );
    assert_eq!(insertion.source_morpheme_range, [4, 5]);
    for kind in [
        "desire",
        "negative",
        "past",
        "attributive_na",
        "nominalization",
        "copula",
    ] {
        assert!(owner.operators.iter().any(|op| op.kind == kind), "{kind}");
    }
    assert!(owner.terminal);
}
