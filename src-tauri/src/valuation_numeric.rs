use crate::{
    market::{Listing, MarketCatalog},
    riven_analysis::{self, AnalysisRequest, ReferenceData, StatAnalysisResult},
};
use std::collections::HashMap;

/// Output of the value-only comparison used by market valuation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericAdjustment {
    pub factor: f64,
    pub coverage: f64,
}

const LOG_RATIO_LIMIT: f64 = std::f64::consts::LN_2;
const FACTOR_EXPONENT: f64 = 0.35;
const FACTOR_MIN: f64 = 0.8;
const FACTOR_MAX: f64 = 1.25;
const BASE_WEIGHT: f64 = 0.25;

fn neutral() -> NumericAdjustment {
    NumericAdjustment {
        factor: 1.0,
        coverage: 0.0,
    }
}

fn magnitude(value: f64, unit: &str) -> f64 {
    if unit == "multiply" {
        (value - 1.0).abs()
    } else {
        value.abs()
    }
}

/// Compare a listing's verified positive stats to the target's verified ranges.
///
/// A listing is analyzed again with the real rules engine before any number is
/// compared. Unknown ranks, unsupported ranges, unit mismatches, and out of
/// range values are deliberately excluded rather than treated as zero.
pub fn adjustment(
    request: &AnalysisRequest,
    target: &StatAnalysisResult,
    listing: &Listing,
    catalog: &MarketCatalog,
    reference: &ReferenceData,
    frequencies: &HashMap<String, f64>,
    category_weights: &HashMap<String, f64>,
    extreme: bool,
) -> NumericAdjustment {
    let Some(rank) = listing.mod_rank.filter(|rank| *rank <= 8) else {
        return neutral();
    };
    let attrs = listing
        .attributes
        .iter()
        .map(|a| riven_analysis::AnalysisAttributeInput {
            slug: a.slug.clone(),
            value: a.value,
            positive: a.positive,
        })
        .collect();
    let listing_request = AnalysisRequest {
        weapon_slug: request.weapon_slug.clone(),
        mod_rank: rank as u8,
        rule_mode: request.rule_mode.clone(),
        disposition: request.disposition,
        attributes: attrs,
        sample_size: request.sample_size,
    };
    let Ok(reference_result) = riven_analysis::analyze(&listing_request, catalog, reference) else {
        return neutral();
    };

    let target_positive_total = target.ranges.iter().filter(|range| range.positive).count();
    if target_positive_total == 0 {
        return neutral();
    }
    let mut compared = Vec::<(f64, f64, String)>::new();
    for target_range in target.ranges.iter().filter(|range| range.positive) {
        if !matches!(target_range.status.as_str(), "within" | "at_display_limit")
            || target_range.position_percent.is_none()
        {
            continue;
        }
        let Some(reference_range) = reference_result.ranges.iter().find(|range| {
            range.slug == target_range.slug
                && range.positive
                && range.unit == target_range.unit
                && matches!(range.status.as_str(), "within" | "at_display_limit")
                && range.position_percent.is_some()
        }) else {
            continue;
        };
        let target_mag =
            magnitude(target_range.value, &target_range.unit) / (request.mod_rank as f64 + 1.0);
        let reference_mag =
            magnitude(reference_range.value, &reference_range.unit) / (rank as f64 + 1.0);
        if !target_mag.is_finite()
            || !reference_mag.is_finite()
            || target_mag <= 0.0
            || reference_mag <= 0.0
        {
            continue;
        }
        let log_ratio = (target_mag / reference_mag).ln();
        if !log_ratio.is_finite() {
            continue;
        }
        let p = frequencies.get(&target_range.slug).copied().unwrap_or(0.0);
        let p = if p.is_finite() {
            p.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let boost = category_weights
            .get(&target_range.slug)
            .copied()
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(1.0);
        let weight = BASE_WEIGHT + p * boost;
        compared.push((
            log_ratio.clamp(-LOG_RATIO_LIMIT, LOG_RATIO_LIMIT),
            weight,
            target_range.slug.clone(),
        ));
    }
    if compared.is_empty() {
        return neutral();
    }
    let weighted_mean = compared
        .iter()
        .map(|(log, weight, _)| log * weight)
        .sum::<f64>()
        / compared.iter().map(|(_, weight, _)| *weight).sum::<f64>();
    let mean = if extreme {
        let selected = compared
            .iter()
            .max_by(|a, b| {
                frequencies
                    .get(&a.2)
                    .copied()
                    .unwrap_or(0.0)
                    .partial_cmp(&frequencies.get(&b.2).copied().unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.2.cmp(&a.2))
            })
            .map(|(_, _, slug)| slug.as_str());
        let top = selected
            .and_then(|slug| compared.iter().find(|(_, _, s)| s == slug))
            .map(|x| x.0)
            .unwrap_or(weighted_mean);
        0.6 * weighted_mean + 0.4 * top
    } else {
        weighted_mean
    };
    NumericAdjustment {
        factor: (FACTOR_EXPONENT * mean).exp().clamp(FACTOR_MIN, FACTOR_MAX),
        coverage: compared.len() as f64 / target_positive_total as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::{DisplayAttribute, LocalizedName, RivenAttribute, RivenWeapon};

    fn catalog() -> MarketCatalog {
        let attrs = [
            ("critical_chance", "WeaponCritChanceMod", "percent"),
            ("critical_damage", "WeaponCritDamageMod", "percent"),
            ("recoil", "WeaponRecoilReductionMod", "percent"),
            ("damage_vs_corpus", "WeaponFactionDamageCorpus", "multiply"),
            ("ammo_efficiency", "WeaponAmmoEfficiency", "percent"),
            ("fire_rate_/_attack_speed", "WeaponFireRateMod", "percent"),
        ];
        MarketCatalog {
            weapons: vec![RivenWeapon {
                id: "torid-test".into(),
                slug: "torid".into(),
                i18n: [(
                    "en".into(),
                    LocalizedName {
                        name: "Torid".into(),
                    },
                )]
                .into_iter()
                .collect(),
                riven_type: "rifle".into(),
                group: "primary".into(),
                disposition: Some(1.0),
                game_ref: String::new(),
            }],
            attributes: attrs
                .into_iter()
                .enumerate()
                .map(|(i, (slug, game_ref, unit))| RivenAttribute {
                    id: format!("test-{i}"),
                    slug: slug.into(),
                    name: slug.into(),
                    english_name: slug.into(),
                    unit: unit.into(),
                    positive_only: false,
                    negative_only: false,
                    exclusive_to: vec![],
                    game_ref: game_ref.into(),
                    positive_is_negative: slug == "recoil",
                })
                .collect(),
        }
    }

    fn req(rank: u8, attrs: &[(&str, f64, bool)]) -> AnalysisRequest {
        AnalysisRequest {
            weapon_slug: "torid".into(),
            mod_rank: rank,
            rule_mode: "current".into(),
            disposition: None,
            attributes: attrs
                .iter()
                .map(
                    |(slug, value, positive)| riven_analysis::AnalysisAttributeInput {
                        slug: (*slug).into(),
                        value: *value,
                        positive: *positive,
                    },
                )
                .collect(),
            sample_size: 20,
        }
    }

    fn listing(rank: Option<u32>, attrs: &[(&str, f64, bool)]) -> Listing {
        Listing {
            id: "fixture".into(),
            riven_name: "fixture".into(),
            price: Some(100),
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
            mastery_rank: Some(8),
        }
    }

    fn maps() -> (HashMap<String, f64>, HashMap<String, f64>) {
        (HashMap::new(), HashMap::new())
    }

    fn analyzed(r: &AnalysisRequest, c: &MarketCatalog) -> StatAnalysisResult {
        riven_analysis::analyze(r, c, &riven_analysis::embedded_reference().unwrap()).unwrap()
    }

    #[test]
    fn higher_shared_values_raise_factor() {
        let c = catalog();
        let r = riven_analysis::embedded_reference().unwrap();
        let target_req = req(
            8,
            &[
                ("critical_chance", 180., true),
                ("critical_damage", 145., true),
            ],
        );
        let target = riven_analysis::analyze(&target_req, &c, &r).unwrap();
        assert!(target.ranges.iter().all(|x| x.position_percent.is_some()));
        let (f, w) = maps();
        let out = adjustment(
            &target_req,
            &target,
            &listing(
                Some(8),
                &[
                    ("critical_chance", 170., true),
                    ("critical_damage", 135., true),
                ],
            ),
            &c,
            &r,
            &f,
            &w,
            false,
        );
        assert!(out.factor > 1.0 && out.coverage > 0.0);
    }

    #[test]
    fn rank_normalization_and_missing_rank() {
        let c = catalog();
        let r = riven_analysis::embedded_reference().unwrap();
        let target_req = req(
            0,
            &[
                ("critical_chance", 20., true),
                ("critical_damage", 145. / 9., true),
            ],
        );
        let target = analyzed(&target_req, &c);
        let (f, w) = maps();
        let same = adjustment(
            &target_req,
            &target,
            &listing(
                Some(8),
                &[
                    ("critical_chance", 180., true),
                    ("critical_damage", 145., true),
                ],
            ),
            &c,
            &r,
            &f,
            &w,
            false,
        );
        assert!((same.factor - 1.0).abs() < 1e-6);
        let missing = adjustment(
            &target_req,
            &target,
            &listing(
                None,
                &[
                    ("critical_chance", 180., true),
                    ("critical_damage", 145., true),
                ],
            ),
            &c,
            &r,
            &f,
            &w,
            false,
        );
        assert_eq!(
            missing,
            NumericAdjustment {
                factor: 1.0,
                coverage: 0.0
            }
        );
    }

    #[test]
    fn multiply_uses_delta_from_one() {
        let c = catalog();
        let r = riven_analysis::embedded_reference().unwrap();
        let (f, w) = maps();
        let q = req(
            8,
            &[
                ("critical_chance", 180., true),
                ("damage_vs_corpus", 1.58, true),
            ],
        );
        let t = analyzed(&q, &c);
        let out = adjustment(
            &q,
            &t,
            &listing(
                Some(8),
                &[
                    ("critical_chance", 180., true),
                    ("damage_vs_corpus", 1.50, true),
                ],
            ),
            &c,
            &r,
            &f,
            &w,
            false,
        );
        assert!(out.factor > 1.01);
    }

    #[test]
    fn signed_recoil_compares_magnitude() {
        let c = catalog();
        let r = riven_analysis::embedded_reference().unwrap();
        let (f, w) = maps();
        let q = req(
            8,
            &[("critical_chance", 180., true), ("recoil", -110., true)],
        );
        let t = analyzed(&q, &c);
        let out = adjustment(
            &q,
            &t,
            &listing(
                Some(8),
                &[("critical_chance", 180., true), ("recoil", -100., true)],
            ),
            &c,
            &r,
            &f,
            &w,
            false,
        );
        assert!(out.factor > 1.01);
    }

    #[test]
    fn unsupported_shared_stat_is_skipped() {
        let c = catalog();
        let r = riven_analysis::embedded_reference().unwrap();
        let (f, w) = maps();
        let q = req(
            8,
            &[
                ("critical_chance", 140., true),
                ("critical_damage", 110., true),
                ("ammo_efficiency", 80., true),
            ],
        );
        let t = analyzed(&q, &c);
        assert!(t
            .ranges
            .iter()
            .any(|x| x.slug == "ammo_efficiency" && x.status == "unsupported"));
        let out = adjustment(
            &q,
            &t,
            &listing(
                Some(8),
                &[
                    ("critical_chance", 135., true),
                    ("critical_damage", 105., true),
                    ("ammo_efficiency", 75., true),
                ],
            ),
            &c,
            &r,
            &f,
            &w,
            false,
        );
        assert!((out.coverage - 2. / 3.).abs() < 1e-9);
    }

    #[test]
    fn extreme_follows_highest_frequency_not_highest_roll() {
        let c = catalog();
        let r = riven_analysis::embedded_reference().unwrap();
        let (mut f, w) = maps();
        f.insert("critical_chance".into(), 0.9);
        f.insert("critical_damage".into(), 0.1);
        f.insert("fire_rate_/_attack_speed".into(), 0.0);
        let q = req(
            8,
            &[
                ("critical_chance", 207., true),
                ("critical_damage", 193., true),
                ("fire_rate_/_attack_speed", -35., false),
            ],
        );
        let t = analyzed(&q, &c);
        assert!(t.ranges.iter().all(|x| x.position_percent.is_some()));
        let l = listing(
            Some(8),
            &[
                ("critical_chance", 237., true),
                ("critical_damage", 162., true),
                ("fire_rate_/_attack_speed", -35., false),
            ],
        );
        let balanced = adjustment(&q, &t, &l, &c, &r, &f, &w, false);
        let extreme = adjustment(&q, &t, &l, &c, &r, &f, &w, true);
        assert!(extreme.factor < balanced.factor);
        assert!(extreme.factor.is_finite());
    }
}
