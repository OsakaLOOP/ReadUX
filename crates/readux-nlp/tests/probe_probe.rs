use readux_nlp::model::{
    FeatureField, MorphemeToken, ProviderToken, QueryForm, FIELD_LABELS, FIELD_NAMES,
};

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

fn run(text: &str, specs: &[Part]) -> readux_nlp::morphology::MorphologyArtifact {
    let (source, morphemes) = parts(text, specs);
    readux_nlp::morphology::collect_with_external(&source, &morphemes, &Vec::new()).unwrap()
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
    Part {
        surface: "で",
        pos: "助詞",
        sub: "接続助詞",
        ctype: "",
        cform: "",
        lemma: "で",
        orth_base: "で",
    }
}
fn aux(surface: &'static str, cform: &'static str) -> Part {
    Part {
        surface,
        pos: "動詞",
        sub: "非自立可能",
        ctype: "上一段-ア行",
        cform,
        lemma: "居る",
        orth_base: "居る",
    }
}
fn auxv(
    surface: &'static str,
    lemma: &'static str,
    ctype: &'static str,
    cform: &'static str,
) -> Part {
    Part {
        surface,
        pos: "動詞",
        sub: "非自立可能",
        ctype,
        cform,
        lemma,
        orth_base: lemma,
    }
}
fn adj(surface: &'static str, lemma: &'static str) -> Part {
    Part {
        surface,
        pos: "形状詞",
        sub: "",
        ctype: "形状詞-ナ型",
        cform: "",
        lemma,
        orth_base: lemma,
    }
}
fn auxd(
    surface: &'static str,
    ctype: &'static str,
    cform: &'static str,
    lemma: &'static str,
) -> Part {
    Part {
        surface,
        pos: "助動詞",
        sub: "",
        ctype,
        cform,
        lemma,
        orth_base: lemma,
    }
}

#[test]
fn probe() {
    for (text, specs) in [
        (
            "読んでいない",
            vec![
                verb("読ん", "連用形-撥音便"),
                te(),
                aux("い", "未然形-一般"),
                auxd("ない", "助動詞-ナイ", "基本形", "ない"),
            ],
        ),
        (
            "読んでいます",
            vec![
                verb("読ん", "連用形-撥音便"),
                te(),
                aux("い", "連用形-一般"),
                auxd("ます", "助動詞-マス", "基本形", "ます"),
            ],
        ),
        (
            "読んでおいてみる",
            vec![
                verb("読ん", "連用形-撥音便"),
                te(),
                auxv("おい", "置く", "五段-ワ行", "連用形-一般"),
                Part {
                    surface: "て",
                    pos: "助詞",
                    sub: "接続助詞",
                    ctype: "",
                    cform: "",
                    lemma: "て",
                    orth_base: "て",
                },
                auxv("みる", "見る", "上一段-ア行", "終止形-一般"),
            ],
        ),
        (
            "読んだりする",
            vec![
                verb("読ん", "連用形-撥音便"),
                Part {
                    surface: "だり",
                    pos: "助動詞",
                    sub: "",
                    ctype: "助動詞-タ",
                    cform: "基本形-一般",
                    lemma: "だり",
                    orth_base: "だり",
                },
                auxv("する", "する", "サ変-スル", "終止形-一般"),
            ],
        ),
        (
            "静かだった",
            vec![
                adj("静か", "静か"),
                auxd("だっ", "助動詞-タ", "連用形-促音便", "だ"),
                auxd("た", "助動詞-タ", "基本形", "た"),
            ],
        ),
        (
            "静かではない",
            vec![
                adj("静か", "静か"),
                te(),
                auxd("は", "助動詞-デハ", "基本形", "は"),
                auxd("ない", "助動詞-ナイ", "基本形", "ない"),
            ],
        ),
    ] {
        let artifact = run(text, &specs);
        println!("=== {text}");
        for chain in &artifact.chains {
            println!(
                "  chain {} status={} role={:?} parent={:?} cat={} form={:?}",
                chain.surface_form,
                chain.status,
                chain.role,
                chain.parent_chain_id,
                chain.final_state.category,
                chain.final_state.form
            );
            for op in &chain.operators {
                println!(
                    "    op {} after={:?}/{:?}",
                    op.kind, op.state_after.category, op.state_after.form
                );
            }
        }
        println!("  diags: {:?}", artifact.diagnostics);
    }
}
