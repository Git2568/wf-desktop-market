use crate::{reroll_pool, reroll_solver};
use serde_json::json;

fn pool() -> reroll_pool::Pool {
    serde_json::from_value(json!({"class":"Rifle","pos":["A","B","C","D","E","F"],"neg":["A","B","C","D","E","F"],"kept_pos":[],"kept_neg":[],"spliced":[],"splices":[]})).unwrap()
}
fn card() -> reroll_solver::Card {
    serde_json::from_value(json!({"layout":{"pos":3,"neg":1},"pos":[{"tag":"A","w":1.0},{"tag":"B","w":1.0},{"tag":"C","w":1.0}],"neg":{"tag":"D","w":1.0},"spliced":null})).unwrap()
}
fn target(groups: serde_json::Value) -> reroll_solver::Target {
    serde_json::from_value(json!({"groups":groups,"layouts":[{"pos":2,"neg":0},{"pos":2,"neg":1},{"pos":3,"neg":0},{"pos":3,"neg":1}],"allowed_neg":["A","B","C","D","E","F"]})).unwrap()
}
#[test]
fn distinct_slots_and_negative_acceptance() {
    use reroll_solver::*;
    let mut s = Solver::new(
        pool(),
        target(json!([{"tags":["A"],"min_w":0.9},{"tags":["A"],"min_w":0.9}])),
        Objective::Cycles,
    );
    let id = s.state_of(&card()).unwrap();
    assert!(!s.is_goal(id));
    let mut t = target(json!([{"tags":["D"],"min_w":0.9}]));
    let mut s = Solver::new(pool(), t.clone(), Objective::Cycles);
    let id = s.state_of(&card()).unwrap();
    assert!(!s.is_goal(id));
    t.allowed_neg.clear();
    t.groups[0].tags = vec!["A".into()];
    let mut s = Solver::new(pool(), t, Objective::Cycles);
    let id = s.state_of(&card()).unwrap();
    assert!(!s.is_goal(id));
}
#[test]
fn budget_fails_while_growing() {
    use reroll_solver::*;
    let a = std::sync::Arc::new(std::sync::Mutex::new(WorkAccount { used: 0, limit: 1 }));
    let mut s = Solver::with_account(
        pool(),
        target(json!([{"tags":["F"],"min_w":0.9}])),
        Objective::Cycles,
        a,
    );
    assert!(matches!(
        s.solve(&card()),
        Err(SolveError::TooComplex { work: 2, .. })
    ));
}
#[test]
fn rare_wait_is_geometric_and_intro_sum_constant_work() {
    use reroll_solver::*;
    let bar = 1.0999999;
    let mut s = Solver::new(
        pool(),
        target(json!([{"tags":["F"],"min_w":bar}])),
        Objective::Kuva,
    );
    let c = card();
    let id = s.solve(&c).unwrap();
    let expected = 3500. / ((2.5 / 6.) * ((1.1 - bar) / 0.2));
    assert!((s.value[id] - expected).abs() / expected < 1e-9);
    let routes = s.routes(&c).unwrap();
    let mut rng = || 0.5;
    let z = s.simulate(&c, &routes[0], 0, 3, &mut rng).unwrap();
    assert!(z.cycles.iter().all(|&n| n > 1_000_000.));
    assert_eq!(cycle_cost_total(0, 10, false), 19950.);
    assert_eq!(
        cycle_cost_total(8, 1_000_000, false),
        3150. + 999999. * 3500.
    );
}
#[test]
fn empty_by_table_preserves_own_identity_and_send() {
    use reroll_solver::*;
    fn assert_send<T: Send>() {}
    assert_send::<Solver>();
    let t = target(json!([{"tags":["A","B"],"min_w":0.9,"min_w_by":{}}]));
    let s = Solver::new(pool(), t, Objective::Kuva);
    let a = s
        .roles
        .iter()
        .position(|r| r.stats.iter().any(|x| x == "A"))
        .unwrap();
    let b = s
        .roles
        .iter()
        .position(|r| r.stats.iter().any(|x| x == "B"))
        .unwrap();
    assert_ne!(a, b);
    assert_eq!(serde_json::to_value(Objective::Cycles).unwrap(), "cycles");
    assert_eq!(
        serde_json::to_value(Action::Cycle {
            lock: Lock::Simple(SimpleLock::None),
            drop: false
        })
        .unwrap(),
        json!({"kind":"cycle","lock":"none"})
    );
}

#[test]
fn nine_golden_cases_match_core_scalars() {
    #[derive(serde::Deserialize)]
    struct C {
        pool: reroll_pool::Pool,
        target: reroll_solver::Target,
        card: reroll_solver::Card,
        objective: reroll_solver::Objective,
    }
    let cases: Vec<C> =
        serde_json::from_str(include_str!("../tests/fixtures/reroll_cases.json")).unwrap();
    let upstream: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../tests/fixtures/reroll_upstream.json")).unwrap();
    assert_eq!(cases.len(), 9);
    for (c, u) in cases.into_iter().zip(upstream) {
        let mut s = reroll_solver::Solver::new(c.pool, c.target, c.objective);
        let id = s.solve(&c.card).unwrap();
        assert_eq!(s.state_count(), u["states"].as_u64().unwrap() as usize);
        let a = s.value[id];
        let b = u["value"].as_f64().unwrap();
        assert!((a - b).abs() / b.abs().max(1.) < 1e-9, "{}", cases_name(&u));
        let routes = s.routes(&c.card).unwrap();
        assert_eq!(routes.len(), u["routes"].as_array().unwrap().len());
        for (i, done) in [0u64, 8, 9].iter().enumerate() {
            let got = s.expected_kuvas(&c.card, &routes, *done).unwrap();
            let key = ["kuva0", "kuva8", "kuva9"][i];
            for (x, y) in got.iter().zip(u[key].as_array().unwrap()) {
                let z = y.as_f64().unwrap();
                assert!((x - z).abs() / z.abs().max(1.) < 1e-9, "{}", cases_name(&u));
            }
        }
    }
}
fn cases_name(v: &serde_json::Value) -> String {
    v["name"].as_str().unwrap_or("case").into()
}

#[test]
fn real_torid_analysis_and_range_api_2p1_and_2p0() {
    use crate::{
        market::MarketService,
        riven_analysis::{
            self, AnalysisAttributeInput, AnalysisRequest, RangeHintInput, RangeHintRequest,
        },
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    let catalog = rt.block_on(MarketService::new().catalog()).unwrap();
    let attr = |game_ref: &str, needle: Option<&str>| {
        catalog
            .attributes
            .iter()
            .find(|a| a.game_ref == game_ref && needle.map(|n| a.slug.contains(n)).unwrap_or(true))
            .cloned()
            .unwrap()
    };
    let ammo = attr("WeaponAmmoEfficiency", None);
    let crit = attr("WeaponCritDamageMod", None);
    let speed = catalog
        .attributes
        .iter()
        .find(|a| {
            !a.positive_only && (a.slug.contains("fire_rate") || a.slug.contains("attack_speed"))
        })
        .cloned()
        .expect("real speed attribute");
    let weapon = catalog
        .weapons
        .iter()
        .find(|w| w.slug == "torid")
        .expect("seed Torid");
    let d = 1.2;
    let layout = crate::riven_roll_data::Layout { pos: 2, neg: 1 };
    let shown = |tag: &str, positive: bool| {
        let s =
            crate::riven_roll_data::stat(crate::riven_roll_data::RivenClass::Rifle, tag).unwrap();
        crate::riven_roll_data::display_number(
            s,
            crate::riven_roll_data::raw_value(s, layout, !positive, 1.0, 8, d),
        )
    };
    let attrs = vec![
        AnalysisAttributeInput {
            slug: ammo.slug.clone(),
            value: shown("WeaponAmmoEfficiency", true),
            positive: true,
        },
        AnalysisAttributeInput {
            slug: crit.slug.clone(),
            value: shown("WeaponCritDamageMod", true),
            positive: true,
        },
        AnalysisAttributeInput {
            slug: speed.slug.clone(),
            value: shown("WeaponFireRateMod", false),
            positive: false,
        },
    ];
    let req = AnalysisRequest {
        weapon_slug: weapon.slug.clone(),
        mod_rank: 8,
        rule_mode: "current".into(),
        disposition: None,
        attributes: attrs.clone(),
        sample_size: 20,
    };
    let reference = riven_analysis::embedded_reference().unwrap();
    let result = riven_analysis::analyze(&req, &catalog, &reference).unwrap();
    assert_eq!(result.ranges.len(), 3);
    for x in &result.ranges {
        assert!(
            matches!(x.status.as_str(), "within" | "available"),
            "{}",
            x.status
        );
        assert!(x.best_value.is_some());
        assert!((x.position_percent.unwrap() - 50.0).abs() < 1.0);
    }
    let hints = riven_analysis::range_hints(
        &RangeHintRequest {
            weapon_slug: weapon.slug.clone(),
            mod_rank: 8,
            rule_mode: "current".into(),
            disposition: None,
            positive_count: 2,
            negative_count: 1,
            attributes: attrs
                .iter()
                .enumerate()
                .map(|(i, a)| RangeHintInput {
                    key: format!("a{i}"),
                    slug: a.slug.clone(),
                    positive: a.positive,
                })
                .collect(),
        },
        &catalog,
        &reference,
    )
    .unwrap();
    assert_eq!(hints.ranges.len(), 3);
    for h in &hints.ranges {
        assert!(h.range_min.is_some() && h.range_max.is_some());
    }
    let req20 = AnalysisRequest {
        attributes: attrs[..2].to_vec(),
        ..req
    };
    let r20 = riven_analysis::analyze(&req20, &catalog, &reference).unwrap();
    assert_eq!(r20.ranges.len(), 2);
}
