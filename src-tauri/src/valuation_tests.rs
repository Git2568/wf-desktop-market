#![allow(clippy::too_many_arguments)]

use crate::{
    analysis_service::{WeeklyGroup, WeeklyWeaponSummary},
    market::{DisplayAttribute, Listing, MarketCatalog, MarketService},
    riven_analysis::{self, AnalysisAttributeInput, AnalysisRequest},
    riven_valuation::{self, ValuationResult},
};

fn catalog() -> MarketCatalog {
    tauri::async_runtime::block_on(MarketService::new().catalog()).unwrap()
}
fn request(slug: &str) -> AnalysisRequest {
    AnalysisRequest {
        weapon_slug: slug.into(),
        mod_rank: 8,
        rule_mode: "current".into(),
        disposition: None,
        sample_size: 20,
        attributes: vec![
            AnalysisAttributeInput {
                slug: "critical_chance".into(),
                value: 180.,
                positive: true,
            },
            AnalysisAttributeInput {
                slug: "critical_damage".into(),
                value: 145.,
                positive: true,
            },
        ],
    }
}
fn listing(id: &str, price: u32, attrs: &[(&str, f64, bool)], rank: Option<u32>) -> Listing {
    Listing {
        id: id.into(),
        riven_name: id.into(),
        price: Some(price),
        price_kind: "一口价".into(),
        seller: "fixture".into(),
        seller_status: "online".into(),
        rerolls: Some(0),
        mod_rank: rank,
        attributes: attrs
            .iter()
            .map(|(slug, value, positive)| DisplayAttribute {
                slug: (*slug).into(),
                name: (*slug).into(),
                value: *value,
                positive: *positive,
                unit: String::new(),
            })
            .collect(),
        polarity: None,
        mastery_rank: None,
    }
}
fn estimate(req: &AnalysisRequest, rows: &[Listing]) -> ValuationResult {
    let c = catalog();
    let r = riven_analysis::embedded_reference().unwrap();
    riven_valuation::estimate(req, &c, &r, rows, None, false)
}
fn exact(id: &str, price: u32) -> Listing {
    listing(
        id,
        price,
        &[
            ("critical_chance", 180., true),
            ("critical_damage", 145., true),
        ],
        Some(8),
    )
}
fn assert_core(req: &AnalysisRequest, c: &MarketCatalog) {
    let x =
        riven_analysis::analyze(req, c, &riven_analysis::embedded_reference().unwrap()).unwrap();
    assert!(x.ranges.iter().all(|r| r.position_percent.is_some()));
}

#[test]
fn exact_pool_selects_three_and_rejects_expensive_dissimilar() {
    let q = request("torid");
    let mut rows = vec![exact("a", 100), exact("b", 110), exact("c", 120)];
    rows.push(listing(
        "far",
        1_000_000,
        &[("critical_chance", 180., true), ("multishot", 100., true)],
        Some(8),
    ));
    let v = estimate(&q, &rows);
    assert_eq!(v.comparable_count, 3);
    assert_eq!(v.estimate, Some(110.));
    assert!((v.numeric_factor - 1.).abs() < 1e-9 && (v.numeric_coverage - 1.).abs() < 1e-9);
}
#[test]
fn median_pool_is_not_controlled_by_extreme_price() {
    let q = request("torid");
    let rows = [
        exact("a", 100),
        exact("b", 110),
        exact("c", 120),
        exact("d", 130),
        exact("e", 1_000_000),
    ];
    let v = estimate(&q, &rows);
    assert_eq!(v.estimate, Some(120.));
}
#[test]
fn partial_nearest_excludes_far_reference() {
    let q = request("torid");
    let rows = [
        listing(
            "partial",
            100,
            &[("critical_chance", 180., true), ("multishot", 100., true)],
            Some(8),
        ),
        listing(
            "far",
            500,
            &[
                ("magazine_capacity", 100., true),
                ("ammo_maximum", 100., true),
            ],
            Some(8),
        ),
    ];
    let v = estimate(&q, &rows);
    assert!(!v.references.iter().any(|x| x.id == "far"));
    assert!(v
        .references
        .iter()
        .all(|x| x.similarity >= v.references.first().map(|y| y.similarity).unwrap_or(0.) - 0.15));
}
#[test]
fn hot_stats_follow_each_weapon_pool() {
    let mut a = request("torid");
    let b = request("soma");
    let va = estimate(
        &a,
        &[
            exact("a", 100),
            exact("b", 110),
            listing(
                "m",
                120,
                &[("critical_chance", 180., true), ("multishot", 100., true)],
                Some(8),
            ),
        ],
    );
    let vb = estimate(
        &b,
        &[
            listing(
                "m1",
                100,
                &[("multishot", 100., true), ("toxin_damage", 100., true)],
                Some(8),
            ),
            listing(
                "m2",
                110,
                &[("multishot", 100., true), ("toxin_damage", 100., true)],
                Some(8),
            ),
            listing(
                "c",
                120,
                &[
                    ("critical_chance", 180., true),
                    ("critical_damage", 145., true),
                ],
                Some(8),
            ),
        ],
    );
    assert_ne!(
        va.hot_stats.first().map(|x| &x.slug),
        vb.hot_stats.first().map(|x| &x.slug)
    );
    a.weapon_slug = "torid".into();
}
#[test]
fn extra_hot_stat_changes_popularity_factor() {
    let c = catalog();
    let q = AnalysisRequest {
        weapon_slug: "torid".into(),
        mod_rank: 8,
        rule_mode: "current".into(),
        disposition: None,
        sample_size: 20,
        attributes: vec![
            AnalysisAttributeInput {
                slug: "critical_chance".into(),
                value: 170.,
                positive: true,
            },
            AnalysisAttributeInput {
                slug: "critical_damage".into(),
                value: 140.,
                positive: true,
            },
            AnalysisAttributeInput {
                slug: "multishot".into(),
                value: 100.,
                positive: true,
            },
            AnalysisAttributeInput {
                slug: "fire_rate_/_attack_speed".into(),
                value: -54.,
                positive: false,
            },
        ],
    };
    assert_core(&q, &c);
    let rows = (0..2)
        .map(|i| {
            listing(
                &format!("cc{i}"),
                100,
                &[
                    ("critical_chance", 220., true),
                    ("critical_damage", 180., true),
                    ("fire_rate_/_attack_speed", -35., false),
                ],
                Some(8),
            )
        })
        .chain((0..3).map(|i| {
            listing(
                &format!("ms{i}"),
                100,
                &[
                    ("multishot", 130., true),
                    ("magazine_capacity", 65., true),
                    ("fire_rate_/_attack_speed", -35., false),
                ],
                Some(8),
            )
        }))
        .collect::<Vec<_>>();
    let v = estimate(&q, &rows);
    assert!(v.references.iter().any(|x| x.popularity_factor > 1.));
}
#[test]
fn numeric_factor_is_propagated_from_single_reference() {
    let q = request("torid");
    let v = estimate(
        &q,
        &[listing(
            "r",
            100,
            &[
                ("critical_chance", 170., true),
                ("critical_damage", 135., true),
            ],
            Some(8),
        )],
    );
    assert!(v.references[0].numeric_factor > 1. && v.references[0].adjusted_price > 100.);
    assert!((v.numeric_factor - v.references[0].numeric_factor).abs() < 1e-9);
}
#[test]
fn invalid_candidates_and_invalid_target_are_unavailable() {
    let q = request("torid");
    let rows = [
        listing(
            "dup",
            0,
            &[
                ("critical_chance", 180., true),
                ("critical_damage", 145., true),
            ],
            Some(8),
        ),
        listing(
            "dup",
            100,
            &[
                ("critical_chance", 180., false),
                ("critical_damage", 145., false),
            ],
            Some(8),
        ),
        listing(
            "neg",
            100,
            &[
                ("critical_chance", 180., false),
                ("critical_damage", 145., false),
            ],
            Some(8),
        ),
        listing(
            "nan",
            100,
            &[
                ("critical_chance", f64::NAN, true),
                ("critical_damage", 145., true),
            ],
            Some(8),
        ),
    ];
    let v = estimate(&q, &rows);
    assert_eq!(v.status, "unavailable");
    assert!(v.estimate.is_none());
    let mut bad = q.clone();
    bad.attributes[0].value = 999.;
    assert_eq!(estimate(&bad, &rows).status, "unavailable");
}
#[test]
fn weekly_heat_uses_group_average_and_none_is_one() {
    let q = request("torid");
    let rows = [exact("a", 100), exact("b", 110)];
    let weekly = WeeklyWeaponSummary {
        source: "fixture".into(),
        source_updated_at: None,
        fetched_at: 0,
        groups: vec![
            WeeklyGroup {
                rerolled: false,
                popularity: 40.,
                avg: 0.,
                median: None,
                min: 0.,
                max: 0.,
                stddev: 0.,
            },
            WeeklyGroup {
                rerolled: true,
                popularity: 50.,
                avg: 0.,
                median: None,
                min: 0.,
                max: 0.,
                stddev: 0.,
            },
        ],
    };
    let c = catalog();
    let r = riven_analysis::embedded_reference().unwrap();
    let v = riven_valuation::estimate(&q, &c, &r, &rows, Some(&weekly), false);
    assert!((v.heat_factor - 1.01).abs() < 1e-9);
    assert!(v.low.unwrap() <= v.estimate.unwrap() && v.estimate.unwrap() <= v.high.unwrap());
    let n = riven_valuation::estimate(&q, &c, &r, &rows, None, false);
    assert_eq!(n.heat_factor, 1.);
}
#[test]
fn missing_rank_pool_has_zero_numeric_coverage() {
    let q = request("torid");
    let rows = (0..20)
        .map(|i| exact(&format!("x{i}"), 100 + i))
        .collect::<Vec<_>>();
    let mut missing = rows.clone();
    for x in &mut missing {
        x.mod_rank = None;
    }
    let v = estimate(&q, &missing);
    assert_eq!(v.numeric_coverage, 0.);
    assert_eq!(v.confidence, "low");
    assert!(v.warnings.iter().any(|x| x.contains("数值")));
    let s = estimate(&q, &rows);
    assert!(s.numeric_coverage > 0. && s.confidence == "medium");
}

#[test]
fn writes_real_ui_fixture_when_requested() {
    let Ok(path) = std::env::var("WM_VALUATION_FIXTURE_PATH") else {
        return;
    };
    let q = request("torid");
    let v = estimate(&q, &[exact("a", 100), exact("b", 110), exact("c", 120)]);
    std::fs::write(path, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
}
