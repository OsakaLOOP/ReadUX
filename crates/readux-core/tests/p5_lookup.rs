//! P5 词典查询目标的代表范围回归。
use readux_core::analysis::{AnalysisService, Request, ResourcePaths};
use serde_json::Value;
use std::path::PathBuf;

fn service() -> AnalysisService {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    AnalysisService::new(ResourcePaths::development(&root))
}

fn analyze(service: &AnalysisService, text: &str) -> (String, Value) {
    let response = service.dispatch(Request::Analyze { text: text.into(), register: readux_nlp::model::Register::Cwj });
    let document = response.result.expect("分析结果缺失");
    let id = document["id"].as_str().expect("分析 ID 缺失").to_owned();
    (id, document)
}

fn targets(service: &AnalysisService, id: &str, document: &Value) -> Value {
    service.dispatch(Request::LookupTargets { analysis_id: id.into(), range: [0, document["characters"].as_u64().unwrap() as usize] })
        .result.expect("查询目标缺失")
}

fn surfaces(targets: &Value) -> Vec<String> {
    targets["outer_targets"].as_array().unwrap().iter().filter_map(|target| target["surface"].as_str().map(str::to_owned)).collect()
}

#[test]
fn p01_direction_and_causative_ranges_are_projected() {
    let service = service();
    let (id, document) = analyze(&service, "政治運動への方向転換。発展せしめる一転機をなした。");
    let targets = targets(&service, &id, &document);
    let outer = surfaces(&targets);
    assert!(outer.contains(&"方向転換".into()), "方向転換应作为最大范围：{outer:?}");
    assert!(outer.contains(&"発展".into()), "使役前词汇核心应保留：{outer:?}");
    assert!(!outer.contains(&"発展せ".into()), "使役连接不应进入外层范围：{outer:?}");
    assert!(outer.contains(&"一転機".into()), "一転機应保留整体候选：{outer:?}");
}

#[test]
fn p02_te_connection_keeps_lexical_core_and_falls_back() {
    let service = service();
    let (id, document) = analyze(&service, "白南風は送梅の風なり。雑ゆ。陰湿漸くに霽れて。");
    let targets = targets(&service, &id, &document);
    let outer = surfaces(&targets);
    assert!(outer.contains(&"送梅".into()) || (outer.contains(&"送".into()) && outer.contains(&"梅".into())), "送梅应整体或相邻分解：{outer:?}");
    assert!(outer.contains(&"雑".into()), "雑应可查询：{outer:?}");
    assert!(outer.contains(&"霽れ".into()), "霽れ应作为词汇核心：{outer:?}");
    assert!(!outer.contains(&"霽れて".into()), "て接续不应进入外层范围：{outer:?}");
}

#[test]
fn p03_dictionary_matrix_returns_real_entries_and_outer_ranges_do_not_overlap() {
    let service = service();
    let (id, document) = analyze(&service, "超絶哲学者の猫。日向ぼこりを読む。新聞記者。");
    let targets = targets(&service, &id, &document);
    let outer = targets["outer_targets"].as_array().unwrap();
    assert!(outer.iter().any(|target| target["surface"] == "哲学者" || target["surface"] == "超絶哲学者"), "哲学者范围缺失：{outer:?}");
    assert!(outer.iter().any(|target| target["surface"] == "日向ぼこり" || target["surface"] == "日向"), "日向ぼこり范围缺失：{outer:?}");
    let mut previous_end = 0;
    for target in outer {
        let range = target["char_range"].as_array().unwrap();
        let start = range[0].as_u64().unwrap();
        let end = range[1].as_u64().unwrap();
        assert!(start >= previous_end, "外层范围发生重叠：{outer:?}");
        previous_end = end;
    }
    let target = outer.iter().find(|target| target["surface"] == "新聞記者" || target["surface"] == "新聞").expect("新聞記者范围缺失");
    if target["matrix_request"].is_object() {
        let result = service.dispatch(Request::QueryTarget { analysis_id: id, target_id: target["id"].as_str().unwrap().into(), selected_form: None }).result.expect("词典矩阵缺失");
        assert!(result["entries"].as_array().is_some_and(|entries| !entries.is_empty()), "应返回真实词典词条：{result:?}");
    }
}

#[test]
fn p01_p03_full_context_preserves_canonical_queries_and_components() {
    use readux_core::dictionary::{lookup::DictionaryEngine, targets};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dictionary = DictionaryEngine::new(root.join("data/dicts")).unwrap();
    for sample in ["p01", "p02", "p03"] {
        let raw: Value = serde_json::from_str(&std::fs::read_to_string(root.join(format!("experiments/p4-sample-review/{sample}.json"))).unwrap()).unwrap();
        let document: readux_nlp::model::UnifiedDocument = serde_json::from_value(raw["unit"]["document"].clone()).unwrap();
        let group = targets::build(&dictionary, &document, [0, document.characters], &[]).unwrap();
        let expected: &[(&str, &str)] = match sample {
            "p01" => &[("成立した", "成立する"), ("発展", "発展する"), ("方向転換", "方向転換"), ("一転機", "一転機")],
            "p02" => &[("送", "送る"), ("梅", "梅"), ("雑", "雑"), ("ゆ", "ゆう"), ("陰湿", "陰湿"), ("漸く", "漸く"), ("霽れ", "霽れる")],
            _ => &[("超絶", "超絶"), ("哲学者", "哲学者"), ("日向ぼこり", "日向ぼこり"), ("新聞記者", "新聞記者"),
                ("ふけっ", "耽る"), ("吹聴して", "吹聴する"), ("頓着し", "頓着する")],
        };
        for (surface, query) in expected {
            let target = group.outer_targets.iter().find(|target| target.surface == *surface)
                .unwrap_or_else(|| panic!("{sample} 缺少 {surface}：{:?}", group.outer_targets.iter().map(|target| &target.surface).collect::<Vec<_>>()));
            assert!(target.lookup_forms.iter().any(|form| form.form == *query), "{sample} {surface} 查询形错误：{:?}", target.lookup_forms);
            let request = target.matrix_request.as_ref().unwrap();
            assert!(request.lookup_forms.iter().all(|form| form.form != "し" && form.form != "発展せ" && form.form != "吹聴し"));
            if matches!(*surface, "成立した" | "発展" | "吹聴して" | "頓着し") {
                assert_eq!(request.lookup_forms[0].form, *query);
            }
            if target.decision == "accepted" {
                let matrix = targets::query(&dictionary, request, None);
                assert!(!matrix.entries.is_empty(), "{sample} {surface} 已接受目标应有正文");
                let active = matrix.forms.iter().find(|form| Some(&form.form_id) == matrix.selected_form_id.as_ref()).unwrap();
                assert!(matrix.entries.iter().all(|entry| readux_core::dictionary::lookup_state::entry_matches_form(entry, &active.display_form)));
            }
        }
        for target in &group.outer_targets {
            for range in group.excluded_ranges.iter().filter(|range| range[0] < target.char_range[1] && target.char_range[0] < range[1]) {
                assert!(group.grammar_targets.iter().any(|grammar| grammar.char_range == *range),
                    "外层目标覆盖的功能成员应保留语法目标：{} {:?}", target.surface, range);
            }
        }
        for child in &group.inner_targets {
            if child.surface == "し" {
                let parent_surface = child.parent_outer_id.as_ref().and_then(|id| group.outer_targets.iter().find(|target| &target.id == id))
                    .map(|target| target.surface.as_str()).unwrap_or("无父目标");
                assert_eq!(child.decision, "component", "{sample} {}（父目标：{}）应归属父词", child.surface, parent_surface);
                assert!(child.matrix_request.is_none());
            }
        }
    }
}

#[test]
fn conditional_sahen_surface_keeps_its_dictionary_and_grammar_targets() {
    let service = service();
    let (id, document) = analyze(&service, "学生なら問題ない。");
    let targets = targets(&service, &id, &document);
    let target = targets["outer_targets"].as_array().unwrap().iter()
        .find(|target| target["surface"] == "なら")
        .expect("なら应生成查询目标");
    assert!(target["pos_evidence"].as_array().unwrap().iter().any(|pos| pos == "助動詞"), "词性证据应保留：{target:?}");
    assert!(targets["grammar_targets"].as_array().unwrap().iter().any(|grammar| grammar["char_range"] == target["char_range"]),
        "条件语法说明应与词典目标并存：{targets:?}");
}

#[test]
fn middle_hanging_strap_remains_a_single_lookup_target() {
    let service = service();
    let (id, document) = analyze(&service, "中吊りに乗る。");
    let targets = targets(&service, &id, &document);
    assert!(targets["outer_targets"].as_array().unwrap().iter().any(|target| target["surface"] == "中吊り"
        && target["char_range"] == serde_json::json!([0, 3])), "中吊り整体查询对象缺失：{targets:?}");
}

#[test]
fn dictionary_backed_adverb_and_to_phrases_keep_whole_targets() {
    let service = service();
    let (id, document) = analyze(&service, "ちょこんと座る。びりっと破れる。ひとつ、ふたつ。");
    let targets = targets(&service, &id, &document);
    let surfaces = surfaces(&targets);
    assert!(surfaces.contains(&"ちょこんと".into()), "ちょこんと整体目标缺失：{surfaces:?}");
    assert!(surfaces.contains(&"びりっと".into()), "びりっと整体目标缺失：{surfaces:?}");
    assert!(surfaces.contains(&"ひとつ".into()), "ひとつ整体目标缺失：{surfaces:?}");
    assert!(surfaces.contains(&"ふたつ".into()), "ふたつ整体目标缺失：{surfaces:?}");
    for (surface, canonical) in [("ひとつ", "一つ"), ("ふたつ", "二つ")] {
        assert!(targets["outer_targets"].as_array().unwrap().iter().any(|target| target["surface"] == surface
            && target["lookup_forms"].as_array().unwrap().iter().any(|form| form["form"] == canonical)),
            "{surface} 缺少规范查询形式 {canonical}");
    }
}

#[test]
fn ruby_boundary_prefers_dictionary_token_range() {
    let service = service();
    let (id, document) = analyze(&service, "審査員各氏絶賛《ゼッサ》");
    let targets = targets(&service, &id, &document);
    let target = targets["outer_targets"].as_array().unwrap().iter()
        .find(|target| target["surface"] == "絶賛")
        .expect("词典边界应将注音定位到绝賛");
    assert_eq!(target["char_range"], serde_json::json!([5, 7]));
    assert!(target["lookup_forms"].as_array().unwrap().iter().any(|form| form["kind"] == "author_ruby"));
}

#[test]
fn sahen_query_object_covers_the_body_conjugation_and_retains_noun_fallback() {
    use readux_core::dictionary::{lookup::DictionaryEngine, targets};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dictionary = DictionaryEngine::new(root.join("data/dicts")).unwrap();
    let service = service();
    for (text, expected_range, grammar_form) in [
        ("力説する。", [0, 4], "する"),
        ("力説した。", [0, 4], "た"),
        ("力説している。", [0, 4], "て"),
    ] {
        let (_, raw) = analyze(&service, text);
        let document: readux_nlp::model::UnifiedDocument = serde_json::from_value(raw).unwrap();
        let group = targets::build(&dictionary, &document, [0, document.characters], &[]).unwrap();
        let target = group.outer_targets.iter().find(|target| target.char_range == expected_range)
            .unwrap_or_else(|| panic!("{text} 缺少サ変整体查询目标：{:?}", group.outer_targets.iter().map(|target| (&target.surface, target.char_range)).collect::<Vec<_>>()));
        assert!(target.lookup_forms.iter().any(|form| form.form == "力説する"), "{text}: {:?}", target.lookup_forms);
        assert!(target.lookup_forms.iter().any(|form| form.form == "力説"), "サ変名词回退查询缺失：{text}");
        let verb = group.inner_targets.iter().find(|child| child.parent_outer_id.as_ref() == Some(&target.id)
            && child.char_range == expected_range && child.decision == "queryable")
            .unwrap_or_else(|| panic!("{text} 缺少サ変动词内部查询对象：{:?}", group.inner_targets));
        assert_eq!(verb.lookup_forms.first().map(|form| form.form.as_str()), Some("力説する"));
        assert!(verb.lookup_forms.iter().any(|form| form.form == "力説"), "动词释义回退查询缺失：{text}");
        assert!(group.inner_targets.iter().any(|child| child.parent_outer_id.as_ref() == Some(&target.id)
            && child.surface == "力説" && child.matrix_request.is_some()), "{text} 缺少名词内部查询对象");
        assert!(group.grammar_targets.iter().any(|grammar| grammar.char_range[0] < expected_range[1]
            && expected_range[0] < grammar.char_range[1]
            && (grammar_form == "する" || grammar.display_form.contains(grammar_form))),
            "{text} 缺少主体/变形语法目标：{:?}", group.grammar_targets);
        if grammar_form == "て" {
            assert!(group.grammar_targets.iter().any(|grammar| grammar.display_form == "て形"),
                "て连接应显示为て形：{:?}", group.grammar_targets);
        }
    }
}

#[test]
fn compound_and_each_noun_component_have_dictionary_targets() {
    let service = service();
    for (word, components) in [("三味線", ["三味", "線"]), ("茶屋酒", ["茶屋", "酒"])] {
        let (id, document) = analyze(&service, word);
        let group = targets(&service, &id, &document);
        let outer = group["outer_targets"].as_array().unwrap().iter()
            .find(|target| target["surface"] == word).expect("整体词条缺失");
        assert_eq!(outer["decision"], "accepted");
        for surface in components {
            let inner = group["inner_targets"].as_array().unwrap().iter()
                .find(|target| target["surface"] == surface).expect("内部词条缺失");
            assert_eq!(inner["decision"], "queryable");
            assert!(inner["matrix_request"].is_object());
            let result = service.dispatch(Request::QueryTarget {
                analysis_id: id.clone(), target_id: inner["id"].as_str().unwrap().into(), selected_form: None,
            }).result.expect("内部词典查询失败");
            assert!(result["entries"].as_array().is_some_and(|entries| !entries.is_empty()));
        }
    }
}

#[test]
fn nominalization_suffix_is_functional_and_queries_adjective_lemma() {
    use readux_core::dictionary::{lookup::DictionaryEngine, targets};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let raw: Value = serde_json::from_str(&std::fs::read_to_string(root.join("experiments/p4-sample-review/p10.json")).unwrap()).unwrap();
    let mut document: readux_nlp::model::UnifiedDocument = serde_json::from_value(raw["unit"]["document"].clone()).unwrap();
    document.morphology = readux_nlp::morphology::collect_with_external(&document.source.tokens, &document.morphemes, &document.external_sources).unwrap();
    let chain = document.morphology.chains.iter().find(|chain| chain.surface_form == "後ろめたさ").unwrap();
    assert_eq!(chain.lookup_form, "後ろめたい");
    assert_eq!(chain.core_morpheme_indices.len(), 1);
    assert_eq!(chain.final_state.category, "名詞");
    let dictionary = DictionaryEngine::new(root.join("data/dicts")).unwrap();
    let group = targets::build(&dictionary, &document, [0, document.characters], &[]).unwrap();
    let target = group.outer_targets.iter().find(|target| target.surface == "後ろめた").expect("後ろめた词汇范围缺失");
    assert!(target.lookup_forms.iter().any(|form| form.form == "後ろめたい"));
    assert!(!group.outer_targets.iter().any(|target| target.surface == "さ"));
    assert!(group.excluded_ranges.iter().any(|range| document.text.chars().skip(range[0]).take(range[1] - range[0]).collect::<String>() == "さ"));
}
