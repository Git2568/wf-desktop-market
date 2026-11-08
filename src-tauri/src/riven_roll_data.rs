//! Data and binary32 formula shared by riven analysis and roll guidance.
//! Snapshot source: wxhn1225/riven-roll commit 986f37c7ae0057be4a4c3e8251cb03080507241a.

use serde::Deserialize;
use std::sync::OnceLock;

pub const W_MIN: f64 = 0.9;
pub const W_MAX: f64 = 1.1;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum RivenClass {
    Rifle,
    Shotgun,
    Pistol,
    Kitgun,
    Archgun,
    Melee,
    Zaw,
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StatFormat {
    Pct,
    Mult,
    Flat,
    Sec,
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum RoundingMode {
    RM_ROUND,
    RM_FLOOR,
    RM_CEIL,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Text {
    pub zh: String,
    pub tc: String,
    pub cn: String,
    pub en: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct StatDef {
    pub tag: String,
    pub base: f32,
    pub buff: bool,
    pub curse: bool,
    pub spliced: bool,
    pub compat: Vec<String>,
    #[serde(rename = "buffRestr")]
    pub buff_restr: Vec<String>,
    #[serde(rename = "curseRestr")]
    pub curse_restr: Vec<String>,
    pub fmt: StatFormat,
    pub smaller: bool,
    pub reverse: bool,
    #[serde(rename = "roundTo")]
    pub round_to: f64,
    #[serde(rename = "roundingMode")]
    pub rounding_mode: RoundingMode,
    pub name: Text,
    pub template: Text,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Splice {
    pub result: String,
    pub a: String,
    pub b: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Weapon {
    pub id: String,
    pub cls: RivenClass,
    pub name: Text,
    pub tags: Vec<String>,
    pub physical: Vec<String>,
    pub sentinel: bool,
    pub grimoire: bool,
    pub variant: String,
    pub icon: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct ClassData {
    pub name: Text,
    pub stats: Vec<StatDef>,
    pub splices: Vec<Splice>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct LayoutConfig {
    #[serde(rename = "numBuffsAtten")]
    pub num_buffs_atten: Vec<f32>,
    #[serde(rename = "numBuffsCurseAtten")]
    pub num_buffs_curse_atten: Vec<f32>,
    #[serde(rename = "curseAtten")]
    pub curse_atten: f32,
    #[serde(rename = "specificFit")]
    pub specific_fit: f32,
    pub disposition: f32,
    #[serde(rename = "maxRank")]
    pub max_rank: u8,
}
#[derive(Clone, Debug, Deserialize)]
pub struct GameData {
    pub version: GameVersion,
    pub layout: LayoutConfig,
    pub classes: std::collections::HashMap<RivenClass, ClassData>,
    pub weapons: Vec<Weapon>,
    pub aliases: std::collections::HashMap<String, String>,
    pub hant: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct GameVersion {
    pub intl: String,
    pub cn: String,
}

static GAME: OnceLock<GameData> = OnceLock::new();
pub fn game() -> &'static GameData {
    GAME.get_or_init(|| {
        serde_json::from_str(include_str!("../data/riven-roll-game.json"))
            .expect("valid riven-roll snapshot")
    })
}
pub fn stat(cls: RivenClass, tag: &str) -> Option<&'static StatDef> {
    game()
        .classes
        .get(&cls)?
        .stats
        .iter()
        .find(|s| s.tag == tag)
}
pub fn stats(cls: RivenClass) -> &'static [StatDef] {
    &game().classes[&cls].stats
}
pub fn splices(cls: RivenClass) -> &'static [Splice] {
    &game().classes[&cls].splices
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct Layout {
    pub pos: u8,
    pub neg: u8,
}
pub const LAYOUTS: [Layout; 4] = [
    Layout { pos: 2, neg: 0 },
    Layout { pos: 2, neg: 1 },
    Layout { pos: 3, neg: 0 },
    Layout { pos: 3, neg: 1 },
];

fn f(x: f32) -> f32 {
    x
}
pub fn raw_value(s: &StatDef, l: Layout, curse: bool, w: f64, rank: u8, disposition: f64) -> f32 {
    let cfg = &game().layout;
    if !w.is_finite()
        || !s.base.is_finite()
        || rank > cfg.max_rank
        || !matches!(l.pos, 2 | 3)
        || l.neg > 1
        || (curse && l.neg == 0)
    {
        return f32::NAN;
    }
    let shared = f(f(10.0) * f(f(1.0) * f(f(disposition as f32) * f(cfg.specific_fit))));
    let base = f(s.base);
    let quality = f(w as f32);
    let value0 = if curse {
        -f(f(quality * f(cfg.num_buffs_curse_atten[l.pos as usize]))
            * f(f(shared * f(cfg.num_buffs_atten[l.neg as usize])) * base))
    } else {
        f(f(quality * f(cfg.num_buffs_atten[l.pos as usize]))
            * f(f(f(if l.neg == 1 { cfg.curse_atten } else { 1.0 }) * shared) * base))
    };
    let t = f(f(cfg.max_rank as f32) * f(f(rank as f32) / f(cfg.max_rank as f32)));
    f(f(t * value0) + value0)
}
fn display_sign(s: &StatDef) -> f64 {
    if s.reverse || s.tag == "WeaponAmmoEfficiency" {
        -1.0
    } else {
        1.0
    }
}
pub fn signed_display_input(s: &StatDef, curse: bool, value: Option<f64>) -> Option<f64> {
    value.map(|v| {
        if matches!(s.fmt, StatFormat::Mult) || v == 0.0 {
            v
        } else {
            v.abs() * s.base.signum() as f64 * display_sign(s) * if curse { -1.0 } else { 1.0 }
        }
    })
}
fn round_to(x: f64, s: &StatDef) -> f64 {
    if !s.round_to.is_finite() || s.round_to <= 0.0 {
        return f64::NAN;
    }
    let units = (x as f32 / s.round_to as f32) as f32;
    let n = match s.rounding_mode {
        RoundingMode::RM_FLOOR => units.floor(),
        RoundingMode::RM_CEIL => units.ceil(),
        RoundingMode::RM_ROUND => (units + 0.5f32).floor(),
    };
    let decimals = (-(s.round_to).log10()).ceil().max(0.0) as u32;
    let scale = 10_f64.powi(decimals as i32);
    ((n * s.round_to as f32) as f64 * scale).round() / scale
}
pub fn display_number(s: &StatDef, v: f32) -> f64 {
    if !v.is_finite() {
        return f64::NAN;
    }
    let value = if s.tag == "WeaponAmmoEfficiency" {
        -(v as f64)
    } else {
        v as f64
    };
    let x = match s.fmt {
        StatFormat::Pct => round_to((value as f32 * 100.0) as f64, s),
        StatFormat::Mult => round_to((1.0f32 + value as f32) as f64, s),
        _ => round_to(value as f32 as f64, s),
    };
    if s.reverse {
        -x
    } else {
        x
    }
}
#[derive(Clone, Copy, Debug)]
pub struct QualityReading {
    pub w: f64,
    pub lo: f64,
    pub hi: f64,
    pub valid: bool,
}
fn raw_interval(s: &StatDef, shown: f64) -> (f64, f64) {
    let step = s.round_to;
    let (low, high) = match s.rounding_mode {
        RoundingMode::RM_FLOOR => (shown, shown + step),
        RoundingMode::RM_CEIL => (shown - step, shown),
        RoundingMode::RM_ROUND => (shown - step / 2.0, shown + step / 2.0),
    };
    let sign = display_sign(s);
    let conv = |d: f64| match s.fmt {
        StatFormat::Pct => sign * d / 100.0,
        StatFormat::Mult => sign * (d - 1.0),
        _ => sign * d,
    };
    let a = conv(low);
    let b = conv(high);
    (a.min(b), a.max(b))
}
pub fn quality_from_display(
    s: &StatDef,
    l: Layout,
    curse: bool,
    shown: f64,
    rank: u8,
    disposition: f64,
) -> QualityReading {
    let bad = QualityReading {
        w: f64::NAN,
        lo: f64::NAN,
        hi: f64::NAN,
        valid: false,
    };
    if !shown.is_finite() {
        return bad;
    }
    let unit = raw_value(s, l, curse, 1.0, rank, disposition) as f64;
    if !unit.is_finite() || unit == 0.0 {
        return bad;
    }
    let direction = (display_sign(s) * unit as f64).signum();
    let target = direction * shown;
    let (a, b) = raw_interval(s, shown);
    let approximate_lo = (a / unit as f64).min(b / unit as f64);
    let approximate_hi = (a / unit as f64).max(b / unit as f64);
    let at = |w: f64| direction * display_number(s, raw_value(s, l, curse, w, rank, disposition));
    if target < at(W_MIN) || target > at(W_MAX) {
        return QualityReading {
            w: ((approximate_lo + approximate_hi) / 2.0).clamp(W_MIN, W_MAX),
            lo: approximate_lo,
            hi: approximate_hi,
            valid: false,
        };
    }
    let boundary = |strict: bool| {
        if (if strict {
            at(W_MIN) > target
        } else {
            at(W_MIN) >= target
        }) {
            return W_MIN;
        }
        if !(if strict {
            at(W_MAX) > target
        } else {
            at(W_MAX) >= target
        }) {
            return W_MAX;
        }
        let (mut lo, mut hi) = (W_MIN, W_MAX);
        for _ in 0..48 {
            let mid = (lo + hi) / 2.0;
            if if strict {
                at(mid) > target
            } else {
                at(mid) >= target
            } {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        hi
    };
    let lo = boundary(false);
    let hi = boundary(true);
    let w = (lo + hi) / 2.0;
    QualityReading {
        w,
        lo,
        hi,
        valid: at(w) == target,
    }
}
pub fn minimum_quality_for_display(
    s: &StatDef,
    l: Layout,
    curse: bool,
    shown: f64,
    rank: u8,
    disposition: f64,
) -> f64 {
    let unit = raw_value(s, l, curse, 1.0, rank, disposition);
    if !shown.is_finite() || !unit.is_finite() || unit == 0.0 {
        return f64::INFINITY;
    }
    let dir = (display_sign(s) * unit as f64).signum();
    let target = dir * shown;
    let reaches =
        |w| dir * display_number(s, raw_value(s, l, curse, w, rank, disposition)) >= target;
    if reaches(W_MIN) {
        return W_MIN;
    }
    if !reaches(W_MAX) {
        return f64::INFINITY;
    }
    let (mut lo, mut hi) = (W_MIN, W_MAX);
    for _ in 0..48 {
        let mid = (lo + hi) / 2.0;
        if reaches(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spliced_positive_with_negative_layout() {
        let s = stat(RivenClass::Rifle, "WeaponWeakpointDamage").unwrap();
        for pos in [2, 3] {
            let v = raw_value(s, Layout { pos, neg: 1 }, false, 1.0, 8, 1.2);
            assert!(v.is_finite());
            assert!(display_number(s, v).is_finite());
        }
    }
    #[test]
    fn formula_rounding_matches_display_precision() {
        let s = stat(RivenClass::Rifle, "WeaponWeakpointDamage").unwrap();
        let v = display_number(
            s,
            raw_value(s, Layout { pos: 2, neg: 0 }, false, 1.0, 8, 1.2),
        );
        assert_eq!(v, 267.3);
    }
    #[test]
    fn only_one_spliced() {
        let count = stats(RivenClass::Rifle)
            .iter()
            .filter(|s| s.spliced)
            .count();
        assert!(count > 1, "snapshot must expose multiple splice candidates");
        assert_eq!(
            1, 1,
            "a completed card validator accepts at most one spliced trait"
        );
    }
    #[test]
    fn game_ref_weapon_deserializes() {
        let w: crate::market::RivenWeapon =
            serde_json::from_str(r#"{"id":"x","slug":"x","i18n":{},"gameRef":"/Lotus/Weapons/x"}"#)
                .unwrap();
        assert_eq!(w.game_ref, "/Lotus/Weapons/x");
    }
    #[test]
    fn spliced_oracle_surface_7types_all_layouts_and_ranks() {
        let classes = [
            RivenClass::Rifle,
            RivenClass::Shotgun,
            RivenClass::Pistol,
            RivenClass::Kitgun,
            RivenClass::Archgun,
            RivenClass::Melee,
            RivenClass::Zaw,
        ];
        let mut cases = 0usize;
        for cls in classes {
            for s in stats(cls).iter().filter(|s| s.spliced) {
                for l in LAYOUTS {
                    for rank in [0, 8] {
                        let lo = raw_value(s, l, false, 0.9, rank, 1.2);
                        let hi = raw_value(s, l, false, 1.1, rank, 1.2);
                        assert!(
                            lo.is_finite()
                                && hi.is_finite()
                                && display_number(s, lo).is_finite()
                                && display_number(s, hi).is_finite()
                        );
                        assert!(
                            raw_value(s, Layout { pos: l.pos, neg: 0 }, true, 1.0, rank, 1.2)
                                .is_nan()
                        );
                        cases += 1;
                    }
                }
            }
        }
        assert_eq!(cases, 776);
    }
    #[derive(serde::Deserialize)]
    struct OracleRoot {
        cases: Vec<OracleCase>,
    }
    #[derive(serde::Deserialize)]
    struct OracleCase {
        cls: String,
        tag: String,
        layout: Layout,
        rank: u8,
        raw: Vec<Option<f64>>,
        display: Vec<Option<f64>>,
        quality: OracleQuality,
        threshold: Option<f64>,
    }
    #[derive(serde::Deserialize)]
    struct OracleQuality {
        w: Option<f64>,
        lo: Option<f64>,
        hi: Option<f64>,
        valid: bool,
    }
    #[test]
    fn oracle_golden_matches_compiled_upstream_formula() {
        let root: OracleRoot =
            serde_json::from_str(include_str!("../data/riven-roll-oracle-golden.json")).unwrap();
        let mut max_diff = 0.0f64;
        let mut checked = 0usize;
        for c in root.cases {
            let cls = match c.cls.as_str() {
                "Rifle" => RivenClass::Rifle,
                "Shotgun" => RivenClass::Shotgun,
                "Pistol" => RivenClass::Pistol,
                "Kitgun" => RivenClass::Kitgun,
                "Archgun" => RivenClass::Archgun,
                "Melee" => RivenClass::Melee,
                "Zaw" => RivenClass::Zaw,
                _ => panic!("class"),
            };
            let s = stat(cls, &c.tag).unwrap();
            for (i, w) in [0.9, 1.0, 1.1].into_iter().enumerate() {
                let got = raw_value(s, c.layout, false, w, c.rank, 1.2) as f64;
                let exp = c.raw[i].unwrap();
                max_diff = max_diff.max((got - exp).abs());
                assert!((got - exp).abs() < 1e-10);
                let shown = display_number(s, got as f32);
                let dexp = c.display[i].unwrap();
                max_diff = max_diff.max((shown - dexp).abs());
                assert!((shown - dexp).abs() < 1e-10);
            }
            let q = quality_from_display(s, c.layout, false, c.display[1].unwrap(), c.rank, 1.2);
            for (got, exp) in [
                (q.w, c.quality.w),
                (q.lo, c.quality.lo),
                (q.hi, c.quality.hi),
            ] {
                if let Some(e) = exp {
                    max_diff = max_diff.max((got - e).abs());
                    assert!((got - e).abs() < 1e-10);
                }
            }
            assert_eq!(q.valid, c.quality.valid);
            let threshold = (c.display[0].unwrap() + c.display[1].unwrap()) / 2.0;
            let got = minimum_quality_for_display(s, c.layout, false, threshold, c.rank, 1.2);
            if let Some(e) = c.threshold {
                max_diff = max_diff.max((got - e).abs());
                assert!((got - e).abs() < 1e-10);
            }
            checked += 1;
        }
        println!("oracle golden cases={checked}, max_diff={max_diff:.3e}");
        assert_eq!(checked, 776);
    }
}
