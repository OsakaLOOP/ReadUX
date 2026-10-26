use readux_nlp::{
    model::{FeatureField, MorphemeToken, ProviderToken, FIELD_LABELS, FIELD_NAMES},
    morphology::{self, ConnectionForm, MorphologyArtifact},
    morphology_config::{self as config, CONFIG},
};

#[derive(Clone, Copy)]
struct Token<'a> {
    surface: &'a str,
    pos: &'a str,
    sub: &'a str,
    ctype: &'a str,
    cform: &'a str,
    lemma: &'a str,
}

fn noun(surface: &str) -> Token<'_> {
    Token {
        surface,
        pos: "名詞",
        sub: "普通名詞",
        ctype: "",
        cform: "",
        lemma: surface,
    }
}

fn verb(surface: &'static str, cform: &'static str) -> Token<'static> {
    Token {
        surface,
        pos: "動詞",
        sub: "一般",
        ctype: "五段-マ行",
        cform,
        lemma: "読む",
    }
}

fn auxiliary<'a>(surface: &'a str, lemma: &'a str, ctype: &'a str, cform: &'a str) -> Token<'a> {
    Token {
        surface,
        pos: "助動詞",
        sub: "",
        ctype,
        cform,
        lemma,
    }
}

fn collect(specs: &[Token<'_>]) -> MorphologyArtifact {
    let mut source = Vec::new();
    let mut morphemes = Vec::new();
    let mut cursor = 0;
    for (index, token) in specs.iter().enumerate() {
        let start = cursor;
        cursor += token.surface.chars().count();
        let mut values = vec![None; FIELD_NAMES.len()];
        for (name, value) in [
            ("pos1", token.pos),
            ("pos2", token.sub),
            ("cType", token.ctype),
            ("cForm", token.cform),
            ("lemma", token.lemma),
            ("orth", token.surface),
            ("orthBase", token.lemma),
        ] {
            values[FIELD_NAMES.iter().position(|field| *field == name).unwrap()] =
                Some(value.to_owned());
        }
        source.push(ProviderToken {
            index,
            surface: token.surface.into(),
            char_range: [start, cursor],
            byte_range: [0, 0],
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
                    name: FIELD_NAMES[index].into(),
                    label: FIELD_LABELS[index].into(),
                    raw: value.clone(),
                    value,
                })
                .collect(),
        });
        morphemes.push(MorphemeToken {
            id: format!("m{index}"),
            source_index: index,
            surface: token.surface.into(),
            char_range: [start, cursor],
            pos: [Some(token.pos.into()), Some(token.sub.into()), None, None],
            lemma: Some(token.lemma.into()),
            reading: None,
            query_forms: Vec::new(),
        });
    }
    morphology::collect_with_external(&source, &morphemes, &[]).unwrap()
}

#[test]
fn judgement_stems_and_endings_form_complete_chains() {
    let cases = [
        (
            verb("読み", "連用形-一般"),
            auxiliary("そう", "そう", "助動詞-ソウダ", "語幹"),
            "sou_modality",
        ),
        (verb("読む", "連体形-一般"), noun("よう"), "you_modality"),
        (
            noun("子ども"),
            auxiliary("みたい", "みたい", "助動詞-ミタイダ", "語幹"),
            "mitai_modality",
        ),
        (
            verb("読む", "終止形-一般"),
            auxiliary("べき", "べし", "助動詞-ベシ", "連体形-一般"),
            "obligation",
        ),
        (verb("読む", "連体形-一般"), noun("はず"), "hazu_modality"),
    ];
    let endings = [
        (
            vec![auxiliary("だ", "だ", "助動詞-ダ", "終止形-一般")],
            "copula",
            ConnectionForm::Terminal,
        ),
        (
            vec![
                auxiliary("だっ", "だ", "助動詞-ダ", "連用形-促音便"),
                auxiliary("た", "た", "助動詞-タ", "終止形-一般"),
            ],
            "copula_past",
            ConnectionForm::Terminal,
        ),
        (
            vec![
                auxiliary("でし", "です", "助動詞-デス", "連用形-一般"),
                auxiliary("た", "た", "助動詞-タ", "終止形-一般"),
            ],
            "copula_past",
            ConnectionForm::Terminal,
        ),
        (
            vec![auxiliary("で", "だ", "助動詞-ダ", "連用形-一般")],
            "copula_de",
            ConnectionForm::Continuative,
        ),
        (
            vec![auxiliary("な", "だ", "助動詞-ダ", "連体形-一般")],
            "attributive_na",
            ConnectionForm::Attributive,
        ),
        (
            vec![auxiliary("に", "だ", "助動詞-ダ", "連用形-一般")],
            "adverbial_ni",
            ConnectionForm::Continuative,
        ),
    ];
    for (root, stem, kind) in cases {
        for (ending, ending_kind, form) in &endings {
            if kind == "hazu_modality" && matches!(*ending_kind, "attributive_na" | "adverbial_ni")
            {
                continue;
            }
            let mut specs = vec![root, stem];
            specs.extend(ending);
            let artifact = collect(&specs);
            assert!(
                artifact.diagnostics.is_empty(),
                "{kind}/{ending_kind}: {:?}",
                artifact.diagnostics
            );
            assert_eq!(artifact.chains.len(), 1, "{kind}/{ending_kind}");
            let chain = &artifact.chains[0];
            assert_eq!(chain.status, "resolved", "{kind}/{ending_kind}");
            assert_eq!(chain.final_state.form, *form);
            assert_eq!(chain.final_state.entity_class, "ナ形容词判断类");
            assert!(chain.terminal);
            let stem_op = chain.operators.iter().find(|op| op.kind == kind).unwrap();
            assert_eq!(stem_op.state_after.form, ConnectionForm::Stem);
            assert_eq!(
                stem_op.state_after.category,
                if kind == "hazu_modality" {
                    "名詞"
                } else {
                    "形状詞"
                }
            );
            let tail = chain
                .operators
                .iter()
                .find(|op| op.kind == *ending_kind)
                .unwrap();
            assert_eq!(tail.source_morpheme_range, [2, specs.len()]);
            assert_eq!(tail.state_before.form, ConnectionForm::Stem);
            assert_eq!(tail.state_after.form, *form);
            if *ending_kind == "copula_past" {
                assert_eq!(
                    chain
                        .operators
                        .iter()
                        .filter(|op| matches!(op.kind.as_str(), "past" | "copula"))
                        .count(),
                    0
                );
            }
        }
    }
}

#[test]
fn judgement_continuations_and_lexical_boundaries_follow_token_identity() {
    let shape = Token {
        surface: "静か",
        pos: "形状詞",
        sub: "一般",
        ctype: "",
        cform: "",
        lemma: "静か",
    };
    let past = auxiliary("た", "た", "助動詞-タ", "終止形-一般");
    for specs in [
        vec![
            shape,
            auxiliary("で", "だ", "助動詞-ダ", "連用形-一般"),
            Token {
                surface: "あっ",
                pos: "動詞",
                sub: "非自立可能",
                ctype: "五段-ラ行",
                cform: "連用形-促音便",
                lemma: "ある",
            },
            past,
        ],
        vec![
            shape,
            auxiliary("で", "だ", "助動詞-ダ", "連用形-一般"),
            auxiliary("ない", "ない", "助動詞-ナイ", "終止形-一般"),
        ],
        vec![
            verb("読む", "終止形-一般"),
            auxiliary("らしかっ", "らしい", "助動詞-ラシイ", "連用形-促音便"),
            past,
        ],
        vec![
            noun("春"),
            Token {
                surface: "らしかっ",
                pos: "接尾辞",
                sub: "形容詞的",
                ctype: "形容詞",
                cform: "連用形-促音便",
                lemma: "らしい",
            },
            past,
        ],
    ] {
        let artifact = collect(&specs);
        assert!(
            artifact.diagnostics.is_empty(),
            "{:?}",
            artifact.diagnostics
        );
        assert_eq!(artifact.chains.len(), 1);
        assert_eq!(artifact.chains[0].status, "resolved");
        assert_eq!(
            artifact.chains[0].final_state.form,
            ConnectionForm::Terminal
        );
    }
    let nominal_link = Token {
        surface: "の",
        pos: "助詞",
        sub: "格助詞",
        ctype: "",
        cform: "",
        lemma: "の",
    };
    for (root, head, kind) in [
        (noun("夢"), noun("よう"), "you_modality"),
        (noun("彼"), noun("はず"), "hazu_modality"),
    ] {
        let artifact = collect(&[
            root,
            nominal_link,
            head,
            auxiliary("だ", "だ", "助動詞-ダ", "終止形-一般"),
        ]);
        assert!(
            artifact.diagnostics.is_empty(),
            "{:?}",
            artifact.diagnostics
        );
        assert_eq!(artifact.chains.len(), 1);
        assert!(artifact.chains[0]
            .operators
            .iter()
            .any(|op| op.kind == kind && op.source_morpheme_range == [1, 3]));
    }
    for lexical in [
        noun("はず"),
        noun("よう"),
        Token {
            surface: "そう",
            pos: "副詞",
            sub: "一般",
            ctype: "",
            cform: "",
            lemma: "そう",
        },
    ] {
        let artifact = collect(&[lexical]);
        assert!(!artifact
            .chains
            .iter()
            .flat_map(|chain| &chain.operators)
            .any(|op| op.executor == "na_adjective_judgement"));
    }
    for (surface, sub) in [("し", "接続助詞"), ("とか", "副助詞")] {
        let modern = collect(&[
            noun("夢"),
            auxiliary("です", "です", "助動詞-デス", "終止形-一般"),
            Token {
                surface,
                pos: "助詞",
                sub,
                ctype: "",
                cform: "",
                lemma: surface,
            },
        ]);
        assert!(modern
            .chains
            .iter()
            .flat_map(|chain| &chain.operators)
            .any(|op| op.kind == "politeness_desu"));
        assert!(!modern
            .chains
            .iter()
            .flat_map(|chain| &chain.operators)
            .any(|op| op.executor == "classical_conjugation"));
    }
    let appearance = collect(&[
        verb("読み", "連用形-一般"),
        auxiliary("そう", "そう", "助動詞-ソウダ", "語幹"),
    ]);
    let hearsay = collect(&[
        verb("読む", "終止形-一般"),
        auxiliary("そう", "そう", "助動詞-ソウダ", "語幹"),
    ]);
    assert!(appearance.chains[0]
        .evidence
        .iter()
        .any(|item| item == "judgement_usage:appearance"));
    assert!(hearsay.chains[0]
        .evidence
        .iter()
        .any(|item| item == "judgement_usage:hearsay"));
    let incomplete = collect(&[shape, auxiliary("だっ", "だ", "助動詞-ダ", "連用形-促音便")]);
    assert!(!incomplete
        .chains
        .iter()
        .any(|chain| chain.operators.iter().any(|op| op.kind == "copula_past")));
}

#[test]
fn every_declared_classical_auxiliary_form_executes_through_ordinary_operators() {
    // 覆盖所有声明词形、每组接续条件和接续允许的活用形，并检查完整运行产物。
    for entry in &CONFIG.classical_auxiliaries {
        for input in &entry.inputs {
            for input_form in &input.forms {
                let (surface, cform) = match input_form {
                    ConnectionForm::Stem => ("夢", ""),
                    ConnectionForm::Irrealis => ("行か", "未然形-一般"),
                    ConnectionForm::Continuative => ("行き", "連用形-一般"),
                    ConnectionForm::Terminal => ("行く", "終止形-一般"),
                    ConnectionForm::Attributive => ("行く", "連体形-一般"),
                    ConnectionForm::Conditional => ("行け", "已然形-一般"),
                    _ => panic!("文言接续表出现未支持的验收输入"),
                };
                let category = &input.categories[0];
                let ctype = input.ctype.first().map(String::as_str).unwrap_or_else(|| {
                    if input
                        .exclude_ctype
                        .iter()
                        .any(|kind| "文語四段-カ行".starts_with(kind))
                    {
                        "文語下二段-カ行"
                    } else {
                        "文語四段-カ行"
                    }
                });
                let root = Token {
                    surface,
                    pos: if category == "文語用言" {
                        "動詞"
                    } else {
                        category
                    },
                    sub: "一般",
                    ctype: if *input_form == ConnectionForm::Stem {
                        ""
                    } else {
                        ctype
                    },
                    cform,
                    lemma: if *input_form == ConnectionForm::Stem {
                        "夢"
                    } else {
                        "行く"
                    },
                };
                for (form, surfaces) in &entry.forms {
                    for suffix in surfaces {
                        let tail = auxiliary(suffix, &entry.lemma, &entry.ctype[0], form);
                        let artifact = collect(&[root, tail]);
                        assert!(
                            artifact.diagnostics.is_empty(),
                            "{} {input_form:?}/{suffix}: {:?}",
                            entry.rule,
                            artifact.diagnostics
                        );
                        assert_eq!(artifact.chains.len(), 1, "{} {suffix}", entry.rule);
                        let chain = &artifact.chains[0];
                        let op = chain
                            .operators
                            .iter()
                            .find(|op| op.kind == entry.rule)
                            .unwrap_or_else(|| {
                                panic!("{} {suffix}: {:?}", entry.rule, chain.operators)
                            });
                        assert_eq!(op.concept_id, CONFIG.operators[&entry.rule].concept);
                        assert_eq!(op.executor, "classical_conjugation");
                        assert_eq!(op.source_morpheme_range, [1, 2]);
                        assert_eq!(op.state_before.form, *input_form);
                        assert_eq!(
                            op.state_after.form,
                            config::classical_connection_form(form).unwrap()
                        );
                        assert_eq!(chain.final_state.category, "文語用言");
                        assert_eq!(chain.final_state.entity_class, "文言活用类");
                        assert_eq!(chain.status, "resolved");
                        assert!(op
                            .evidence
                            .iter()
                            .any(|item| item.starts_with("classical_paradigm:")));
                        assert!(op
                            .evidence
                            .iter()
                            .any(|item| item.starts_with("classical_input:")));
                    }
                }
            }
        }
    }
}

#[test]
fn classical_paradigms_cover_all_forms_and_generalize_to_other_lexemes() {
    for entry in &CONFIG.classical_forms {
        let stem = entry.lemma.strip_suffix(&entry.lemma_suffix).unwrap();
        for (form, endings) in &entry.forms {
            for ending in endings {
                let surface = format!("{stem}{ending}");
                let artifact = collect(&[Token {
                    surface: &surface,
                    pos: "形容詞",
                    sub: "一般",
                    ctype: &entry.ctype[0],
                    cform: form,
                    lemma: &entry.lemma,
                }]);
                let chain = &artifact.chains[0];
                assert_eq!(
                    chain.final_state.form,
                    config::classical_connection_form(form).unwrap()
                );
                assert_eq!(chain.final_state.entity_class, "文言活用类");
                assert!(
                    chain
                        .evidence
                        .iter()
                        .any(|item| item.starts_with("classical_paradigm:")),
                    "{surface}"
                );
            }
        }
    }
    for (surface, lemma, ctype) in [
        ("高き", "高し", "文語形容詞-ク"),
        ("悲しき", "悲し", "文語形容詞-シク"),
        ("穏やかなる", "穏やかなり", "文語形状詞-ナリ"),
        ("厳然たる", "厳然たり", "文語形状詞-タリ"),
    ] {
        let artifact = collect(&[Token {
            surface,
            lemma,
            ctype,
            cform: "連体形-一般",
            sub: "一般",
            pos: "形容詞",
        }]);
        assert_eq!(
            artifact.chains[0].final_state.entity_class, "文言活用类",
            "{surface}"
        );
    }
    let modern = collect(&[Token {
        surface: "高く",
        lemma: "高い",
        ctype: "形容詞",
        cform: "連用形-一般",
        pos: "形容詞",
        sub: "一般",
    }]);
    assert_eq!(modern.chains[0].final_state.entity_class, "イ形容词转换类");
}

#[test]
fn classical_chaining_and_modern_boundaries_use_form_and_context() {
    let classical_root = Token {
        surface: "書か",
        pos: "動詞",
        sub: "一般",
        ctype: "文語四段-カ行",
        cform: "未然形-一般",
        lemma: "書く",
    };
    let chain = collect(&[
        classical_root,
        auxiliary("れ", "る", "助動詞-ル", "連用形-一般"),
        auxiliary("けり", "けり", "助動詞-ケリ", "終止形-一般"),
    ]);
    assert!(chain.diagnostics.is_empty(), "{:?}", chain.diagnostics);
    assert_eq!(chain.chains.len(), 1);
    assert!(chain.chains[0].terminal);
    for (suffix, form) in [
        ("べし", "終止形-一般"),
        ("べく", "連用形-一般"),
        ("べき", "連体形-一般"),
    ] {
        let artifact = collect(&[
            verb("読む", "終止形-一般"),
            auxiliary(suffix, "べし", "助動詞-ベシ", form),
        ]);
        assert!(artifact.chains[0]
            .operators
            .iter()
            .any(|op| op.kind == "classical_beshi" && op.concept_id == "grammar.classical.beshi"));
        assert!(!artifact.chains[0]
            .operators
            .iter()
            .any(|op| op.kind == "obligation"));
    }
    let link = Token {
        surface: "の",
        pos: "助詞",
        sub: "格助詞",
        ctype: "",
        cform: "",
        lemma: "の",
    };
    for (head, kind, class) in [
        (
            auxiliary("ごとき", "ごとし", "助動詞-ゴトシ", "連体形-一般"),
            "classical_gotoshi",
            "文言活用类",
        ),
        (noun("よう"), "you_modality", "ナ形容词判断类"),
    ] {
        let artifact = collect(&[noun("夢"), link, head]);
        assert!(
            artifact.diagnostics.is_empty(),
            "{:?}",
            artifact.diagnostics
        );
        assert_eq!(artifact.chains.len(), 1);
        assert_eq!(artifact.chains[0].final_state.entity_class, class);
        assert!(artifact.chains[0]
            .operators
            .iter()
            .any(|op| op.kind == kind));
    }
    // り仅接受サ变未然形和四段已然形，四段未然形应保留诊断。
    let invalid = collect(&[
        classical_root,
        auxiliary("り", "り", "助動詞-リ", "終止形-一般"),
    ]);
    assert_eq!(invalid.chains[0].status, "candidate");
    assert!(invalid
        .diagnostics
        .iter()
        .any(|item| item.starts_with("classical_input_state:")));
    let zu = collect(&[
        verb("読ま", "未然形-一般"),
        auxiliary("ず", "ず", "助動詞-ヌ", "連用形-一般"),
        Token {
            surface: "に",
            pos: "助詞",
            sub: "格助詞",
            ctype: "",
            cform: "",
            lemma: "に",
        },
    ]);
    assert!(zu.diagnostics.is_empty(), "{:?}", zu.diagnostics);
    assert_eq!(zu.chains.len(), 1);
    assert_eq!(zu.chains[0].final_state.form, ConnectionForm::Other);
    assert!(zu.chains[0].terminal);
}

#[test]
fn real_classical_gotokunari_composes_gotoshi_and_nari() {
    let artifact = collect(&[
        auxiliary("ごとく", "ごとし", "文語助動詞-ゴトシ", "連用形-一般"),
        auxiliary("なり", "なり", "文語助動詞-ナリ-断定", "終止形-一般"),
    ]);
    assert!(
        artifact.diagnostics.is_empty(),
        "{:?}",
        artifact.diagnostics
    );
    assert_eq!(artifact.chains.len(), 1);
    let chain = &artifact.chains[0];
    assert_eq!(chain.surface_form, "ごとくなり");
    assert_eq!(chain.status, "resolved");
    assert_eq!(chain.final_state.entity_class, "文言活用类");
    assert!(chain
        .operators
        .iter()
        .any(|operator| operator.kind == "classical_gotoshi"));
    assert!(chain
        .operators
        .iter()
        .any(|operator| operator.kind == "classical_gotokunari"));
    assert!(chain
        .evidence
        .iter()
        .any(|item| item == "classical_composition:ごとし+なり"));
}

#[test]
fn real_classical_input_forms_cover_modern_unidic_boundaries() {
    let cases = [
        (
            "見ざるなり",
            vec![
                Token {
                    surface: "見",
                    pos: "動詞",
                    sub: "一般",
                    ctype: "上一段-マ行",
                    cform: "連用形-一般",
                    lemma: "見る",
                },
                auxiliary("ざる", "ず", "文語助動詞-ズ", "連体形-補助"),
                auxiliary("なり", "なり", "文語助動詞-ナリ-断定", "終止形-一般"),
            ],
            &["negative_zuni", "classical_nari_copula"][..],
        ),
        (
            "静かなりけり",
            vec![
                Token {
                    surface: "静か",
                    pos: "形状詞",
                    sub: "",
                    ctype: "",
                    cform: "",
                    lemma: "静か",
                },
                auxiliary("なり", "なり", "文語助動詞-ナリ-断定", "終止形-一般"),
                auxiliary("けり", "けり", "文語助動詞-ケリ", "終止形-一般"),
            ],
            &["classical_nari_copula", "classical_keri"][..],
        ),
        (
            "来まじ",
            vec![
                Token {
                    surface: "来",
                    pos: "動詞",
                    sub: "一般",
                    ctype: "カ行変格",
                    cform: "連用形-一般",
                    lemma: "来る",
                },
                auxiliary("まじ", "まじ", "文語助動詞-マジ", "終止形-一般"),
            ],
            &["classical_maji"][..],
        ),
        (
            "行かざるべからず",
            vec![
                Token {
                    surface: "行か",
                    pos: "動詞",
                    sub: "一般",
                    ctype: "五段-カ行",
                    cform: "未然形-一般",
                    lemma: "行く",
                },
                auxiliary("ざる", "ず", "文語助動詞-ズ", "連体形-補助"),
                auxiliary("べから", "べし", "文語助動詞-ベシ", "未然形-補助"),
                auxiliary("ず", "ず", "助動詞-ヌ", "終止形-一般"),
            ],
            &["negative_zuni", "classical_beshi", "negative_zuni"][..],
        ),
    ];
    for (text, specs, expected) in cases {
        let artifact = collect(&specs);
        assert!(
            artifact.diagnostics.is_empty(),
            "{text}: {:?}",
            artifact.diagnostics
        );
        assert_eq!(artifact.chains.len(), 1, "{text}");
        assert_eq!(artifact.chains[0].status, "resolved", "{text}");
        for kind in expected {
            assert!(
                artifact.chains[0]
                    .operators
                    .iter()
                    .any(|operator| operator.kind == *kind),
                "{text}: missing {kind}"
            );
        }
    }
}
