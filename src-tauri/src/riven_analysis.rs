use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::market::MarketCatalog;
use crate::riven_roll_data::{self, Layout as RollLayout, RivenClass};

pub const REFERENCE_SEED: &str = include_str!("../data/riven-analysis-reference.json");

fn roll_class(class: &str) -> Option<RivenClass> {
    Some(match class {
        "rifle" => RivenClass::Rifle,
        "shotgun" => RivenClass::Shotgun,
        "pistol" => RivenClass::Pistol,
        "kitgun" => RivenClass::Kitgun,
        "archgun" => RivenClass::Archgun,
        "melee" => RivenClass::Melee,
        "zaw" => RivenClass::Zaw,
        _ => return None,
    })
}
fn spliced_range(
    class: &str,
    tag: &str,
    pos: u8,
    neg: u8,
    rank: u8,
    d: f64,
) -> Option<(String, f64, f64, f64)> {
    let cls = roll_class(class)?;
    let s = riven_roll_data::stat(cls, tag)?;
    if !s.spliced {
        return None;
    }
    let l = RollLayout { pos, neg };
    let lo =
        riven_roll_data::display_number(s, riven_roll_data::raw_value(s, l, false, 0.9, rank, d));
    let hi =
        riven_roll_data::display_number(s, riven_roll_data::raw_value(s, l, false, 1.1, rank, d));
    let unit = match s.fmt {
        riven_roll_data::StatFormat::Pct => "percent",
        riven_roll_data::StatFormat::Mult => "multiply",
        riven_roll_data::StatFormat::Sec => "seconds",
        riven_roll_data::StatFormat::Flat => "flat",
    }
    .to_string();
    // The game's improvement direction is the display value at W=1.10.  This
    // matters for negative-base traits such as AmmoEfficiency, whose formatter
    // reverses the sign before display.
    Some((unit, lo.min(hi), lo.max(hi), hi))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub weapon_slug: String,
    pub mod_rank: u8,
    pub rule_mode: String,
    pub disposition: Option<f64>,
    pub attributes: Vec<AnalysisAttributeInput>,
    pub sample_size: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisAttributeInput {
    pub slug: String,
    pub value: f64,
    pub positive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeHintRequest {
    pub weapon_slug: String,
    pub mod_rank: u8,
    pub rule_mode: String,
    pub disposition: Option<f64>,
    pub positive_count: u8,
    pub negative_count: u8,
    pub attributes: Vec<RangeHintInput>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeHintInput {
    pub key: String,
    pub slug: String,
    pub positive: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeHintsResult {
    pub ranges: Vec<RangeHint>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeHint {
    pub key: String,
    pub slug: String,
    pub positive: bool,
    pub unit: String,
    pub range_min: Option<f64>,
    pub range_max: Option<f64>,
    pub status: String,
    pub message: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatAnalysisResult {
    pub weapon_slug: String,
    pub weapon_name: String,
    pub rule_mode: String,
    pub rule_version: String,
    pub reference_version: String,
    pub disposition: f64,
    pub mod_rank: u8,
    pub ranges: Vec<StatRangeResult>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatRangeResult {
    pub slug: String,
    pub name: String,
    pub positive: bool,
    pub value: f64,
    #[serde(deserialize_with = "deserialize_unit")]
    pub unit: String,
    pub range_min: Option<f64>,
    pub range_max: Option<f64>,
    pub best_value: Option<f64>,
    pub position_percent: Option<f64>,
    pub status: String,
    pub message: Option<String>,
}

fn deserialize_unit<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

fn deserialize_exclusive<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<Vec<Option<String>>>::deserialize(deserializer)?
        .unwrap_or_default()
        .into_iter()
        .flatten()
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceData {
    pub schema: String,
    pub rule_version: String,
    pub reference_version: String,
    pub retrieved_at: String,
    pub sources: Vec<ReferenceSource>,
    pub types: HashMap<String, Vec<ReferenceCoefficient>>,
    pub attributes: HashMap<String, ReferenceAttribute>,
    pub aliases: HashMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceSource {
    pub url: String,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceCoefficient {
    pub tag: String,
    pub base: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceAttribute {
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    #[serde(deserialize_with = "deserialize_unit")]
    pub unit: String,
    #[serde(rename = "positiveOnly", default)]
    pub positive_only: bool,
    #[serde(rename = "negativeOnly", default)]
    pub negative_only: bool,
    #[serde(rename = "positiveIsNegative", default)]
    pub positive_is_negative: bool,
    #[serde(
        rename = "exclusiveTo",
        default,
        deserialize_with = "deserialize_exclusive"
    )]
    pub exclusive_to: Vec<String>,
}

pub fn embedded_reference() -> Result<ReferenceData, String> {
    let r: ReferenceData = serde_json::from_str(REFERENCE_SEED)
        .map_err(|e| format!("invalid embedded riven reference: {e}"))?;
    validate_reference(&r)?;
    Ok(r)
}

pub fn validate_reference(r: &ReferenceData) -> Result<(), String> {
    if r.schema != "wm.riven-analysis-reference.v1" || r.rule_version != "44.1.3" {
        return Err("unsupported riven reference schema/rule version".into());
    }
    if r.reference_version.is_empty() || r.retrieved_at.is_empty() || r.sources.is_empty() {
        return Err("riven reference provenance is incomplete".into());
    }
    for source in &r.sources {
        if source.url.is_empty() || source.sha256.len() != 64 {
            return Err("riven reference source hash is invalid".into());
        }
    }
    if r.types.is_empty() {
        return Err("riven reference has no coefficient tables".into());
    }
    if r.types.len() != 7 || r.types.values().any(Vec::is_empty) {
        return Err("riven reference coefficient coverage is incomplete".into());
    }
    for (kind, rows) in &r.types {
        if rows.is_empty() {
            return Err(format!("empty coefficient table {kind}"));
        }
        let mut tags = HashSet::new();
        for row in rows {
            if row.tag.is_empty()
                || !row.base.is_finite()
                || row.base == 0.0
                || !tags.insert(&row.tag)
            {
                return Err(format!("invalid or duplicate coefficient in {kind}"));
            }
        }
    }
    Ok(())
}

fn layout_factors(positives: usize, negatives: usize) -> Option<(f64, Option<f64>)> {
    match (positives, negatives) {
        (2, 0) => Some((0.99, None)),
        (2, 1) => Some((1.2375, Some(-0.495))),
        (3, 0) => Some((0.75, None)),
        (3, 1) => Some((0.9375, Some(-0.75))),
        _ => None,
    }
}

fn theoretical_value(base: f64, rank: u8, disposition: f64, factor: f64, roll: f64) -> f64 {
    base * 10.0 * (rank as f64 + 1.0) * disposition * factor * roll
}

pub fn validate_request(r: &AnalysisRequest, catalog: &MarketCatalog) -> Result<(), String> {
    if !matches!(r.rule_mode.as_str(), "current" | "legacy") {
        return Err("规则模式必须是 current 或 legacy".into());
    }
    if r.mod_rank > 8 {
        return Err("段位必须是 0 到 8".into());
    }
    if !(1..=50).contains(&r.sample_size) {
        return Err("样本数必须是 1 到 50".into());
    }
    if r.attributes.len() < 2 || r.attributes.len() > 4 {
        return Err("紫卡必须有 2 到 3 个正面词条和 0 到 1 个负面词条".into());
    }
    let pos = r.attributes.iter().filter(|a| a.positive).count();
    let neg = r.attributes.len() - pos;
    if !(2..=3).contains(&pos) || neg > 1 {
        return Err("Riven layout must contain 2-3 positives and 0-1 negative".into());
    }
    if r.attributes.iter().any(|a| !a.value.is_finite()) {
        return Err("词条数值必须是有限数".into());
    }
    let mut seen = HashSet::new();
    if r.attributes.iter().any(|a| !seen.insert(&a.slug)) {
        return Err("词条不能重复".into());
    }
    let weapon = catalog
        .weapons
        .iter()
        .find(|w| w.slug == r.weapon_slug)
        .ok_or_else(|| "未找到紫卡武器".to_string())?;
    let effective_class = if weapon.group == "archgun" {
        "archgun"
    } else {
        weapon.riven_type.as_str()
    };
    if r.rule_mode == "legacy" {
        let d = r.disposition.or(weapon.disposition).unwrap_or(0.0);
        if !(0.5..=1.55).contains(&d) || !d.is_finite() {
            return Err("历史倾向必须在 0.5 到 1.55 之间".into());
        }
    }
    for a in &r.attributes {
        let x = catalog
            .attributes
            .iter()
            .find(|x| x.slug == a.slug)
            .ok_or_else(|| format!("未找到词条 {}", a.slug))?;
        if a.positive && x.negative_only {
            return Err(format!("{} 只能作为负面词条", a.slug));
        }
        if !a.positive && x.positive_only {
            return Err(format!("{} 只能作为正面词条", a.slug));
        }
        let class_known = matches!(
            effective_class,
            "archgun" | "zaw" | "kitgun" | "melee" | "pistol" | "shotgun" | "rifle"
        );
        if class_known
            && !x.exclusive_to.is_empty()
            && !x
                .exclusive_to
                .iter()
                .any(|t| t == effective_class || t == "default")
        {
            return Err(format!("{} 与武器类型不兼容", a.slug));
        }
        if let Some(cls) = roll_class(effective_class) {
            if let Some(s) = riven_roll_data::stat(cls, &x.game_ref) {
                if s.spliced && !a.positive {
                    return Err(format!("{} 熔接词条只能作为正面词条", a.slug));
                }
            }
        }
    }
    let spliced_count = r
        .attributes
        .iter()
        .filter(|a| {
            catalog
                .attributes
                .iter()
                .find(|x| x.slug == a.slug)
                .and_then(|x| {
                    roll_class(effective_class).and_then(|c| riven_roll_data::stat(c, &x.game_ref))
                })
                .is_some_and(|s| s.spliced)
        })
        .count();
    if spliced_count > 1 {
        return Err("每张紫卡最多只能有一条熔接词条".into());
    }
    Ok(())
}

pub fn analyze(
    r: &AnalysisRequest,
    catalog: &MarketCatalog,
    reference: &ReferenceData,
) -> Result<StatAnalysisResult, String> {
    validate_request(r, catalog)?;
    let weapon = catalog
        .weapons
        .iter()
        .find(|w| w.slug == r.weapon_slug)
        .ok_or_else(|| "未找到紫卡武器".to_string())?;
    let d = if r.rule_mode == "current" {
        1.2
    } else {
        r.disposition
            .or(weapon.disposition)
            .ok_or_else(|| "历史倾向不可用".to_string())?
    };
    let pos = r.attributes.iter().filter(|a| a.positive).count();
    let neg = r.attributes.len() - pos;
    let (lp, ln) = layout_factors(pos, neg).ok_or_else(|| "不支持的紫卡词条布局".to_string())?;
    let class = if weapon.group == "archgun" {
        "archgun"
    } else {
        weapon.riven_type.as_str()
    };
    let type_key = match class {
        "archgun" => "LotusArchgunRandomModRare",
        "zaw" => "LotusModularMeleeRandomModRare",
        "kitgun" => "LotusModularPistolRandomModRare",
        "melee" => "PlayerMeleeWeaponRandomModRare",
        "pistol" => "LotusPistolRandomModRare",
        "shotgun" => "LotusShotgunRandomModRare",
        "rifle" => "LotusRifleRandomModRare",
        _ => "",
    };
    let coeffs = reference.types.get(type_key);
    let mut ranges = Vec::new();
    for a in &r.attributes {
        let catalog_attr = catalog.attributes.iter().find(|x| x.slug == a.slug);
        let ra = reference.attributes.get(&a.slug);
        let unit_verified = ra
            .map(|x| matches!(x.unit.as_str(), "" | "percent" | "multiply" | "seconds"))
            .unwrap_or(false);
        if !unit_verified {
            ranges.push(StatRangeResult {
                slug: a.slug.clone(),
                name: catalog_attr
                    .map(|x| x.name.clone())
                    .unwrap_or_else(|| a.slug.clone()),
                positive: a.positive,
                value: a.value,
                unit: "unknown".into(),
                range_min: None,
                range_max: None,
                best_value: None,
                position_percent: None,
                status: "unsupported".into(),
                message: Some("未核实该词条的属性元数据或单位".into()),
            });
            continue;
        }
        let tag0 = catalog_attr
            .map(|x| x.game_ref.as_str())
            .filter(|x| !x.is_empty())
            .or_else(|| ra.map(|x| x.game_ref.as_str()))
            .unwrap_or("");
        let tag = if weapon.riven_type == "melee" || weapon.riven_type == "zaw" {
            reference
                .aliases
                .get(tag0)
                .map(String::as_str)
                .unwrap_or(tag0)
        } else {
            tag0
        };
        if let Some((unit, lo2, hi2, best)) =
            spliced_range(&class, tag, pos as u8, neg as u8, r.mod_rank, d)
        {
            if !a.positive {
                ranges.push(StatRangeResult {
                    slug: a.slug.clone(),
                    name: a.slug.clone(),
                    positive: a.positive,
                    value: a.value,
                    unit,
                    range_min: None,
                    range_max: None,
                    best_value: None,
                    position_percent: None,
                    status: "unsupported".into(),
                    message: Some("熔接词条只能作为正面词条".into()),
                });
            } else {
                let p = roll_class(&class)
                    .and_then(|cls| riven_roll_data::stat(cls, tag))
                    .and_then(|s| {
                        let q = riven_roll_data::quality_from_display(
                            s,
                            RollLayout {
                                pos: pos as u8,
                                neg: neg as u8,
                            },
                            false,
                            a.value,
                            r.mod_rank,
                            d,
                        );
                        q.valid.then_some(
                            ((q.w - riven_roll_data::W_MIN)
                                / (riven_roll_data::W_MAX - riven_roll_data::W_MIN)
                                * 100.0)
                                .clamp(0.0, 100.0),
                        )
                    });
                ranges.push(StatRangeResult {
                    slug: a.slug.clone(),
                    name: a.slug.clone(),
                    positive: true,
                    value: a.value,
                    unit,
                    range_min: Some(lo2),
                    range_max: Some(hi2),
                    best_value: Some(best),
                    position_percent: p,
                    status: if a.value < lo2 || a.value > hi2 {
                        "out_of_range"
                    } else {
                        "within"
                    }
                    .into(),
                    message: None,
                });
            }
            continue;
        }
        let reverse_unverified = tag == "WeaponMeleeComboPointsOnHitMod";
        let c = if reverse_unverified {
            None
        } else {
            coeffs.and_then(|rows| rows.iter().find(|x| x.tag == tag))
        };
        let name = catalog
            .attributes
            .iter()
            .find(|x| x.slug == a.slug)
            .map(|x| x.name.clone())
            .unwrap_or_else(|| a.slug.clone());
        let unit = ra
            .map(|x| x.unit.clone())
            .unwrap_or_else(|| "unknown".into());
        let Some(c) = c else {
            ranges.push(StatRangeResult {
                slug: a.slug.clone(),
                name,
                positive: a.positive,
                value: a.value,
                unit,
                range_min: None,
                range_max: None,
                best_value: None,
                position_percent: None,
                status: "unsupported".into(),
                message: Some(
                    if reverse_unverified {
                        "未核实该词条的反向显示规则"
                    } else if type_key.is_empty() {
                        "未核实该武器类型的计算规则"
                    } else {
                        "未核实该词条的基数或映射"
                    }
                    .into(),
                ),
            });
            continue;
        };
        let factor = if a.positive { lp } else { ln.unwrap() };
        let (mut lo, mut hi) = (
            theoretical_value(c.base, r.mod_rank, d, factor, 0.9),
            theoretical_value(c.base, r.mod_rank, d, factor, 1.1),
        );
        if lo > hi {
            std::mem::swap(&mut lo, &mut hi);
        }
        let transform = |v: f64| {
            if unit == "multiply" {
                1.0 + v
            } else if unit == "percent" {
                v * 100.0
            } else {
                v
            }
        };
        let (lo2, hi2, val2) = (transform(lo), transform(hi), a.value);
        let best = if c.base >= 0.0 { hi2 } else { lo2 };
        let worst = if (best - hi2).abs() < f64::EPSILON {
            lo2
        } else {
            hi2
        };
        let span = (best - worst).abs();
        let in_range = val2 >= lo2 - if unit == "multiply" { 0.005 } else { 0.05 }
            && val2 <= hi2 + if unit == "multiply" { 0.005 } else { 0.05 };
        let p = if span > 0.0 && in_range {
            Some(((val2 - worst) / (best - worst) * 100.0).clamp(0.0, 100.0))
        } else {
            None
        };
        let tol = if unit == "multiply" { 0.005 } else { 0.05 };
        let status = if !in_range {
            "out_of_range"
        } else if (val2 - best).abs() <= tol {
            "at_display_limit"
        } else {
            "within"
        };
        ranges.push(StatRangeResult {
            slug: a.slug.clone(),
            name,
            positive: a.positive,
            value: a.value,
            unit,
            range_min: Some(lo2),
            range_max: Some(hi2),
            best_value: Some(best),
            position_percent: p,
            status: status.into(),
            message: None,
        });
    }
    let warnings = if r.rule_mode == "legacy" {
        vec![
            "历史模式仅使用现行词条规则与用户提供的历史倾向倍率模拟；不代表历史熔接数据存在".into(),
        ]
    } else {
        vec![]
    };
    Ok(StatAnalysisResult {
        weapon_slug: r.weapon_slug.clone(),
        weapon_name: weapon
            .i18n
            .get("zh-hans")
            .or_else(|| weapon.i18n.get("en"))
            .map(|x| x.name.clone())
            .unwrap_or_else(|| weapon.slug.clone()),
        rule_mode: r.rule_mode.clone(),
        rule_version: reference.rule_version.clone(),
        reference_version: reference.reference_version.clone(),
        disposition: d,
        mod_rank: r.mod_rank,
        ranges,
        warnings,
    })
}

pub fn range_hints(
    r: &RangeHintRequest,
    catalog: &MarketCatalog,
    reference: &ReferenceData,
) -> Result<RangeHintsResult, String> {
    if r.mod_rank > 8 {
        return Err("强化等级必须是 0 到 8".into());
    }
    if !matches!(r.rule_mode.as_str(), "current" | "legacy") {
        return Err("规则模式必须是 current 或 legacy".into());
    }
    if r.attributes.len() > 4 {
        return Err("词条最多 4 项".into());
    }
    if r.positive_count > 3 || r.negative_count > 1 {
        return Err("词条槽位数量无效".into());
    }
    let mut keys = HashSet::new();
    if r.attributes
        .iter()
        .any(|a| a.key.is_empty() || !keys.insert(&a.key))
    {
        return Err("词条 key 必须非空且唯一".into());
    }
    let Some(weapon) = catalog.weapons.iter().find(|w| w.slug == r.weapon_slug) else {
        return Err("未找到紫卡武器".into());
    };
    let invalid_layout = r.positive_count < 2;
    if r.attributes.iter().filter(|a| a.positive).count() > r.positive_count as usize
        || r.attributes.iter().filter(|a| !a.positive).count() > r.negative_count as usize
    {
        return Err("词条方向数量超过槽位".into());
    }
    let (lp, ln) =
        layout_factors(r.positive_count as usize, r.negative_count as usize).unwrap_or((0.0, None));
    let d = if r.rule_mode == "current" {
        1.2
    } else {
        r.disposition
            .or(weapon.disposition)
            .ok_or_else(|| "历史倾向不可用".to_string())?
    };
    if !d.is_finite() || (r.rule_mode == "legacy" && !(0.5..=1.55).contains(&d)) {
        return Err("历史倾向必须在 0.5 到 1.55 之间".into());
    }
    let class = if weapon.group == "archgun" {
        "archgun"
    } else {
        weapon.riven_type.as_str()
    };
    let type_key = match class {
        "archgun" => "LotusArchgunRandomModRare",
        "zaw" => "LotusModularMeleeRandomModRare",
        "kitgun" => "LotusModularPistolRandomModRare",
        "melee" => "PlayerMeleeWeaponRandomModRare",
        "pistol" => "LotusPistolRandomModRare",
        "shotgun" => "LotusShotgunRandomModRare",
        "rifle" => "LotusRifleRandomModRare",
        _ => "",
    };
    let coeffs = reference.types.get(type_key);
    let mut ranges = Vec::new();
    for a in &r.attributes {
        let Some(ca) = catalog.attributes.iter().find(|x| x.slug == a.slug) else {
            ranges.push(RangeHint {
                key: a.key.clone(),
                slug: a.slug.clone(),
                positive: a.positive,
                unit: String::new(),
                range_min: None,
                range_max: None,
                status: "unsupported".into(),
                message: Some("未找到词条元数据".into()),
            });
            continue;
        };
        let Some(ra) = reference.attributes.get(&a.slug) else {
            ranges.push(RangeHint {
                key: a.key.clone(),
                slug: a.slug.clone(),
                positive: a.positive,
                unit: ca.unit.clone(),
                range_min: None,
                range_max: None,
                status: "unsupported".into(),
                message: Some("未核实该词条的属性元数据".into()),
            });
            continue;
        };
        let tag0 = if ca.game_ref.is_empty() {
            ra.game_ref.as_str()
        } else {
            ca.game_ref.as_str()
        };
        let tag = if class == "melee" || class == "zaw" {
            reference
                .aliases
                .get(tag0)
                .map(String::as_str)
                .unwrap_or(tag0)
        } else {
            tag0
        };
        if let Some((unit, lo, hi, _best)) = spliced_range(
            &class,
            tag,
            r.positive_count,
            r.negative_count,
            r.mod_rank,
            if r.rule_mode == "current" {
                1.2
            } else {
                r.disposition.unwrap_or(1.2)
            },
        ) {
            let mut out = RangeHint {
                key: a.key.clone(),
                slug: a.slug.clone(),
                positive: a.positive,
                unit,
                range_min: None,
                range_max: None,
                status: "unsupported".into(),
                message: None,
            };
            if !a.positive {
                out.message = Some("熔接词条只能作为正面词条".into());
            } else {
                out.range_min = Some(lo);
                out.range_max = Some(hi);
                out.status = "within".into();
            }
            ranges.push(out);
            continue;
        }
        let effective_class = if weapon.group == "archgun" {
            "archgun"
        } else {
            weapon.riven_type.as_str()
        };
        if (a.positive && ra.negative_only)
            || (!a.positive && ra.positive_only)
            || (!ca.exclusive_to.is_empty()
                && !ca
                    .exclusive_to
                    .iter()
                    .any(|x| x == effective_class || x == "default"))
        {
            let out = RangeHint {
                key: a.key.clone(),
                slug: a.slug.clone(),
                positive: a.positive,
                unit: ra.unit.clone(),
                range_min: None,
                range_max: None,
                status: "unsupported".into(),
                message: Some("词条方向或武器类型不匹配".into()),
            };
            ranges.push(out);
            continue;
        }
        if tag == "WeaponMeleeComboPointsOnHitMod" {
            ranges.push(RangeHint {
                key: a.key.clone(),
                slug: a.slug.clone(),
                positive: a.positive,
                unit: ra.unit.clone(),
                range_min: None,
                range_max: None,
                status: "unsupported".into(),
                message: Some("未核实该词条的反向显示规则".into()),
            });
            continue;
        }
        let coeff = coeffs.and_then(|rows| rows.iter().find(|x| x.tag == tag));
        let mut out = RangeHint {
            key: a.key.clone(),
            slug: a.slug.clone(),
            positive: a.positive,
            unit: ra.unit.clone(),
            range_min: None,
            range_max: None,
            status: "unsupported".into(),
            message: None,
        };
        if (!a.positive && r.negative_count == 0) || (a.positive && r.positive_count == 0) {
            out.message = Some("词条方向与槽位不匹配".into());
            ranges.push(out);
            continue;
        }
        if invalid_layout {
            out.status = "invalid_layout".into();
            out.message = Some("至少添加2个正面词条".into());
            ranges.push(out);
            continue;
        }
        let Some(c) = coeff else {
            out.message = Some("未核实该词条的基数或映射".into());
            ranges.push(out);
            continue;
        };
        if !matches!(ra.unit.as_str(), "" | "percent" | "multiply" | "seconds") {
            out.message = Some("未核实该词条单位".into());
            ranges.push(out);
            continue;
        }
        let factor = if a.positive { lp } else { ln.unwrap_or(0.0) };
        let mut lo = theoretical_value(c.base, r.mod_rank, d, factor, 0.9);
        let mut hi = theoretical_value(c.base, r.mod_rank, d, factor, 1.1);
        if lo > hi {
            std::mem::swap(&mut lo, &mut hi);
        }
        let transform = |v: f64| {
            if ra.unit == "multiply" {
                1.0 + v
            } else if ra.unit == "percent" {
                v * 100.0
            } else {
                v
            }
        };
        out.range_min = Some(transform(lo));
        out.range_max = Some(transform(hi));
        out.status = "available".into();
        ranges.push(out);
    }
    Ok(RangeHintsResult { ranges })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::{LocalizedName, RivenAttribute, RivenWeapon};

    fn catalog() -> MarketCatalog {
        MarketCatalog {
            weapons: vec![RivenWeapon {
                id: "w".into(),
                slug: "kulstar".into(),
                i18n: [(
                    "en".into(),
                    LocalizedName {
                        name: "Kulstar".into(),
                    },
                )]
                .into_iter()
                .collect(),
                riven_type: "pistol".into(),
                group: "secondary".into(),
                disposition: Some(0.5),
                game_ref: String::new(),
            }],
            attributes: vec![
                RivenAttribute {
                    id: "a".into(),
                    slug: "critical_chance".into(),
                    name: "Critical Chance".into(),
                    english_name: "Critical Chance".into(),
                    game_ref: "WeaponCritChanceMod".into(),
                    unit: "percent".into(),
                    positive_only: false,
                    negative_only: false,
                    positive_is_negative: false,
                    exclusive_to: vec![],
                },
                RivenAttribute {
                    id: "b".into(),
                    slug: "critical_damage".into(),
                    name: "Critical Damage".into(),
                    english_name: "Critical Damage".into(),
                    game_ref: "WeaponCritDamageMod".into(),
                    unit: "percent".into(),
                    positive_only: false,
                    negative_only: false,
                    positive_is_negative: false,
                    exclusive_to: vec![],
                },
                RivenAttribute {
                    id: "c".into(),
                    slug: "fire_rate".into(),
                    name: "Fire Rate".into(),
                    english_name: "Fire Rate".into(),
                    game_ref: "WeaponFireRateMod".into(),
                    unit: "percent".into(),
                    positive_only: false,
                    negative_only: false,
                    positive_is_negative: false,
                    exclusive_to: vec![],
                },
                RivenAttribute {
                    id: "d".into(),
                    slug: "recoil".into(),
                    name: "Recoil".into(),
                    english_name: "Recoil".into(),
                    game_ref: "WeaponRecoilReductionMod".into(),
                    unit: "percent".into(),
                    positive_only: false,
                    negative_only: false,
                    positive_is_negative: true,
                    exclusive_to: vec![],
                },
                RivenAttribute {
                    id: "e".into(),
                    slug: "damage_vs_corpus".into(),
                    name: "Damage vs Corpus".into(),
                    english_name: "Damage vs Corpus".into(),
                    game_ref: "WeaponFactionDamageCorpus".into(),
                    unit: "multiply".into(),
                    positive_only: false,
                    negative_only: false,
                    positive_is_negative: false,
                    exclusive_to: vec![],
                },
            ],
        }
    }

    #[test]
    fn embedded_seed_has_all_types_and_coefficients() {
        let r = embedded_reference().unwrap();
        assert_eq!(r.types.len(), 7);
        assert_eq!(r.types.len(), 7);
        assert!(r.types.values().all(|rows| !rows.is_empty()));
        assert_eq!(r.rule_version, "44.1.3");
    }

    #[test]
    fn range_hints_uses_slot_layout_without_values() {
        let c = catalog();
        let r = embedded_reference().unwrap();
        let h = range_hints(
            &RangeHintRequest {
                weapon_slug: "kulstar".into(),
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
            },
            &c,
            &r,
        )
        .unwrap();
        let full = analyze(
            &AnalysisRequest {
                weapon_slug: "kulstar".into(),
                mod_rank: 8,
                rule_mode: "current".into(),
                disposition: None,
                sample_size: 1,
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
                    AnalysisAttributeInput {
                        slug: "fire_rate".into(),
                        value: 50.,
                        positive: true,
                    },
                    AnalysisAttributeInput {
                        slug: "recoil".into(),
                        value: 50.,
                        positive: false,
                    },
                ],
            },
            &c,
            &r,
        )
        .unwrap();
        assert_eq!(h.ranges[0].status, "available");
        assert!((h.ranges[0].range_min.unwrap() - full.ranges[1].range_min.unwrap()).abs() < 1e-9);
    }
    #[test]
    fn range_hints_invalid_layout_and_unknown_are_per_item() {
        let c = catalog();
        let r = embedded_reference().unwrap();
        let h = range_hints(
            &RangeHintRequest {
                weapon_slug: "kulstar".into(),
                mod_rank: 8,
                rule_mode: "current".into(),
                disposition: None,
                positive_count: 1,
                negative_count: 0,
                attributes: vec![RangeHintInput {
                    key: "x".into(),
                    slug: "critical_chance".into(),
                    positive: true,
                }],
            },
            &c,
            &r,
        )
        .unwrap();
        assert_eq!(h.ranges[0].status, "invalid_layout");
    }
    #[test]
    fn writes_range_hint_fixture_when_env_is_set() {
        let Some(path) = std::env::var_os("WM_RANGE_HINT_FIXTURE_PATH") else {
            return;
        };
        let c =
            tauri::async_runtime::block_on(crate::market::MarketService::new().catalog()).unwrap();
        let r = embedded_reference().unwrap();
        let mut all = Vec::new();
        for (p, n) in [(2, 0), (2, 1), (3, 0), (3, 1)] {
            let mut ranges = Vec::new();
            for slug in ["critical_chance", "critical_damage", "multishot"] {
                let h = range_hints(
                    &RangeHintRequest {
                        weapon_slug: "torid".into(),
                        mod_rank: 8,
                        rule_mode: "current".into(),
                        disposition: None,
                        positive_count: p,
                        negative_count: n,
                        attributes: vec![RangeHintInput {
                            key: slug.into(),
                            slug: slug.into(),
                            positive: true,
                        }],
                    },
                    &c,
                    &r,
                )
                .unwrap();
                ranges.extend(h.ranges);
            }
            if n == 1 {
                let h = range_hints(
                    &RangeHintRequest {
                        weapon_slug: "torid".into(),
                        mod_rank: 8,
                        rule_mode: "current".into(),
                        disposition: None,
                        positive_count: p,
                        negative_count: n,
                        attributes: vec![RangeHintInput {
                            key: "fire_rate_/_attack_speed".into(),
                            slug: "fire_rate_/_attack_speed".into(),
                            positive: false,
                        }],
                    },
                    &c,
                    &r,
                )
                .unwrap();
                ranges.extend(h.ranges);
            }
            all.push(serde_json::json!({"positive_count":p,"negative_count":n,"ranges":ranges}));
        }
        std::fs::write(path, serde_json::to_vec_pretty(&all).unwrap()).unwrap();
    }

    #[test]
    fn layout_rank_and_disposition_math() {
        assert_eq!(layout_factors(2, 0), Some((0.99, None)));
        assert_eq!(layout_factors(2, 1), Some((1.2375, Some(-0.495))));
        assert_eq!(layout_factors(3, 0), Some((0.75, None)));
        assert_eq!(layout_factors(3, 1), Some((0.9375, Some(-0.75))));
        assert!((theoretical_value(1.0, 0, 1.2, 0.99, 1.0) - 11.88).abs() < 1e-9);
        assert!((theoretical_value(1.0, 8, 1.2, 0.99, 1.0) - 106.92).abs() < 1e-9);
        assert!((theoretical_value(1.0, 0, 0.5, 0.99, 1.0) - 4.95).abs() < 1e-9);
    }

    #[test]
    fn invalid_rank_and_sample_are_rejected() {
        let req = AnalysisRequest {
            weapon_slug: "missing".into(),
            mod_rank: 9,
            rule_mode: "current".into(),
            disposition: None,
            attributes: vec![],
            sample_size: 0,
        };
        assert!(validate_request(&req, &catalog()).is_err());
    }

    #[test]
    fn analyze_uses_display_input_and_current_d() {
        let reference = embedded_reference().unwrap();
        let req = AnalysisRequest {
            weapon_slug: "kulstar".into(),
            mod_rank: 0,
            rule_mode: "current".into(),
            disposition: Some(0.5),
            sample_size: 1,
            attributes: vec![
                AnalysisAttributeInput {
                    slug: "critical_chance".into(),
                    value: 20.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "critical_damage".into(),
                    value: 20.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "fire_rate".into(),
                    value: 20.0,
                    positive: true,
                },
            ],
        };
        let result = analyze(&req, &catalog(), &reference).unwrap();
        assert_eq!(result.disposition, 1.2);
        assert_eq!(result.ranges[0].value, 20.0);
    }

    #[test]
    fn analyze_outside_range_has_no_position() {
        let reference = embedded_reference().unwrap();
        let req = AnalysisRequest {
            weapon_slug: "kulstar".into(),
            mod_rank: 0,
            rule_mode: "current".into(),
            disposition: None,
            sample_size: 1,
            attributes: vec![
                AnalysisAttributeInput {
                    slug: "critical_chance".into(),
                    value: 200.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "critical_damage".into(),
                    value: 20.0,
                    positive: true,
                },
            ],
        };
        let result = analyze(&req, &catalog(), &reference).unwrap();
        assert_eq!(result.ranges[0].status, "out_of_range");
        assert!(result.ranges[0].position_percent.is_none());
    }

    #[test]
    fn analyze_negative_recoil_legacy_rank_and_multiplier() {
        let reference = embedded_reference().unwrap();
        let mut c = catalog();
        let req = AnalysisRequest {
            weapon_slug: "kulstar".into(),
            mod_rank: 8,
            rule_mode: "legacy".into(),
            disposition: None,
            sample_size: 1,
            attributes: vec![
                AnalysisAttributeInput {
                    slug: "recoil".into(),
                    value: -20.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "critical_chance".into(),
                    value: 20.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "damage_vs_corpus".into(),
                    value: 1.5,
                    positive: true,
                },
            ],
        };
        let result = analyze(&req, &c, &reference).unwrap();
        assert_eq!(result.disposition, 0.5);
        assert_eq!(result.ranges[0].value, -20.0);
        assert_eq!(result.ranges[2].value, 1.5);
        assert!(result.ranges.iter().all(|x| x.range_min.is_some()));
        c.weapons[0].group = "archgun".into();
        c.weapons[0].riven_type = "rifle".into();
        let arch = analyze(
            &AnalysisRequest {
                attributes: vec![
                    AnalysisAttributeInput {
                        slug: "critical_chance".into(),
                        value: 20.0,
                        positive: true,
                    },
                    AnalysisAttributeInput {
                        slug: "critical_damage".into(),
                        value: 20.0,
                        positive: true,
                    },
                ],
                rule_mode: "current".into(),
                mod_rank: 0,
                disposition: None,
                sample_size: 1,
                weapon_slug: "kulstar".into(),
            },
            &c,
            &reference,
        )
        .unwrap();
        assert!(arch.ranges[0].range_max.unwrap() < result.ranges[1].range_max.unwrap());
    }

    #[test]
    fn analyze_real_rifle_negative_direction_and_multiply_bounds() {
        let reference = embedded_reference().unwrap();
        let mut c = catalog();
        c.weapons[0].riven_type = "rifle".into();
        let req = |attrs| AnalysisRequest {
            weapon_slug: "kulstar".into(),
            mod_rank: 8,
            rule_mode: "current".into(),
            disposition: None,
            sample_size: 1,
            attributes: attrs,
        };
        let negative_recoil = analyze(
            &req(vec![
                AnalysisAttributeInput {
                    slug: "critical_chance".into(),
                    value: 180.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "critical_damage".into(),
                    value: 145.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "recoil".into(),
                    value: 48.1,
                    positive: false,
                },
            ]),
            &c,
            &reference,
        )
        .unwrap();
        assert!((negative_recoil.ranges[2].range_min.unwrap() - 48.114).abs() < 0.01);
        assert!((negative_recoil.ranges[2].range_max.unwrap() - 58.806).abs() < 0.01);
        assert!((negative_recoil.ranges[2].best_value.unwrap() - 48.114).abs() < 0.01);
        assert!(negative_recoil.ranges[2].position_percent.unwrap() > 99.0);
        let positive_recoil = analyze(
            &AnalysisRequest {
                weapon_slug: "kulstar".into(),
                mod_rank: 8,
                rule_mode: "current".into(),
                disposition: None,
                sample_size: 1,
                attributes: vec![
                    AnalysisAttributeInput {
                        slug: "critical_chance".into(),
                        value: 180.0,
                        positive: true,
                    },
                    AnalysisAttributeInput {
                        slug: "recoil".into(),
                        value: -100.0,
                        positive: true,
                    },
                ],
            },
            &c,
            &reference,
        )
        .unwrap();
        assert!((positive_recoil.ranges[1].range_min.unwrap() + 117.612).abs() < 0.01);
        assert!((positive_recoil.ranges[1].range_max.unwrap() + 96.228).abs() < 0.01);
        assert!((positive_recoil.ranges[1].best_value.unwrap() + 117.612).abs() < 0.01);
        let multiply_negative = analyze(
            &req(vec![
                AnalysisAttributeInput {
                    slug: "critical_chance".into(),
                    value: 180.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "critical_damage".into(),
                    value: 145.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "damage_vs_corpus".into(),
                    value: 1.5,
                    positive: false,
                },
            ]),
            &c,
            &reference,
        )
        .unwrap();
        assert!(
            multiply_negative.ranges[2].range_min.unwrap() < 1.0
                && multiply_negative.ranges[2].range_max.unwrap() < 1.0
        );
        assert_eq!(multiply_negative.ranges[2].value, 1.5);
        assert_eq!(
            multiply_negative.ranges[2].best_value.unwrap(),
            multiply_negative.ranges[2].range_max.unwrap()
        );
        let out = analyze(
            &req(vec![
                AnalysisAttributeInput {
                    slug: "critical_chance".into(),
                    value: 180.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "critical_damage".into(),
                    value: 145.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "damage_vs_corpus".into(),
                    value: 2.0,
                    positive: false,
                },
            ]),
            &c,
            &reference,
        )
        .unwrap();
        assert_eq!(out.ranges[2].status, "out_of_range");
        assert!(out.ranges[2].position_percent.is_none());
    }

    #[test]
    fn unknown_class_and_unverified_tag_are_unsupported() {
        let reference = embedded_reference().unwrap();
        let mut c = catalog();
        c.weapons[0].riven_type = "unknown".into();
        let req = AnalysisRequest {
            weapon_slug: "kulstar".into(),
            mod_rank: 0,
            rule_mode: "current".into(),
            disposition: None,
            sample_size: 1,
            attributes: vec![
                AnalysisAttributeInput {
                    slug: "critical_chance".into(),
                    value: 20.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "critical_damage".into(),
                    value: 20.0,
                    positive: true,
                },
            ],
        };
        let result = analyze(&req, &c, &reference).unwrap();
        assert!(result.ranges.iter().all(|x| x.status == "unsupported"));
    }

    #[test]
    fn known_tag_with_new_slug_without_metadata_is_unsupported() {
        let reference = embedded_reference().unwrap();
        let mut c = catalog();
        c.attributes.push(RivenAttribute {
            id: "new".into(),
            slug: "new_critical".into(),
            name: "New Critical".into(),
            english_name: "New Critical".into(),
            game_ref: "WeaponCritChanceMod".into(),
            unit: "percent".into(),
            positive_only: false,
            negative_only: false,
            positive_is_negative: false,
            exclusive_to: vec![],
        });
        let req = AnalysisRequest {
            weapon_slug: "kulstar".into(),
            mod_rank: 0,
            rule_mode: "current".into(),
            disposition: None,
            sample_size: 1,
            attributes: vec![
                AnalysisAttributeInput {
                    slug: "new_critical".into(),
                    value: 20.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "critical_damage".into(),
                    value: 20.0,
                    positive: true,
                },
            ],
        };
        let result = analyze(&req, &c, &reference).unwrap();
        assert_eq!(result.ranges[0].status, "unsupported");
        assert!(result.ranges[0].range_min.is_none());
    }

    #[test]
    fn writes_requested_ui_fixture_only_when_env_is_set() {
        let Some(path) = std::env::var_os("WM_ANALYSIS_FIXTURE_PATH") else {
            return;
        };
        let reference = embedded_reference().unwrap();
        let mut c = catalog();
        c.weapons[0].slug = "torid".into();
        c.weapons[0].riven_type = "rifle".into();
        c.weapons[0].group = "primary".into();
        c.weapons[0].i18n.insert(
            "en".into(),
            LocalizedName {
                name: "Torid".into(),
            },
        );
        let req = AnalysisRequest {
            weapon_slug: "torid".into(),
            mod_rank: 8,
            rule_mode: "current".into(),
            disposition: None,
            sample_size: 20,
            attributes: vec![
                AnalysisAttributeInput {
                    slug: "critical_chance".into(),
                    value: 180.0,
                    positive: true,
                },
                AnalysisAttributeInput {
                    slug: "critical_damage".into(),
                    value: 145.0,
                    positive: true,
                },
            ],
        };
        let result = analyze(&req, &c, &reference).unwrap();
        std::fs::write(path, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    }
}
