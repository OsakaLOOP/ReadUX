//! 实际 UniDic 活用与 GiNZA 信息变化的定向验收。
use readux_core::analysis::{AnalysisService, Request, ResourcePaths};
use readux_nlp::{
    model::{Register, UnifiedDocument},
    morphology::MorphologyRole,
};
use std::path::PathBuf;

fn analyze(service: &AnalysisService, text: &str) -> UnifiedDocument {
    let response = service.dispatch(Request::Analyze {
        text: text.into(),
        register: Register::Cwj,
    });
    assert!(response.error.is_none(), "{:?}", response.error);
    serde_json::from_value(response.result.unwrap()).unwrap()
}

#[test]
fn stage_eleven_real_tokens_have_unique_connection_paths() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../data/validation/morphology_stage11_cases.json"
    )).unwrap();
    let mut failures = Vec::new();
    for case in cases {
        let text = case["text"].as_str().unwrap();
        let document = analyze(&service, text);
        let ops: Vec<_> = document.morphology.chains.iter().flat_map(|c| &c.operators)
            .filter(|op| op.kind == case["kind"].as_str().unwrap()).collect();
        if ops.len() != 1 || ops[0].connection.as_ref().is_none_or(|c|
            c.rule != case["rule"] || c.path.as_deref() != case["path"].as_str()
                || c.matched_paths.len() != 1 || !c.diagnostics.is_empty()) {
            failures.push(format!("{text}: {ops:?}; {:?}", document.morphology.diagnostics));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn supplied_predicates_keep_constructions_and_local_boundaries() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for (text, required) in [
        (
            "もうファンになっちゃうしかなくない",
            vec!["ni_naru", "contracted_te_shimau", "shika_nai", "negative"],
        ),
        (
            "屋上から投げ出されそうになった",
            vec!["passive_potential", "sou_modality", "ni_naru", "past"],
        ),
        (
            "場所を作ろうとしているからな",
            vec![
                "volitional",
                "volitional_to_suru",
                "te_iru",
                "kara_connection",
            ],
        ),
        (
            "順番に達成していくことになる",
            vec!["te_iku", "koto_nominalization", "ni_naru"],
        ),
        (
            "　置いていかれないように集中した",
            vec![
                "te_iku",
                "passive_potential",
                "negative",
                "you_modality",
                "adverbial_ni",
                "past",
            ],
        ),
        (
            "考えなくていいんだから",
            vec![
                "negative",
                "te_ii",
                "nominalization",
                "copula",
                "kara_connection",
            ],
        ),
        (
            "舐めてかかっていただろう",
            vec!["te_kakaru", "te_iru", "past", "darou_modality"],
        ),
    ] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology.diagnostics.is_empty(),
            "{text}: {:?}",
            doc.morphology.diagnostics
        );
        assert!(
            doc.morphology
                .chains
                .iter()
                .all(|chain| chain.status == "resolved"),
            "{text}"
        );
        for kind in required {
            assert!(
                doc.morphology
                    .chains
                    .iter()
                    .flat_map(|chain| &chain.operators)
                    .any(|op| op.kind == kind),
                "{text}: 缺少 {kind}"
            );
        }
        let response = service.dispatch(Request::LookupTargets {
            analysis_id: doc.id.clone(),
            range: [0, text.chars().count()],
        });
        assert!(response.error.is_none(), "{text}: {:?}", response.error);
        let targets = response.result.unwrap();
        for op in doc
            .morphology
            .chains
            .iter()
            .flat_map(|chain| &chain.operators)
            .filter(|op| {
                matches!(
                    op.kind.as_str(),
                    "shika_nai"
                        | "volitional_to_suru"
                        | "koto_nominalization"
                        | "te_ii"
                        | "te_kakaru"
                )
            })
        {
            let entry = readux_core::grammar_catalog::get(&op.concept_id).unwrap();
            assert!(
                !entry.explanation.connection.is_empty(),
                "{text}: {}",
                op.kind
            );
            assert!(
                targets["grammar_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|target| target["concept_id"].as_str() == Some(op.concept_id.as_str())),
                "{text}: {}",
                op.kind
            );
        }
        if text == "舐めてかかっていただろう" || text == "考えなくていいんだから"
        {
            assert_eq!(
                targets["outer_targets"].as_array().unwrap().len(),
                1,
                "{text}"
            );
            let expected = if text.starts_with("舐め") {
                "舐める"
            } else {
                "考える"
            };
            assert_eq!(
                targets["outer_targets"][0]["lookup_forms"][0]["form"], expected,
                "{text}"
            );
        }
        for chain in &doc.morphology.chains {
            if let Some(parent) = &chain.parent_chain_id {
                let parent = doc
                    .morphology
                    .chains
                    .iter()
                    .find(|candidate| &candidate.chain_id == parent)
                    .unwrap();
                assert_eq!(parent.char_range[1], chain.char_range[0], "{text}");
            }
        }
    }
    for (text, kind, lookup, parent_lookup) in [
        (
            "場所を作ろうとしているからな",
            "volitional_to_suru",
            "する",
            "作る",
        ),
        ("考えなくていいんだから", "te_ii", "いい", "考える"),
        ("順番に達成していくことになる", "ni_naru", "なる", "いく"),
    ] {
        let doc = analyze(&service, text);
        let child = doc
            .morphology
            .chains
            .iter()
            .find(|chain| chain.operators.iter().any(|op| op.kind == kind))
            .unwrap();
        assert_eq!(child.lookup_form, lookup, "{text}");
        let parent = doc
            .morphology
            .chains
            .iter()
            .find(|chain| Some(&chain.chain_id) == child.parent_chain_id.as_ref())
            .unwrap();
        assert_eq!(parent.lookup_form, parent_lookup, "{text}");
        let occurrence = doc
            .morphology
            .occurrences
            .iter()
            .find(|item| item.kind == kind)
            .unwrap();
        assert!(occurrence.hit_ranges.len() >= 2, "{text}");
        assert!(
            occurrence.context_range[0] <= parent.char_range[0],
            "{text}"
        );
    }
    for (text, kind) in [
        ("作るとしている", "volitional_to_suru"),
        ("作ろうと思う", "volitional_to_suru"),
        ("作ろうと、する", "volitional_to_suru"),
        ("作ろうと する", "volitional_to_suru"),
        ("これしか読まない", "shika_nai"),
        ("読むしか、ない", "shika_nai"),
        ("読むしか ない", "shika_nai"),
        ("琴になる", "koto_nominalization"),
        ("読む、ことになる", "koto_nominalization"),
        ("読んで、いい", "te_ii"),
        ("読んで いい", "te_ii"),
    ] {
        let doc = analyze(&service, text);
        assert!(
            !doc.morphology
                .chains
                .iter()
                .flat_map(|chain| &chain.operators)
                .any(|op| op.kind == kind),
            "{text}"
        );
    }
    // しかない的否定归属与后续否定分别保留；名词化后的变化动词继续递归。
    for text in [
        "読むしかなかった",
        "読むしかなくなった",
        "帰ることになっちゃった",
        "読もうとしていなかった",
        "読まなくてよかった",
    ] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology.diagnostics.is_empty(),
            "{text}: {:?}",
            doc.morphology.diagnostics
        );
        assert!(
            doc.morphology
                .chains
                .iter()
                .all(|chain| chain.status == "resolved"),
            "{text}"
        );
    }
}

#[test]
fn attitude_and_humble_support_preserve_recursive_inflection() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for (text, kind, parent_form, lookup_form, tail) in [
        (
            "舐めてかかっていただろう",
            "te_kakaru",
            "舐める",
            "かかる",
            "darou_modality",
        ),
        (
            "教えていただいております",
            "te_itadaku",
            "教える",
            "いただく",
            "politeness_masu",
        ),
        ("教えて頂いた", "te_itadaku", "教える", "頂く", "past"),
        ("疑って掛かった", "te_kakaru", "疑う", "掛かる", "past"),
    ] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology.diagnostics.is_empty(),
            "{text}: {:?}",
            doc.morphology.diagnostics
        );
        let support = doc
            .morphology
            .chains
            .iter()
            .find(|chain| chain.operators.iter().any(|op| op.kind == kind))
            .unwrap();
        assert_eq!(support.lookup_form, lookup_form, "{text}");
        assert_eq!(support.role, MorphologyRole::Functional, "{text}");
        let parent = doc
            .morphology
            .chains
            .iter()
            .find(|chain| Some(&chain.chain_id) == support.parent_chain_id.as_ref())
            .unwrap();
        assert_eq!(parent.lookup_form, parent_form, "{text}");
        let last = doc.morphology.chains.last().unwrap();
        assert!(last.operators.iter().any(|op| op.kind == tail), "{text}");
        assert!(
            doc.morphology
                .chains
                .iter()
                .all(|chain| chain.status == "resolved"),
            "{text}"
        );
        if text.contains("かかって") || text.contains("いただいて") {
            assert_eq!(
                last.parent_chain_id.as_ref(),
                Some(&support.chain_id),
                "{text}"
            );
            assert!(
                last.operators.iter().any(|op| op.kind == "te_iru"),
                "{text}"
            );
        }
    }
    for text in [
        "時間がかかった",
        "屋根に掛かった",
        "本をいただいた",
        "舐めて、かかった",
        "教えて いただいた",
    ] {
        let doc = analyze(&service, text);
        assert!(
            !doc.morphology
                .chains
                .iter()
                .flat_map(|chain| &chain.operators)
                .any(|op| matches!(op.kind.as_str(), "te_kakaru" | "te_itadaku")),
            "{text}"
        );
    }
}

#[test]
fn conditional_toka_preserves_enumerative_and_predicate_connections() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for text in [
        "気合いが入っていたりとかしていない",
        "読んだりとかしなかった",
    ] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology.diagnostics.is_empty(),
            "{text}: {:?}",
            doc.morphology.diagnostics
        );
        let owner = doc
            .morphology
            .chains
            .iter()
            .find(|chain| {
                chain
                    .operators
                    .iter()
                    .any(|op| op.kind == "exemplification")
            })
            .unwrap();
        assert!(
            owner.surface_form.ends_with("たりとか") || owner.surface_form.ends_with("だりとか"),
            "{text}"
        );
        let insertion = owner
            .operators
            .iter()
            .find(|op| op.kind == "exemplification")
            .unwrap();
        assert_eq!(insertion.state_before, insertion.state_after);
        assert_eq!(
            insertion.state_before.form,
            readux_nlp::morphology::ConnectionForm::Continuative
        );
        let child = doc
            .morphology
            .chains
            .iter()
            .find(|chain| chain.operators.iter().any(|op| op.kind == "tari_suru"))
            .unwrap();
        assert_eq!(
            child.parent_chain_id.as_deref(),
            Some(owner.chain_id.as_str())
        );
        assert!(doc
            .morphology
            .chains
            .iter()
            .all(|chain| chain.status == "resolved"));
        assert!(doc
            .morphology
            .chains
            .iter()
            .any(|chain| chain.operators.iter().any(|op| op.kind == "negative")));
    }
    for (text, expected) in [("学生ですとか先生だとか", 2), ("本とか読む", 1)] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology.diagnostics.is_empty(),
            "{text}: {:?}",
            doc.morphology.diagnostics
        );
        let insertions: Vec<_> = doc
            .morphology
            .chains
            .iter()
            .flat_map(|chain| &chain.operators)
            .filter(|op| op.kind == "exemplification")
            .collect();
        assert_eq!(insertions.len(), expected, "{text}");
        assert!(insertions
            .iter()
            .all(|op| op.state_before == op.state_after));
    }
    for text in ["読みとかする", "読んでとかいる", "読んだりしているとかする"]
    {
        let doc = analyze(&service, text);
        // 拒绝将历史连接器重新用于后面的独立谓词。
        let last = doc.morphology.chains.last().unwrap();
        assert!(
            last.parent_chain_id.is_none(),
            "{text}: {:?}",
            doc.morphology.chains
        );
    }
    for text in ["読んだりとか、する", "読んだり とかする"] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology
                .chains
                .last()
                .unwrap()
                .parent_chain_id
                .is_none(),
            "{text}"
        );
    }
}

#[test]
fn recursive_predicates_use_local_inflection_and_preserve_nominal_inputs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for (text, kind, parent_surface, category) in [
        ("訪ねてきたりしないよ", "tari_suru", "きたり", "形容詞"),
        ("読んだりしなかった", "tari_suru", "読んだり", "形容詞"),
        ("読んだりできなかった", "tari_dekiru", "読んだり", "形容詞"),
        ("読んだりさせられた", "tari_suru", "読んだり", "動詞"),
        ("静かだったりしない", "tari_suru", "静かだったり", "形容詞"),
        ("高かったりできない", "tari_dekiru", "高かったり", "形容詞"),
        ("読んでいさせられた", "te_iru", "読んで", "動詞"),
        (
            "読んでいたくなくなった",
            "adjectival_naru",
            "いたくなく",
            "動詞",
        ),
        ("高くなくなってしまった", "te_shimau", "なって", "動詞"),
        (
            "キャンセルになるかもしれなくなってしまった",
            "te_shimau",
            "なって",
            "動詞",
        ),
        (
            "自分だけ先に帰ることになっちゃったとかだろうか",
            "contracted_te_shimau",
            "なっ",
            "動詞",
        ),
        ("代表になった", "ni_naru", "代表に", "動詞"),
        ("これになった", "ni_naru", "これに", "動詞"),
        ("強さになった", "ni_naru", "強さに", "動詞"),
        ("読むのになった", "ni_naru", "読むのに", "動詞"),
        ("静かになった", "ni_naru", "静かに", "動詞"),
        ("読めるようになった", "ni_naru", "読めるように", "動詞"),
    ] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology.diagnostics.is_empty(),
            "{text}: {:?}",
            doc.morphology.diagnostics
        );
        assert!(
            doc.morphology
                .chains
                .iter()
                .all(|chain| chain.status == "resolved"),
            "{text}"
        );
        let child = doc
            .morphology
            .chains
            .iter()
            .find(|chain| chain.operators.iter().any(|op| op.kind == kind))
            .unwrap_or_else(|| panic!("{text}: 缺少 {kind}"));
        let parent = doc
            .morphology
            .chains
            .iter()
            .find(|chain| Some(&chain.chain_id) == child.parent_chain_id.as_ref())
            .unwrap();
        assert_eq!(parent.surface_form, parent_surface, "{text}");
        assert_eq!(child.final_state.category, category, "{text}");
        let op = child.operators.iter().find(|op| op.kind == kind).unwrap();
        assert_eq!(op.state_after.category, "動詞", "{text}");
        assert_eq!(
            child.lookup_form,
            if kind == "tari_suru" {
                "する"
            } else if kind == "tari_dekiru" {
                "できる"
            } else if kind == "te_iru" {
                "いる"
            } else if matches!(kind, "te_shimau" | "contracted_te_shimau") {
                "しまう"
            } else {
                "なる"
            },
            "{text}"
        );
        if kind == "ni_naru" {
            assert!(matches!(
                parent.final_state.category.as_str(),
                "名詞" | "代名詞" | "形状詞"
            ));
        }
    }
    for (text, kind, category) in [
        ("読めるかもしれなかった", "kamoshirenai", "形容詞"),
        ("読んでほしくなかった", "te_hoshii", "形容詞"),
        ("読んじゃいたかった", "contracted_te_shimau", "形容詞"),
        ("読んだのだったらしい", "rashii_modality", "形容詞"),
        ("こうならないはずなのだから", "hazu_modality", "形状詞"),
        ("リサーチしつつ", "tsutsu_connection", "動詞"),
        ("支え合ったりできるような気がする", "tari_dekiru", "形状詞"),
    ] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology.diagnostics.is_empty(),
            "{text}: {:?}",
            doc.morphology.diagnostics
        );
        let chain = doc
            .morphology
            .chains
            .iter()
            .find(|chain| chain.operators.iter().any(|op| op.kind == kind))
            .unwrap();
        assert_eq!(chain.status, "resolved", "{text}");
        assert_eq!(chain.final_state.category, category, "{text}");
    }
    for text in [
        "読みましたい",
        "読みまします",
        "読まなくたい",
        "読まないたい",
    ] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology
                .chains
                .iter()
                .any(|chain| chain.status != "resolved"),
            "{text}"
        );
    }
    let ordinary_case = analyze(&service, "机に置いた");
    assert!(ordinary_case
        .morphology
        .chains
        .iter()
        .all(|chain| chain.parent_chain_id.is_none()));
    let ordinary_noni = analyze(&service, "読むのに時間がかかる");
    assert!(ordinary_noni.morphology.chains[0]
        .operators
        .iter()
        .any(|op| op.kind == "noni_connection"));
}

#[test]
fn modern_inflections_and_functional_ownership() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for (text, surface, base, kinds) in [
        (
            "行かせられなかった。",
            "行かせられなかった",
            "行く",
            vec!["causative", "passive_potential", "negative", "past"],
        ),
        (
            "読みませんでした。",
            "読みませんでした",
            "読む",
            vec!["polite_negative", "politeness_desu", "past"],
        ),
        (
            "食べたかった。",
            "食べたかった",
            "食べる",
            vec!["desire", "past"],
        ),
        (
            "来なかった。",
            "来なかった",
            "来る",
            vec!["negative", "past"],
        ),
        ("成立した。", "成立した", "成立する", vec!["past"]),
        (
            "高くなかった。",
            "高くなかった",
            "高い",
            vec!["negative", "past"],
        ),
        ("静かな町だ。", "静かな", "静か", vec!["attributive_na"]),
        ("行くな。", "行くな", "行く", vec!["prohibitive"]),
        ("読め。", "読め", "読む", vec!["imperative"]),
        ("読もう。", "読もう", "読む", vec!["volitional"]),
        (
            "読めば分かる。",
            "読めば",
            "読む",
            vec!["conditional", "ba_connection"],
        ),
        (
            "発展せしめる。",
            "発展せしめる",
            "発展する",
            vec!["causative"],
        ),
        (
            "本である。",
            "本である",
            "本",
            vec!["copula_de", "copula_aru"],
        ),
        (
            "降るらしい。",
            "降るらしい",
            "降る",
            vec!["rashii_modality"],
        ),
        (
            "読めそうだ。",
            "読めそうだ",
            "読める",
            vec!["sou_modality", "copula"],
        ),
        (
            "行くまい。",
            "行くまい",
            "行く",
            vec!["negative_volitional"],
        ),
        ("読みやすい。", "読みやすい", "読みやすい", vec!["ease"]),
        (
            "高くはない。",
            "高くはない",
            "高い",
            vec!["insertion", "negative"],
        ),
        (
            "読むだけだ。",
            "読むだけだ",
            "読む",
            vec!["insertion", "copula"],
        ),
        (
            "気付きたくなかっただけなのだ。",
            "気付きたくなかっただけなのだ",
            "気付く",
            vec![
                "desire",
                "negative",
                "past",
                "insertion",
                "attributive_na",
                "nominalization",
                "copula",
            ],
        ),
    ] {
        let doc = analyze(&service, text);
        let chain = doc
            .morphology
            .chains
            .iter()
            .find(|c| c.surface_form == surface)
            .unwrap_or_else(|| {
                panic!(
                    "{text}: {:?}",
                    doc.morphology
                        .chains
                        .iter()
                        .map(|c| (&c.surface_form, &c.final_state))
                        .collect::<Vec<_>>()
                )
            });
        assert_eq!(chain.dictionary_form, base, "{text}");
        for kind in kinds {
            assert!(
                chain.operators.iter().any(|o| o.kind == kind),
                "{text} 缺少 {kind}：{:?}；状态 {:?}；诊断 {:?}",
                chain
                    .operators
                    .iter()
                    .map(|operator| &operator.kind)
                    .collect::<Vec<_>>(),
                chain.final_state,
                doc.morphology.diagnostics
            );
        }
        let expected_concepts = match text {
            "降るらしい。" => [("rashii_modality", "grammar.modality.rashii")].as_slice(),
            "読めそうだ。" => [("sou_modality", "grammar.modality.sou")].as_slice(),
            _ => &[] as &[(&str, &str)],
        };
        for (kind, concept) in expected_concepts {
            let operator = chain
                .operators
                .iter()
                .find(|operator| operator.kind == *kind)
                .unwrap();
            assert_eq!(operator.concept_id, *concept, "{text}: {kind}");
            assert!(
                readux_core::grammar_catalog::get(concept).is_ok(),
                "{text}: {concept}"
            );
        }
        for op in &chain.operators {
            assert!(
                readux_core::grammar_catalog::get(&op.concept_id).is_ok(),
                "目录引用无效：{}",
                op.concept_id
            );
        }
        assert!(
            chain.query_forms.iter().any(|q| q.form == base),
            "{text} 缺少基本形查询"
        );
    }
    for text in [
        "読んでくださった。",
        "専有してしまっている。",
        "読んでいる。",
        "覗かれてる。",
        "読んじゃった。",
        "読んどく。",
    ] {
        let doc = analyze(&service, text);
        let functional: Vec<_> = doc
            .morphology
            .chains
            .iter()
            .filter(|c| c.parent_chain_id.is_some())
            .collect();
        assert!(
            !functional.is_empty(),
            "{text}: {:?}",
            doc.morphology
                .chains
                .iter()
                .map(|c| &c.surface_form)
                .collect::<Vec<_>>()
        );
        for chain in functional {
            assert_eq!(chain.role, MorphologyRole::Functional);
            assert!(doc
                .morphology
                .chains
                .iter()
                .any(|p| Some(&p.chain_id) == chain.parent_chain_id.as_ref()));
        }
    }
    for text in ["そこにいる。", "読んで いる。", "読んで。いる。"] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology
                .chains
                .iter()
                .all(|c| c.parent_chain_id.is_none()),
            "{text}"
        );
    }
    for (text, base, expanded) in [
        ("覗かれてる。", "いる", "ている"),
        ("読んでる。", "いる", "ている"),
        ("食べちゃった。", "しまう", "てしまう"),
        ("読んじゃった。", "しまう", "てしまう"),
        ("食べとく。", "おく", "ておく"),
        ("読んどく。", "おく", "ておく"),
    ] {
        let doc = analyze(&service, text);
        let chain = doc
            .morphology
            .chains
            .iter()
            .find(|c| c.parent_chain_id.is_some())
            .unwrap();
        assert_eq!(chain.lookup_form, base);
        assert!(chain
            .query_forms
            .iter()
            .any(|q| q.kind == "expanded_auxiliary" && q.form == base));
        assert_eq!(chain.display_form, expanded, "{text}");
        assert!(
            chain
                .operators
                .iter()
                .any(|o| o.normalized_form.as_deref() == Some(expanded)),
            "{text}"
        );
    }
    let doc = analyze(&service, "読んでいる。");
    let occurrence = doc
        .morphology
        .occurrences
        .iter()
        .find(|o| o.kind == "te_iru")
        .unwrap();
    assert_eq!(occurrence.char_range, [2, 5]);
    assert_eq!(occurrence.hit_ranges, vec![[2, 3], [3, 5]]);
}

#[test]
fn stage_nine_operators_resolve_formal_concepts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for (text, kind, concept) in [
        ("行きたがる。", "desire_outward", "grammar.modality.tagaru"),
        ("学生である。", "copula_aru", "grammar.functional.aru"),
        ("静かさを測る。", "nominalization", "grammar.functional.sa"),
        (
            "食べたら寝る。",
            "tara_condition",
            "morphology.mood.conditional",
        ),
        (
            "春なら出かける。",
            "nara_condition",
            "morphology.mood.conditional",
        ),
        (
            "歩きながら話す。",
            "nagara_connection",
            "grammar.connective.nagara",
        ),
        (
            "考えつつ進む。",
            "tsutsu_connection",
            "grammar.connective.tsutsu",
        ),
        (
            "雨なれど行く。",
            "concessive_connection",
            "grammar.connective.keredomo",
        ),
        (
            "ここで騒ぐな。",
            "prohibitive",
            "grammar.sentence_particle.na",
        ),
        (
            "本を読んだり聞いたりする。",
            "enumerative",
            "grammar.connective.tari",
        ),
        (
            "読むとか書くとかする。",
            "exemplification",
            "grammar.connective.toka",
        ),
        ("高いくらいだ。", "degree", "grammar.particle.kurai"),
        ("早く行け。", "imperative", "morphology.form.imperative"),
        (
            "静かなので帰る。",
            "node_connection",
            "grammar.connective.node",
        ),
        (
            "食べないで寝る。",
            "negative_de",
            "grammar.connective.naide",
        ),
        (
            "ここに入ってはならない。",
            "prohibition_tewa_condition_naru",
            "grammar.prohibition.tewa_naranai",
        ),
        (
            "ここに入ってはいけない。",
            "prohibition_tewa_condition_ikeru",
            "grammar.prohibition.tewa_ikenai",
        ),
        (
            "ここに入ってはだめだ。",
            "prohibition_tewa_condition_dame",
            "grammar.prohibition.tewa_dame",
        ),
        ("見てごらん。", "te_goran", "grammar.auxiliary.goran"),
        (
            "会いたくてたまらない。",
            "te_tamaranai",
            "grammar.auxiliary.tamaranai",
        ),
        (
            "忘れられなくてならない。",
            "te_naranai",
            "grammar.auxiliary.naranai",
        ),
        (
            "願ってやまない。",
            "te_yamanai",
            "grammar.auxiliary.yamanai",
        ),
    ] {
        let doc = analyze(&service, text);
        let chain = doc
            .morphology
            .chains
            .iter()
            .find(|chain| chain.operators.iter().any(|operator| operator.kind == kind))
            .unwrap_or_else(|| panic!("{text}: missing chain for {kind}"));
        assert_eq!(chain.status, "resolved", "{text}: {kind}");
        assert!(
            doc.morphology.diagnostics.is_empty(),
            "{text}: {kind}: {:?}",
            doc.morphology.diagnostics
        );
        let operator = doc
            .morphology
            .chains
            .iter()
            .flat_map(|chain| chain.operators.iter())
            .find(|operator| operator.kind == kind)
            .unwrap_or_else(|| panic!("{text}: missing {kind}"));
        assert_eq!(operator.concept_id, concept, "{text}: {kind}");
    }
}

#[test]
fn i_adjective_conversion_continues_through_polarity_and_tense() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for (text, surface, expected_kinds, expected_form) in [
        (
            "食べたくない。",
            "食べたくない",
            vec!["desire", "negative"],
            "形容詞",
        ),
        (
            "食べたかった。",
            "食べたかった",
            vec!["desire", "past"],
            "形容詞",
        ),
        (
            "行きたくなかった。",
            "行きたくなかった",
            vec!["desire", "negative", "past"],
            "形容詞",
        ),
        (
            "読みやすかった。",
            "読みやすかった",
            vec!["ease", "past"],
            "形容詞",
        ),
    ] {
        let doc = analyze(&service, text);
        let chain = doc
            .morphology
            .chains
            .iter()
            .find(|chain| chain.surface_form == surface)
            .unwrap_or_else(|| {
                panic!(
                    "{text}: {:?}",
                    doc.morphology
                        .chains
                        .iter()
                        .map(|chain| (&chain.surface_form, &chain.final_state))
                        .collect::<Vec<_>>()
                )
            });
        assert_eq!(chain.final_state.category, expected_form, "{text}");
        assert_eq!(chain.final_state.entity_class, "イ形容词转换类", "{text}");
        for kind in expected_kinds {
            let operator = chain
                .operators
                .iter()
                .find(|operator| operator.kind == kind)
                .unwrap_or_else(|| {
                    panic!(
                        "{text}: 缺少 {kind}: {:?}",
                        chain
                            .operators
                            .iter()
                            .map(|operator| &operator.kind)
                            .collect::<Vec<_>>()
                    )
                });
            assert_eq!(operator.state_after.category, "形容詞", "{text}: {kind}");
        }
    }
}

#[test]
fn yuku_forms_a_support_chain_after_te() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for text in [
        "散布されてゆく。",
        "代わられてゆく。",
        "広がってゆきました。",
        "読んでゆく。",
    ] {
        let doc = analyze(&service, text);
        let chain = doc
            .morphology
            .chains
            .iter()
            .find(|chain| chain.operators.iter().any(|op| op.kind == "te_yuku"))
            .unwrap();
        let parent = doc
            .morphology
            .chains
            .iter()
            .find(|parent| Some(&parent.chain_id) == chain.parent_chain_id.as_ref())
            .unwrap();
        assert_eq!(chain.role, MorphologyRole::Functional);
        assert_eq!(chain.lookup_form, "ゆく");
        assert_eq!(parent.char_range[1], chain.char_range[0]);
        let occurrence = doc
            .morphology
            .occurrences
            .iter()
            .find(|occurrence| occurrence.kind == "te_yuku")
            .unwrap();
        assert_eq!(occurrence.hit_ranges.len(), 2);
        assert_eq!(occurrence.context_range[0], parent.char_range[0]);
        let op = chain
            .operators
            .iter()
            .find(|op| op.kind == "te_yuku")
            .unwrap();
        assert_eq!(op.normalized_form.as_deref(), Some("てゆく"));
        assert!(readux_core::grammar_catalog::get(&op.concept_id).is_ok());
    }
    for text in ["学校へゆく。", "散布されて、ゆく。", "散布されて ゆく。"] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology
                .chains
                .iter()
                .all(|chain| !chain.operators.iter().any(|op| op.kind == "te_yuku")),
            "{text}"
        );
    }
}

#[test]
fn tara_and_dara_are_complete_conditional_forms() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for (text, ending) in [
        ("なれたら。", "たら"),
        ("食べたら。", "たら"),
        ("読んだら。", "だら"),
        ("静かだったら。", "たら"),
        ("なれたらいいな。", "たら"),
    ] {
        let doc = analyze(&service, text);
        let occurrence = doc
            .morphology
            .occurrences
            .iter()
            .find(|occurrence| occurrence.kind == "tara_condition")
            .unwrap();
        let surface: String = doc
            .text
            .chars()
            .skip(occurrence.char_range[0])
            .take(occurrence.char_range[1] - occurrence.char_range[0])
            .collect();
        assert_eq!(surface, ending);
        assert_eq!(occurrence.hit_ranges, vec![occurrence.char_range]);
        let operators = doc
            .morphology
            .chains
            .iter()
            .flat_map(|chain| &chain.operators)
            .filter(|op| op.char_range == occurrence.char_range)
            .collect::<Vec<_>>();
        assert_eq!(operators.len(), 1, "{text}");
        assert_eq!(operators[0].normalized_form.as_deref(), Some(ending));
    }
    for (text, kind) in [("読んだ。", "past"), ("読めば。", "ba_connection")] {
        let doc = analyze(&service, text);
        assert!(doc
            .morphology
            .occurrences
            .iter()
            .any(|occurrence| occurrence.kind == kind));
        assert!(!doc
            .morphology
            .occurrences
            .iter()
            .any(|occurrence| occurrence.kind == "tara_condition"));
    }
}

#[test]
fn sentence_final_no_is_outside_the_predicate_chain() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    for text in [
        "なにそれ食べられるの？",
        "食べられるの",
        "読んだん？",
        "読んだの。",
        "「食べるの？」",
        "食べるのね。",
    ] {
        let doc = analyze(&service, text);
        let index = doc
            .morphemes
            .iter()
            .position(|token| token.lemma.as_deref() == Some("の"))
            .unwrap();
        let chain = doc
            .morphology
            .chains
            .iter()
            .find(|chain| chain.morpheme_indices.contains(&index))
            .unwrap();
        assert_eq!(chain.morpheme_indices, vec![index], "{text}");
        assert!(chain.parent_chain_id.is_none());
        assert!(chain.operators.is_empty());
        assert!(!doc
            .morphology
            .occurrences
            .iter()
            .any(|occurrence| occurrence.morpheme_indices.contains(&index)));
        assert!(doc
            .grammar
            .occurrences
            .iter()
            .any(|occurrence| occurrence.morpheme_indices.contains(&index)));
    }
    for text in [
        "読んだのだ。",
        "読んだんです。",
        "読んだのが好き。",
        "読むのをやめた。",
    ] {
        let doc = analyze(&service, text);
        assert!(
            doc.morphology
                .chains
                .iter()
                .any(|chain| chain.surface_form.starts_with("読")
                    && chain.operators.iter().any(|op| op.kind == "nominalization")),
            "{text}"
        );
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
        let token = doc
            .morphemes
            .iter()
            .find(|token| token.surface == surface)
            .unwrap();
        let chain = doc
            .morphology
            .chains
            .iter()
            .find(|chain| {
                chain
                    .operators
                    .iter()
                    .any(|op| op.kind == kind && op.char_range == token.char_range)
            })
            .unwrap();
        let operators = chain
            .operators
            .iter()
            .filter(|op| op.char_range == token.char_range)
            .collect::<Vec<_>>();
        assert_eq!(operators.len(), 1, "{text}");
        assert_eq!(operators[0].normalized_form.as_deref(), Some(form));
        assert!(readux_core::grammar_catalog::get(&operators[0].concept_id).is_ok());
        assert!(
            chain.display_form.ends_with(display),
            "{text}: {}",
            chain.display_form
        );
        assert_eq!(
            doc.morphology
                .occurrences
                .iter()
                .filter(|occurrence| occurrence.char_range == token.char_range)
                .count(),
            1
        );
    }
    let doc = analyze(&service, "駅に行くな。");
    assert!(!doc
        .morphology
        .occurrences
        .iter()
        .any(|occurrence| matches!(occurrence.kind.as_str(), "adverbial_ni" | "attributive_na")));
    assert!(doc
        .morphology
        .occurrences
        .iter()
        .any(|occurrence| occurrence.kind == "prohibitive"));
}

#[test]
fn ginza_changes_core_boundaries_and_functional_status() {
    use readux_nlp::{
        external::{Endpoint, RelationKind, SourceArtifact, SourceRelation},
        syntax::*,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    let doc = analyze(&service, "読んでいる。");
    let specs = [
        ("t0", [0, 2], "VERB"),
        ("t1", [2, 3], "SCONJ"),
        ("t2", [3, 5], "AUX"),
    ];
    let syntax = SyntaxArtifact {
        schema: SCHEMA.into(),
        segment_id: None,
        text_characters: doc.characters,
        text_sha256: readux_nlp::external::text_digest(&doc.text),
        provider: SyntaxProviderDescriptor {
            id: "ginza".into(),
            version: None,
            capabilities: Vec::new(),
            license: None,
        },
        spans: specs
            .iter()
            .map(|(id, range, pos)| SyntaxSpan {
                id: (*id).into(),
                kind: "token".into(),
                char_range: *range,
                head_char_range: None,
                source_id: "fixture".into(),
                surface: None,
                labels: vec![format!("pos:{pos}")],
            })
            .collect(),
    };
    let mut source = SourceArtifact::from_syntax(&syntax, &doc.text).unwrap();
    source.relations.push(SourceRelation {
        id: "aux2".into(),
        kind: RelationKind::Dependency,
        source: Endpoint::Node { id: "t2".into() },
        target: Endpoint::Node { id: "t0".into() },
        label: "aux".into(),
        features: serde_json::json!({}),
    });
    let with = readux_nlp::morphology::collect_with_external(
        &doc.source.tokens,
        &doc.morphemes,
        &[source.clone()],
    )
    .unwrap();
    assert_eq!(
        with.chains
            .iter()
            .find(|c| c.surface_form == "いる")
            .unwrap()
            .status,
        "resolved"
    );
    source.relations.clear();
    let without = readux_nlp::morphology::collect_with_external(
        &doc.source.tokens,
        &doc.morphemes,
        &[source],
    )
    .unwrap();
    assert_eq!(
        without
            .chains
            .iter()
            .find(|c| c.surface_form == "いる")
            .unwrap()
            .status,
        "resolved"
    );
    assert!(without
        .chains
        .iter()
        .find(|c| c.surface_form == "いる")
        .unwrap()
        .evidence
        .iter()
        .any(|item| item == "support_lexical_connection_only"));

    let doc = analyze(&service, "情報処理。");
    let mut syntax = syntax;
    syntax.text_characters = doc.characters;
    syntax.text_sha256 = readux_nlp::external::text_digest(&doc.text);
    syntax.spans = vec![SyntaxSpan {
        id: "whole".into(),
        kind: "token".into(),
        char_range: [0, 4],
        head_char_range: None,
        source_id: "fixture".into(),
        surface: None,
        labels: vec!["pos:NOUN".into()],
    }];
    let source = SourceArtifact::from_syntax(&syntax, &doc.text).unwrap();
    let with = readux_nlp::morphology::collect_with_external(
        &doc.source.tokens,
        &doc.morphemes,
        &[source],
    )
    .unwrap();
    assert!(with
        .chains
        .iter()
        .any(|c| c.surface_form == "情報処理" && c.core_morpheme_indices.len() == 2));
    assert!(doc
        .morphology
        .chains
        .iter()
        .all(|c| c.core_morpheme_indices.len() == 1));
}

#[test]
fn formation_uses_dependencies_and_preserves_partial_coverage() {
    use readux_nlp::{external::*, formation::*, syntax::*};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let service = AnalysisService::new(ResourcePaths::development(&root));
    let doc = analyze(&service, "情報処理。");
    let mut syntax = SyntaxArtifact {
        schema: readux_nlp::syntax::SCHEMA.into(),
        segment_id: None,
        provider: SyntaxProviderDescriptor {
            id: "ginza".into(),
            version: None,
            capabilities: vec![],
            license: None,
        },
        text_sha256: text_digest(&doc.text),
        text_characters: doc.characters,
        spans: vec![],
    };
    for (id, range) in [("a", [0, 2]), ("b", [2, 4])] {
        syntax.spans.push(SyntaxSpan {
            id: id.into(),
            kind: "token".into(),
            char_range: range,
            head_char_range: None,
            source_id: "fixture".into(),
            surface: None,
            labels: vec!["pos:NOUN".into()],
        });
    }
    let mut source = SourceArtifact::from_syntax(&syntax, &doc.text).unwrap();
    source.relations.push(SourceRelation {
        id: "compound-a-b".into(),
        kind: RelationKind::Dependency,
        source: Endpoint::Node { id: "a".into() },
        target: Endpoint::Node { id: "b".into() },
        label: "compound".into(),
        features: serde_json::json!({}),
    });
    let mut artifact =
        collect_formations_with_sources(&doc.text, &doc.morphemes, &doc.structure, &[source])
            .unwrap();
    attach_words(&doc.text, &doc.morphemes, &doc.morphology, &mut artifact);
    let word = artifact
        .nodes
        .iter()
        .find(|n| n.char_range == [0, 4])
        .unwrap()
        .word
        .as_ref()
        .unwrap();
    assert_eq!(word.source_relation_ids, vec!["compound-a-b"]);
    assert_eq!(word.head_morpheme, Some(1));
    assert_eq!(word.component_candidate_ids.len(), 2);
    assert_eq!(
        word.query_forms[0].reading.as_deref(),
        Some("ジョウホウショリ")
    );
    let mut uncertain = doc.morphology.clone();
    uncertain.chains[0].status = "candidate".into();
    attach_words(&doc.text, &doc.morphemes, &uncertain, &mut artifact);
    assert_eq!(artifact.nodes[0].status, FormationStatus::Candidate);
    assert_eq!(
        artifact.nodes[0].word.as_ref().unwrap().reason,
        "morphology_core_candidate"
    );
    let mut tokens = doc.morphemes.clone();
    tokens[1].query_forms.push(readux_nlp::model::QueryForm {
        kind: "base".into(),
        form: "処理する".into(),
        reading: None,
        reading_field: None,
    });
    attach_words(&doc.text, &tokens, &doc.morphology, &mut artifact);
    let restored = artifact.nodes[0]
        .word
        .as_ref()
        .unwrap()
        .query_forms
        .iter()
        .find(|q| q.form == "情報処理する")
        .unwrap();
    assert!(restored.reading.is_none(), "基本形缺少读音时应保留缺失状态");
    syntax.spans[0].char_range = [0, 1];
    syntax.spans[0].kind = "compound".into();
    syntax.spans.truncate(1);
    let (structure, _) = readux_nlp::structure::merge_external(
        readux_nlp::structure::local_candidates(&doc.text),
        &syntax,
        &doc.text,
    )
    .unwrap();
    let mut partial = collect_formations(&doc.text, &doc.morphemes, &structure).unwrap();
    attach_words(&doc.text, &doc.morphemes, &doc.morphology, &mut partial);
    assert_eq!(partial.nodes[0].status, FormationStatus::Pending);
    assert!(partial.nodes[0].word.as_ref().unwrap().query_forms[0]
        .reading
        .is_none());
}
