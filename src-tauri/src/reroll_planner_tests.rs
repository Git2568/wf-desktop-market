use super::*;
use crate::market::{RivenAttribute, RivenWeapon};
use serde_json::{json, Value};
fn catalog() -> MarketCatalog {
    let v: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/catalog-seed.json"
    )))
    .unwrap();
    let weapons = v["weapons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| serde_json::from_value::<RivenWeapon>(x.clone()).unwrap())
        .collect();
    let attributes = v["attributes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            let slug = a["slug"].as_str().unwrap().to_string();
            let name = a["i18n"]["zh-hans"]["name"]
                .as_str()
                .or(a["i18n"]["en"]["name"].as_str())
                .unwrap_or(&slug)
                .to_string();
            RivenAttribute {
                id: a["id"].as_str().unwrap().into(),
                slug: slug.clone(),
                name: name.clone(),
                english_name: a["i18n"]["en"]["name"].as_str().unwrap_or(&name).into(),
                unit: a["unit"].as_str().unwrap_or("").into(),
                positive_only: a["positiveOnly"].as_bool().unwrap_or(false),
                negative_only: a["negativeOnly"].as_bool().unwrap_or(false),
                exclusive_to: a["exclusiveTo"]
                    .as_array()
                    .map(|x| {
                        x.iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                game_ref: a["gameRef"].as_str().unwrap_or("").into(),
                positive_is_negative: a["positiveIsNegative"].as_bool().unwrap_or(false),
            }
        })
        .collect();
    MarketCatalog {
        weapons,
        attributes,
    }
}
fn attr(slug: &str, value: f64, positive: bool, spliced: bool) -> InputAttribute {
    InputAttribute {
        slug: slug.into(),
        value,
        positive,
        spliced,
    }
}
fn request() -> RerollPlanRequest {
    RerollPlanRequest {
        weapon_slug: "torid".into(),
        mod_rank: 8,
        rule_mode: "current".into(),
        disposition: Some(1.2),
        current_attributes: vec![
            attr("critical_chance", 180., true, false),
            attr("critical_damage", 145., true, false),
        ],
        rerolls: 0,
        kuva_budget: Some(50000.),
        objective: Objective::Kuva,
        target: TargetInput {
            groups: vec![TargetGroupInput {
                slugs: vec!["multishot".into()],
                min_w: 1.,
                min_values: None,
            }],
            sets: None,
            layouts: vec![
                Layout { pos: 2, neg: 0 },
                Layout { pos: 2, neg: 1 },
                Layout { pos: 3, neg: 0 },
                Layout { pos: 3, neg: 1 },
            ],
            allowed_negatives: None,
            at_least: None,
        },
        new_attributes: Some(vec![
            attr("multishot", 112., true, false),
            attr("critical_damage", 145., true, false),
        ]),
    }
}
fn fixture(name: &str, r: &RerollPlanRequest, z: &RerollPlanResponse) {
    let Some(dir) = std::env::var_os("WM_REROLL_EXPORT_FIXTURES") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(format!("{name}.json")),
        serde_json::to_string_pretty(&json!({"request":r,"response":z})).unwrap(),
    )
    .unwrap();
}
#[test]
fn torid_current_new_real_quantiles_and_budget_cache() {
    let c = catalog();
    let r = request();
    let mut planner = Planner::new();
    let z = planner.plan(&c, r.clone());
    assert!(z.ok, "{:?}", z.error);
    assert!(!z.done && z.reachable);
    assert_eq!(z.compare_values, vec![Some(0.)]);
    assert_eq!(z.compare_neg, vec![None]);
    assert!(z.current_value.unwrap() > 0.);
    assert_eq!(z.samples, 3000);
    assert!(z.routes.iter().any(|s| s.kuva.p10 != s.kuva.p90));
    assert!(z
        .routes
        .iter()
        .all(|s| s.within_budget.is_some_and(|p| p > 0. && p < 1.)));
    assert!(z
        .routes
        .iter()
        .all(|s| matches!(&s.route.action, Action::Cycle { .. })));
    assert!(z
        .routes
        .iter()
        .all(|s| s.route.outcomes.iter().chain(&s.route.worse).all(|o| o
            .needs
            .iter()
            .all(|n| n.stats.iter().all(|x| !x.starts_with("Weapon"))))));
    assert!(z
        .routes
        .iter()
        .all(|s| s.next_cycle_cost == 900. || s.next_cycle_cost == 1800.));
    fixture("torid-compare", &r, &z);
    let work = planner.work_used();
    let seed = planner.seed;
    let mut r2 = r.clone();
    r2.kuva_budget = Some(10000.);
    let z2 = planner.plan(&c, r2.clone());
    assert!(z2.ok);
    assert_eq!(
        planner.seed, seed,
        "budget-only change must reuse simulations"
    );
    assert_eq!(planner.work_used(), work);
    for (a, b) in z.routes.iter().zip(&z2.routes) {
        assert_eq!(a.kuva.mean, b.kuva.mean);
        assert!(b.within_budget.unwrap() <= a.within_budget.unwrap());
    }
    fixture("torid-budget-10000", &r2, &z2);
    let mut done = r;
    done.current_attributes = done.new_attributes.take().unwrap();
    done.rerolls = 1;
    let zd = planner.plan(&c, done.clone());
    assert!(zd.ok && zd.done && zd.reachable && zd.routes.is_empty());
    assert_eq!(zd.current_value, Some(0.));
}
#[test]
fn torid_direct_splice_and_existing_spliced_layout() {
    let c = catalog();
    let mut r = request();
    r.current_attributes = vec![
        attr("toxin_damage", 105., true, false),
        attr("cold_damage", 105., true, false),
    ];
    r.target.groups = vec![TargetGroupInput {
        slugs: vec!["viral".into()],
        min_w: 0.9,
        min_values: None,
    }];
    r.target.layouts = vec![Layout { pos: 2, neg: 0 }];
    r.target.allowed_negatives = Some(vec![]);
    r.new_attributes = Some(vec![
        attr("viral", 105., true, true),
        attr("critical_damage", 145., true, false),
    ]);
    let prepared = prepare(&c, &r).unwrap();
    assert_eq!(prepared.compared.as_ref().unwrap().layout.pos, 2);
    assert_eq!(prepared.compared.as_ref().unwrap().pos.len(), 1);
    let mut planner = Planner::new();
    let z = planner.plan(&c, r.clone());
    assert!(z.ok, "{:?}", z.error);
    assert_eq!(z.compare_values, vec![Some(0.)]);
    let splice = z
        .routes
        .iter()
        .find(|s| matches!(&s.route.action,Action::Splice{result,..}if result=="viral"))
        .expect("direct viral splice");
    assert_eq!(splice.next_cycle_cost, 0.);
    assert_eq!(splice.route.expected, 0.);
    assert_eq!(splice.kuva.mean, 0.);
    assert_eq!(splice.cycles.mean, 0.);
    assert_eq!(splice.splicers, 1.);
    assert_eq!(splice.within_budget, Some(1.));
    fixture("torid-splice", &r, &z);
    let mut done = r;
    done.current_attributes = done.new_attributes.take().unwrap();
    let zd = planner.plan(&c, done.clone());
    assert!(zd.ok && zd.done);
}
#[test]
fn exact_mapping_options_and_kept_pool() {
    let c = catalog();
    let braton = options(&c, "braton", &[]).unwrap();
    let torid = options(&c, "torid", &[]).unwrap();
    assert!(braton.positive.contains(&"impact_damage".into()));
    assert!(!torid.positive.contains(&"impact_damage".into()));
    let with_seen = options(&c, "torid", &[attr("impact_damage", 100., true, false)]).unwrap();
    assert!(with_seen.positive.contains(&"impact_damage".into()));
    assert!(torid.spliced.contains(&"viral".into()));
    assert!(!torid.negative.contains(&"toxin_damage".into()));
    assert!(
        options(&c, "plague_keewar", &[]).is_err(),
        "unmapped path must not guess variant"
    );
    let mut c2 = c.clone();
    c2.weapons
        .iter_mut()
        .find(|w| w.slug == "braton")
        .unwrap()
        .game_ref = "/Lotus/NotReal".into();
    assert!(options(&c2, "braton", &[]).is_err());
    let skana = Mapping::new(&c, RivenClass::Melee).unwrap();
    assert_eq!(
        skana.tag("base_damage_/_melee_damage").unwrap(),
        "WeaponMeleeDamageMod"
    );
    assert_eq!(
        skana.slug("WeaponMeleeFactionDamageCorpus").unwrap(),
        "damage_vs_corpus"
    );
}
#[test]
fn braton_satisfied_and_display_threshold_conversion() {
    let c = catalog();
    let mut r = request();
    r.weapon_slug = "braton".into();
    r.new_attributes = None;
    r.current_attributes = vec![
        attr("multishot", 112., true, false),
        attr("critical_damage", 145., true, false),
    ];
    let z = plan(&c, r.clone());
    assert!(z.ok && z.done);
    fixture("braton-done", &r, &z);
    r.current_attributes = vec![
        attr("critical_chance", 180., true, false),
        attr("critical_damage", 145., true, false),
    ];
    r.target.groups[0].min_w = 0.9;
    r.target.groups[0].min_values = Some(HashMap::from([("multishot".into(), 116.)]));
    let p = prepare(&c, &r).unwrap();
    let by = p.target.groups[0].min_w_by.as_ref().unwrap();
    let bars = &by["WeaponFireIterationsMod"];
    assert_ne!(bars["20"], bars["21"]);
    assert!(bars["31"].is_infinite());
    let z = plan(&c, r.clone());
    assert!(z.ok && z.reachable);
    fixture("braton-display-threshold", &r, &z);
    r.target.layouts = vec![Layout { pos: 3, neg: 1 }];
    let z = plan(&c, r.clone());
    assert!(z.ok && !z.reachable);
    assert_eq!(z.current_value, None);
    assert!(z.routes.is_empty());
    let serialized = serde_json::to_value(&z).unwrap();
    assert!(serialized["current_value"].is_null());
    fixture("braton-unreachable-threshold", &r, &z);
}
#[test]
fn compare_negative_only_and_real_cycles_objective() {
    let c = catalog();
    let mut r = request();
    r.objective = Objective::Cycles;
    r.target.allowed_negatives = Some(vec!["recoil".into()]);
    r.target.layouts = vec![Layout { pos: 2, neg: 1 }];
    let mapping = Mapping::new(&c, RivenClass::Rifle).unwrap();
    let l = data::Layout { pos: 2, neg: 1 };
    let make = |slug: &str, w: f64, positive: bool| {
        let s = mapping.stat(slug).unwrap();
        attr(
            slug,
            data::display_number(s, data::raw_value(s, l, !positive, w, 8, 1.2)),
            positive,
            s.spliced,
        )
    };
    r.current_attributes = vec![
        make("critical_chance", 1., true),
        make("critical_damage", 1., true),
        make("recoil", 1., false),
    ];
    r.new_attributes = Some(vec![
        make("multishot", 1.08, true),
        make("critical_damage", 1., true),
        make("zoom", 1., false),
    ]);
    let mut p = Planner::new();
    let z = p.plan(&c, r.clone());
    assert!(z.ok, "{:?}", z.error);
    assert_eq!(z.compare_neg, vec![Some("zoom".into())]);
    assert!(z.compare_values[0].unwrap() > 0.);
    for s in &z.routes {
        assert_eq!(s.cycles.mean, s.route.expected);
    }
    fixture("torid-compare-negative", &r, &z);
}
#[test]
fn invalid_flags_numbers_types_and_budget() {
    let c = catalog();
    let mut r = request();
    r.current_attributes[0].spliced = true;
    assert!(!plan(&c, r.clone()).ok);
    r = request();
    r.current_attributes[0].value = f64::NAN;
    assert!(!plan(&c, r.clone()).ok);
    r = request();
    r.current_attributes[1].slug = r.current_attributes[0].slug.clone();
    assert!(!plan(&c, r.clone()).ok);
    r = request();
    r.current_attributes[0].slug = "range".into();
    assert!(!plan(&c, r.clone()).ok);
    r = request();
    r.target.sets = Some(vec![vec![0, 0]]);
    assert!(!plan(&c, r.clone()).ok);
    r = request();
    r.target.groups[0].min_values = Some(HashMap::from([("critical_damage".into(), 100.)]));
    assert!(!plan(&c, r.clone()).ok);
    r = request();
    let mut p = Planner::with_budget(1);
    let z = p.plan(&c, r);
    assert!(!z.ok);
    assert_eq!(z.error.as_deref(), Some("too-complex"));
    assert_eq!(p.work_used(), 0);
}
#[test]
fn ui_sequence_fixtures_use_real_planner_and_options() {
    let c = catalog();
    let mut current = request();
    current.new_attributes = None;
    let z = plan(&c, current.clone());
    assert!(z.ok && !z.done);
    fixture("torid-current", &current, &z);
    let mut keep = current.clone();
    keep.rerolls = 1;
    keep.kuva_budget = Some(49100.);
    let z = plan(&c, keep.clone());
    assert!(z.ok && !z.done);
    fixture("torid-keep-old-r1", &keep, &z);
    let mut applied = request();
    applied.current_attributes = applied.new_attributes.take().unwrap();
    applied.rerolls = 1;
    applied.kuva_budget = Some(49100.);
    let z = plan(&c, applied.clone());
    assert!(z.ok && z.done);
    fixture("torid-applied-done", &applied, &z);
    let mut splice = current;
    splice.current_attributes = vec![
        attr("toxin_damage", 105., true, false),
        attr("cold_damage", 105., true, false),
    ];
    splice.target.groups = vec![TargetGroupInput {
        slugs: vec!["viral".into()],
        min_w: 0.9,
        min_values: None,
    }];
    splice.target.layouts = vec![Layout { pos: 2, neg: 0 }];
    splice.target.allowed_negatives = Some(vec![]);
    let z = plan(&c, splice.clone());
    assert!(z.ok && !z.done);
    fixture("torid-splice-current", &splice, &z);
    let mut applied = splice;
    applied.current_attributes = vec![
        attr("viral", 105., true, true),
        attr("critical_damage", 145., true, false),
    ];
    let z = plan(&c, applied.clone());
    assert!(z.ok && z.done);
    fixture("torid-splice-applied-done", &applied, &z);
    let options = options(&c, "torid", &[]).unwrap();
    if let Some(dir) = std::env::var_os("WM_REROLL_EXPORT_FIXTURES") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("torid-options.json"),
            serde_json::to_string_pretty(&options).unwrap(),
        )
        .unwrap();
    }
}
