use crate::{
    market::{MarketCatalog, MarketService},
    riven_analysis::{
        self, AnalysisAttributeInput, AnalysisRequest, RangeHintInput, RangeHintRequest,
    },
};

fn catalog() -> MarketCatalog {
    tauri::async_runtime::block_on(MarketService::new().catalog()).expect("embedded catalog")
}

fn reference() -> riven_analysis::ReferenceData {
    riven_analysis::embedded_reference().expect("embedded reference")
}

fn inputs(include_third: bool, include_negative: bool) -> Vec<RangeHintInput> {
    let mut out = vec![
        RangeHintInput {
            key: "cc".into(),
            slug: "critical_chance".into(),
            positive: true,
        },
        RangeHintInput {
            key: "cd".into(),
            slug: "critical_damage".into(),
            positive: true,
        },
    ];
    if include_third {
        out.push(RangeHintInput {
            key: "ms".into(),
            slug: "multishot".into(),
            positive: true,
        });
    }
    if include_negative {
        out.push(RangeHintInput {
            key: "fr".into(),
            slug: "fire_rate_/_attack_speed".into(),
            positive: false,
        });
    }
    out
}

fn hint(
    req: RangeHintRequest,
    c: &MarketCatalog,
    r: &riven_analysis::ReferenceData,
) -> riven_analysis::RangeHintsResult {
    riven_analysis::range_hints(&req, c, r).expect("range hints")
}

fn compare_to_analyze(
    req: &RangeHintRequest,
    c: &MarketCatalog,
    r: &riven_analysis::ReferenceData,
) {
    let analysis = AnalysisRequest {
        weapon_slug: req.weapon_slug.clone(),
        mod_rank: req.mod_rank,
        rule_mode: req.rule_mode.clone(),
        disposition: req.disposition,
        attributes: req
            .attributes
            .iter()
            .map(|a| AnalysisAttributeInput {
                slug: a.slug.clone(),
                value: if a.positive { 100.0 } else { -50.0 },
                positive: a.positive,
            })
            .collect(),
        sample_size: 20,
    };
    let analyzed = riven_analysis::analyze(&analysis, c, r).expect("analysis result");
    let hinted = hint(req.clone(), c, r);
    for h in hinted.ranges.iter().filter(|x| x.status == "available") {
        let a = analyzed
            .ranges
            .iter()
            .find(|x| x.slug == h.slug && x.positive == h.positive)
            .expect("matching range");
        assert_eq!(
            (&h.unit, h.range_min, h.range_max),
            (&a.unit, a.range_min, a.range_max),
            "{}",
            h.slug
        );
    }
}

#[test]
fn torid_all_layouts_match_analysis_ranges() {
    let c = catalog();
    let r = reference();
    for (positive_count, negative_count) in [(2, 0), (2, 1), (3, 0), (3, 1)] {
        let q = RangeHintRequest {
            weapon_slug: "torid".into(),
            mod_rank: 8,
            rule_mode: "current".into(),
            disposition: None,
            positive_count,
            negative_count,
            attributes: inputs(positive_count == 3, negative_count == 1),
        };
        compare_to_analyze(&q, &c, &r);
    }
}

#[test]
fn partial_attributes_use_declared_layout_slots() {
    let c = catalog();
    let r = reference();
    let q = RangeHintRequest {
        weapon_slug: "torid".into(),
        mod_rank: 8,
        rule_mode: "current".into(),
        disposition: None,
        positive_count: 3,
        negative_count: 1,
        attributes: vec![RangeHintInput {
            key: "cd".into(),
            slug: "critical_damage".into(),
            positive: true,
        }],
    };
    let partial = hint(q, &c, &r);
    let full = hint(
        RangeHintRequest {
            weapon_slug: "torid".into(),
            mod_rank: 8,
            rule_mode: "current".into(),
            disposition: None,
            positive_count: 3,
            negative_count: 1,
            attributes: inputs(true, true),
        },
        &c,
        &r,
    );
    let full_cd = full
        .ranges
        .iter()
        .find(|x| x.slug == "critical_damage")
        .unwrap();
    assert_eq!(
        (
            &partial.ranges[0].unit,
            partial.ranges[0].range_min,
            partial.ranges[0].range_max
        ),
        (&full_cd.unit, full_cd.range_min, full_cd.range_max)
    );
}

#[test]
fn rank_scales_percent_and_multiply_ranges() {
    let c = catalog();
    let r = reference();
    let mk = |rank: u8, slug: &str| RangeHintRequest {
        weapon_slug: "torid".into(),
        mod_rank: rank,
        rule_mode: "current".into(),
        disposition: None,
        positive_count: 2,
        negative_count: 0,
        attributes: vec![RangeHintInput {
            key: "x".into(),
            slug: slug.into(),
            positive: true,
        }],
    };
    let pslug = "critical_chance";
    let p0 = hint(mk(0, pslug), &c, &r).ranges[0].range_min.unwrap();
    let p8 = hint(mk(8, pslug), &c, &r).ranges[0].range_min.unwrap();
    assert!((p8 / p0 - 9.0).abs() < 1e-9);
    if let Some(a) = c
        .attributes
        .iter()
        .find(|a| a.game_ref.contains("FactionDamage") || a.slug.contains("faction_damage"))
    {
        let m0 = hint(mk(0, &a.slug), &c, &r).ranges[0].range_min.unwrap() - 1.0;
        let m8 = hint(mk(8, &a.slug), &c, &r).ranges[0].range_min.unwrap() - 1.0;
        assert!((m8 / m0 - 9.0).abs() < 1e-9);
    }
}

#[test]
fn unsupported_and_invalid_requests_are_safe() {
    let c = catalog();
    let r = reference();
    let mut q = RangeHintRequest {
        weapon_slug: "torid".into(),
        mod_rank: 8,
        rule_mode: "current".into(),
        disposition: None,
        positive_count: 3,
        negative_count: 1,
        attributes: vec![RangeHintInput {
            key: "x".into(),
            slug: "ComboPoints".into(),
            positive: true,
        }],
    };
    assert!(hint(q.clone(), &c, &r).ranges[0].status == "unsupported");
    q.attributes[0].slug = "reverseValueSymbol".into();
    assert_eq!(hint(q.clone(), &c, &r).ranges[0].status, "unsupported");
    q.attributes[0].slug = "critical_chance".into();
    q.attributes[0].positive = false;
    q.negative_count = 0;
    assert!(riven_analysis::range_hints(&q, &c, &r).is_err());
    q.attributes[0].positive = true;
    q.negative_count = 1;
    q.attributes[0].key = "".into();
    assert!(riven_analysis::range_hints(&q, &c, &r).is_err());
    q.attributes[0].key = "x".into();
    q.attributes.push(RangeHintInput {
        key: "x".into(),
        slug: "critical_damage".into(),
        positive: true,
    });
    assert!(riven_analysis::range_hints(&q, &c, &r).is_err());
    q.attributes = inputs(true, true);
    q.positive_count = 4;
    assert!(riven_analysis::range_hints(&q, &c, &r).is_err());
    q.positive_count = 1;
    q.negative_count = 0;
    q.attributes = vec![RangeHintInput {
        key: "x".into(),
        slug: "critical_chance".into(),
        positive: true,
    }];
    assert_eq!(hint(q, &c, &r).ranges[0].status, "invalid_layout");
}

#[test]
fn unknown_weapon_category_is_unsupported() {
    let mut c = catalog();
    c.weapons
        .iter_mut()
        .find(|w| w.slug == "torid")
        .unwrap()
        .riven_type = "unknown".into();
    let r = reference();
    let q = RangeHintRequest {
        weapon_slug: "torid".into(),
        mod_rank: 8,
        rule_mode: "current".into(),
        disposition: None,
        positive_count: 2,
        negative_count: 0,
        attributes: inputs(false, false),
    };
    assert!(hint(q, &c, &r)
        .ranges
        .iter()
        .all(|x| x.status == "unsupported"));
}
