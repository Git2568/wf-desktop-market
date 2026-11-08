use crate::{
    analysis_service::WeeklyWeaponSummary,
    market::{Listing, MarketCatalog},
    riven_analysis::{self, AnalysisRequest, ReferenceData},
};
use serde::Serialize;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

pub const RULE_VERSION: &str = "sample-heuristic-v1";
const SIMILARITY_THRESHOLD: f64 = 0.8;
const JACCARD_SIMILARITY_WEIGHT: f64 = 0.8;
const JACCARD_BASE_WEIGHT: f64 = 0.25;
const SIMILARITY_BONUS: f64 = 0.1;
const NEAREST_GAP: f64 = 0.15;
const POPULARITY_COEFFICIENT: f64 = 0.30;
const POPULARITY_MIN: f64 = 0.75;
const POPULARITY_MAX: f64 = 1.35;
const HEAT_COEFFICIENT: f64 = 0.02;
const SIMILAR_WIDTH: f64 = 0.25;
const NEAREST_WIDTH: f64 = 0.40;
const MEDIUM_WIDTH: f64 = 0.15;
const STALE_WIDTH_ADD: f64 = 0.10;
const LOW_COVERAGE_WIDTH_ADD: f64 = 0.05;
const MAX_WIDTH: f64 = 0.60;

#[derive(Debug, Clone, Serialize)]
pub struct HotStat {
    pub slug: String,
    pub name: String,
    pub count: u32,
    pub frequency: f64,
}
#[derive(Debug, Clone, Serialize)]
pub struct ValuationReference {
    pub id: String,
    pub price: u32,
    pub similarity: f64,
    pub adjusted_price: f64,
    pub popularity_factor: f64,
    pub numeric_factor: f64,
    pub numeric_coverage: f64,
    pub positive_matches: u32,
    pub positive_total: u32,
    pub negative_match: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct ValuationResult {
    pub rule_version: String,
    pub status: String,
    pub method: String,
    pub estimate: Option<f64>,
    pub low: Option<f64>,
    pub high: Option<f64>,
    pub confidence: String,
    pub pool_count: u32,
    pub comparable_count: u32,
    pub hot_stats: Vec<HotStat>,
    pub popularity_factor: f64,
    pub numeric_factor: f64,
    pub heat_factor: f64,
    pub numeric_coverage: f64,
    pub layout_focus: String,
    pub references: Vec<ValuationReference>,
    pub warnings: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Clone)]
struct Card {
    l: Listing,
    pos: Vec<String>,
    neg: Option<String>,
}
fn unavailable(reason: &str, warnings: Vec<String>) -> ValuationResult {
    ValuationResult {
        rule_version: RULE_VERSION.into(),
        status: "unavailable".into(),
        method: "none".into(),
        estimate: None,
        low: None,
        high: None,
        confidence: "low".into(),
        pool_count: 0,
        comparable_count: 0,
        hot_stats: vec![],
        popularity_factor: 1.0,
        numeric_factor: 1.0,
        heat_factor: 1.0,
        numeric_coverage: 0.0,
        layout_focus: "balanced".into(),
        references: vec![],
        warnings,
        reason: Some(reason.into()),
    }
}
const DAMAGE_GAME_REFS: &[&str] = &[
    "WeaponDamageAmountMod",
    "WeaponMeleeDamageMod",
    "WeaponCritChanceMod",
    "WeaponCritDamageMod",
    "WeaponWeakpointDamage",
    "WeaponWeakpointCriticalChance",
    "WeaponFireIterationsMod",
    "WeaponFireDamageMod",
    "WeaponFreezeDamageMod",
    "WeaponElectricityDamageMod",
    "WeaponToxinDamageMod",
    "WeaponSlashDamageMod",
    "WeaponImpactDamageMod",
    "WeaponArmorPiercingDamageMod",
];
const EXTREME_GAME_REFS: &[&str] = &[
    "WeaponAmmoEfficiency",
    "WeaponReloadSpeedMod",
    "WeaponFireRateMod",
    "WeaponClipMaxMod",
    "WeaponAmmoMaxMod",
    "WeaponRecoilReductionMod",
    "WeaponProjectileSpeedMod",
];

fn focus(req: &AnalysisRequest) -> String {
    let p = req.attributes.iter().filter(|a| a.positive).count();
    let n = req.attributes.iter().filter(|a| !a.positive).count();
    if p == 3 && n == 1 {
        "damage".into()
    } else if p == 2 && n == 1 {
        "extreme".into()
    } else {
        "balanced".into()
    }
}
fn category(catalog: &MarketCatalog, slug: &str, focus: &str) -> f64 {
    let g = catalog
        .attributes
        .iter()
        .find(|x| x.slug == slug)
        .map(|x| x.game_ref.as_str())
        .unwrap_or("");
    if focus == "damage" && DAMAGE_GAME_REFS.contains(&g) {
        1.2
    } else if focus == "extreme" && EXTREME_GAME_REFS.contains(&g) {
        1.2
    } else {
        1.0
    }
}
fn median(mut xs: Vec<(f64, f64)>) -> f64 {
    xs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    let t = xs.iter().map(|x| x.1).sum::<f64>() / 2.0;
    let mut s = 0.;
    for (v, w) in xs {
        s += w;
        if s >= t {
            return v;
        }
    }
    0.
}
fn round_price(v: f64) -> f64 {
    if v >= 50. {
        (v / 5.).round() * 5.
    } else {
        v.round()
    }
}
fn weighted_quantile(mut xs: Vec<(f64, f64)>, q: f64) -> f64 {
    xs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    let total = xs.iter().map(|x| x.1).sum::<f64>();
    let mut n = 0.;
    for (v, w) in xs {
        n += w;
        if n >= total * q {
            return v;
        }
    }
    0.
}
fn floor_price(v: f64) -> f64 {
    let x = if v >= 50. {
        (v / 5.).floor() * 5.
    } else {
        v.floor()
    };
    x.max(1.)
}
fn ceil_price(v: f64) -> f64 {
    let x = if v >= 50. {
        (v / 5.).ceil() * 5.
    } else {
        v.ceil()
    };
    x.max(1.)
}

pub fn estimate(
    req: &AnalysisRequest,
    catalog: &MarketCatalog,
    reference: &ReferenceData,
    listings: &[Listing],
    weekly: Option<&WeeklyWeaponSummary>,
    stale: bool,
) -> ValuationResult {
    let target = match riven_analysis::analyze(req, catalog, reference) {
        Ok(v) => v,
        Err(e) => {
            return unavailable(
                &format!("无法分析当前紫卡：{e}"),
                vec!["请检查数值、强化等级和模式".into()],
            )
        }
    };
    if target.ranges.iter().any(|x| x.status == "out_of_range") {
        return unavailable(
            "当前紫卡存在超出规则范围的数值，请检查数值、强化等级和模式",
            vec![],
        );
    }
    let mut seen = HashSet::new();
    let mut pool = Vec::new();
    for l in listings {
        if l.price_kind != "一口价" || l.price.unwrap_or(0) == 0 || !seen.insert(l.id.clone()) {
            continue;
        }
        if l.attributes.iter().any(|a| !a.value.is_finite()) {
            continue;
        }
        let mut unique = HashSet::new();
        if l.attributes.iter().any(|a| !unique.insert(a.slug.clone())) {
            continue;
        }
        let pos = l
            .attributes
            .iter()
            .filter(|a| a.positive)
            .map(|a| a.slug.clone())
            .collect::<Vec<_>>();
        let negatives = l
            .attributes
            .iter()
            .filter(|a| !a.positive)
            .collect::<Vec<_>>();
        if negatives.len() > 1 {
            continue;
        }
        let neg = l
            .attributes
            .iter()
            .find(|a| !a.positive)
            .map(|a| a.slug.clone());
        if (2..=3).contains(&pos.len()) && neg.is_some() as usize <= 1 {
            pool.push(Card {
                l: l.clone(),
                pos,
                neg,
            })
        }
    }
    let focus = focus(req);
    let mut hot = HashMap::<String, u32>::new();
    for c in &pool {
        for x in &c.pos {
            *hot.entry(x.clone()).or_default() += 1
        }
    }
    let mut hot_stats = hot
        .iter()
        .map(|(s, c)| HotStat {
            slug: s.clone(),
            name: catalog
                .attributes
                .iter()
                .find(|a| &a.slug == s)
                .map(|a| a.name.clone())
                .unwrap_or_else(|| s.clone()),
            count: *c,
            frequency: *c as f64 / pool.len().max(1) as f64,
        })
        .collect::<Vec<_>>();
    hot_stats.sort_by(|a, b| b.count.cmp(&a.count).then(a.slug.cmp(&b.slug)));
    hot_stats.truncate(6);
    if pool.is_empty() {
        return ValuationResult {
            pool_count: 0,
            hot_stats,
            layout_focus: focus,
            ..unavailable("当前武器没有可用的一口价样本，无法估价", vec![])
        };
    }
    let target_pos = req
        .attributes
        .iter()
        .filter(|a| a.positive)
        .map(|a| a.slug.clone())
        .collect::<Vec<_>>();
    let target_neg = req
        .attributes
        .iter()
        .find(|a| !a.positive)
        .map(|a| a.slug.clone());
    let pool_count = pool.len();
    let mut scored = Vec::new();
    for c in pool {
        let a: HashSet<&String> = target_pos.iter().collect();
        let b: HashSet<&String> = c.pos.iter().collect();
        let union: HashSet<&String> = a.union(&b).copied().collect();
        let inter: HashSet<&String> = a.intersection(&b).copied().collect();
        let jw = |s: &String| {
            let p = *hot.get(s).unwrap_or(&0) as f64 / pool_count.max(1) as f64;
            JACCARD_BASE_WEIGHT + p
        };
        let den = union.iter().map(|s| jw(s)).sum::<f64>();
        let num = inter.iter().map(|s| jw(s)).sum::<f64>();
        let j = if den > 0. { num / den } else { 0. };
        let same = target_pos.len() == c.pos.len() && target_neg.is_some() == c.neg.is_some();
        let neg = target_neg == c.neg;
        let sim = (JACCARD_SIMILARITY_WEIGHT * j
            + SIMILARITY_BONUS * same as u8 as f64
            + SIMILARITY_BONUS * neg as u8 as f64)
            .clamp(0., 1.);
        scored.push((sim, same, neg, c));
    }
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(Ordering::Equal)
            .then(a.3.l.id.cmp(&b.3.l.id))
    });
    let high = scored
        .iter()
        .filter(|x| x.0 >= SIMILARITY_THRESHOLD && x.1 && x.2)
        .cloned()
        .collect::<Vec<_>>();
    let method = if !high.is_empty() {
        "similar"
    } else {
        "nearest"
    };
    let mut refs = if !high.is_empty() {
        high
    } else {
        let best = scored.first().map(|x| x.0).unwrap_or(0.);
        scored
            .iter()
            .filter(|x| best - x.0 <= NEAREST_GAP)
            .take(5)
            .cloned()
            .collect()
    };
    refs.truncate(5);
    let ref_count = refs.len();
    let mut vals = Vec::new();
    let mut outrefs = Vec::new();
    let mut weighted_numeric_cov = 0.;
    let mut weighted_numeric_fac = 0.;
    let mut weighted_popularity = 0.;
    let mut weighted_heat = 0.;
    let mut total_weight = 0.;
    for (s, _, _, c) in refs {
        let mut pop = 0.;
        for x in &target_pos {
            pop +=
                *hot.get(x).unwrap_or(&0) as f64 / pool_count as f64 * category(catalog, x, &focus)
        }
        for x in &c.pos {
            pop -=
                *hot.get(x).unwrap_or(&0) as f64 / pool_count as f64 * category(catalog, x, &focus)
        }
        let pf = (POPULARITY_COEFFICIENT * pop)
            .exp()
            .clamp(POPULARITY_MIN, POPULARITY_MAX);
        let hf = 1.
            + HEAT_COEFFICIENT
                * weekly
                    .map(|w| {
                        w.groups
                            .iter()
                            .map(|g| g.popularity)
                            .fold(0., f64::max)
                            .clamp(0., 100.)
                            / 100.
                    })
                    .unwrap_or(0.);
        let freq = hot
            .iter()
            .map(|(k, v)| (k.clone(), *v as f64 / pool_count.max(1) as f64))
            .collect();
        let categories = hot
            .keys()
            .map(|slug| (slug.clone(), category(catalog, slug, &focus)))
            .collect::<HashMap<_, _>>();
        let na = crate::valuation_numeric::adjustment(
            req,
            &target,
            &c.l,
            catalog,
            reference,
            &freq,
            &categories,
            focus == "extreme",
        );
        let (nf, ncov) = (na.factor, na.coverage);
        let weight = (SIMILARITY_BONUS + s).powi(3);
        weighted_numeric_cov += ncov * weight;
        weighted_numeric_fac += nf * weight;
        weighted_popularity += pf * weight;
        weighted_heat += hf * weight;
        total_weight += weight;
        let adj = c.l.price.unwrap_or(0) as f64 * pf * nf * hf;
        vals.push((adj, (0.1 + s).powi(3)));
        outrefs.push(ValuationReference {
            id: c.l.id,
            price: c.l.price.unwrap_or(0),
            similarity: s,
            adjusted_price: adj,
            popularity_factor: pf,
            numeric_factor: nf,
            numeric_coverage: ncov,
            positive_matches: c.pos.iter().filter(|x| target_pos.contains(x)).count() as u32,
            positive_total: target_pos.len() as u32,
            negative_match: target_neg == c.neg,
        });
    }
    let est = round_price(median(vals.clone()));
    let mut sorted = vals;
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let q20 = weighted_quantile(sorted.clone(), 0.2);
    let q80 = weighted_quantile(sorted.clone(), 0.8);
    let divisor = total_weight.max(f64::EPSILON);
    let numeric_coverage = weighted_numeric_cov / divisor;
    let numeric_factor = weighted_numeric_fac / divisor;
    let medium = method == "similar"
        && pool_count >= 20
        && ref_count >= 3
        && numeric_coverage >= 0.5
        && !stale;
    let mut width: f64 = if medium {
        MEDIUM_WIDTH
    } else if method == "similar" {
        SIMILAR_WIDTH
    } else {
        NEAREST_WIDTH
    };
    if stale {
        width += STALE_WIDTH_ADD
    };
    width = width.min(MAX_WIDTH);
    if numeric_coverage < 0.5 {
        width = (width + LOW_COVERAGE_WIDTH_ADD).min(MAX_WIDTH);
    }
    let low = floor_price((q20.min(est * (1. - width))).max(1.));
    let high = ceil_price(q80.max(est * (1. + width)).max(est));
    let mut warnings = vec!["价格区间基于当前挂牌，不代表成交价".into()];
    if ref_count < 3 {
        warnings.push("可比参考少于 3 张，置信度较低".into());
    }
    if method == "nearest" {
        warnings.push("未找到足够相似的同布局参考，已使用最近样本".into());
    }
    if numeric_coverage < 1.0 {
        warnings.push("部分参考卡未完成数值比较".into());
    }
    if numeric_coverage < 0.5 {
        warnings.push("部分参考卡缺少可比数值，数值调整覆盖不足".into());
    }
    if stale {
        warnings.push("挂单样本已过期，区间已扩大".into());
    }
    if outrefs.iter().any(|x| !x.negative_match) {
        warnings.push("参考卡负面不同，未自动估算其对该武器的损失".into());
    }
    warnings.push("尚无成交回测校准，启发式结果不代表市场公允价".into());
    ValuationResult {
        rule_version: RULE_VERSION.into(),
        status: "estimated".into(),
        method: method.into(),
        estimate: Some(est),
        low: Some(low.min(est)),
        high: Some(high.max(est)),
        confidence: if medium {
            "medium".into()
        } else {
            "low".into()
        },
        pool_count: pool_count as u32,
        comparable_count: outrefs.len() as u32,
        hot_stats,
        popularity_factor: weighted_popularity / divisor,
        numeric_factor,
        heat_factor: weighted_heat / divisor,
        numeric_coverage,
        layout_focus: focus,
        references: outrefs,
        warnings,
        reason: None,
    }
}
