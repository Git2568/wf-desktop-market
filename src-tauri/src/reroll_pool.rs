//! Weapon-specific normal/spliced stat pool construction.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolStat {
    pub tag: String,
    pub buff: bool,
    pub curse: bool,
    pub spliced: bool,
    pub compat: Vec<String>,
    pub buff_restr: Vec<String>,
    pub curse_restr: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Splice {
    pub result: String,
    pub a: String,
    pub b: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeaponPoolInput {
    pub class: String,
    pub tags: Vec<String>,
    pub physical: Vec<String>,
    pub sentinel: bool,
    pub grimoire: bool,
    pub stats: Vec<PoolStat>,
    pub splices: Vec<Splice>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pool {
    pub class: String,
    pub pos: Vec<String>,
    pub neg: Vec<String>,
    pub kept_pos: Vec<String>,
    pub kept_neg: Vec<String>,
    pub spliced: Vec<String>,
    pub splices: Vec<Splice>,
}

fn passes(s: &PoolStat, w: &WeaponPoolInput, restrictions: &[String]) -> bool {
    if !s.compat.iter().all(|x| w.tags.iter().any(|t| t == x)) {
        return false;
    }
    restrictions.iter().all(|r| match r.as_str() {
        "NotSentinel" => !w.sentinel,
        "NotGrimoire" => !w.grimoire,
        "DamageTypes"
            if [
                "WeaponImpactDamageMod",
                "WeaponArmorPiercingDamageMod",
                "WeaponSlashDamageMod",
            ]
            .contains(&s.tag.as_str()) =>
        {
            w.physical.iter().any(|x| x == &s.tag)
        }
        _ => true,
    })
}
pub fn pool_for(w: &WeaponPoolInput, seen_pos: &[String], seen_neg: &[String]) -> Pool {
    let normal: Vec<_> = w.stats.iter().filter(|s| !s.spliced).collect();
    let draw_pos: HashSet<_> = normal
        .iter()
        .filter(|s| s.buff && passes(s, w, &s.buff_restr))
        .map(|s| s.tag.clone())
        .collect();
    let draw_neg: HashSet<_> = normal
        .iter()
        .filter(|s| s.curse && passes(s, w, &s.curse_restr))
        .map(|s| s.tag.clone())
        .collect();
    let kept_pos: Vec<_> = normal
        .iter()
        .filter(|s| s.buff && !draw_pos.contains(&s.tag) && seen_pos.contains(&s.tag))
        .map(|s| s.tag.clone())
        .collect();
    let kept_neg: Vec<_> = normal
        .iter()
        .filter(|s| s.curse && !draw_neg.contains(&s.tag) && seen_neg.contains(&s.tag))
        .map(|s| s.tag.clone())
        .collect();
    let pos: Vec<_> = normal
        .iter()
        .filter(|s| draw_pos.contains(&s.tag) || kept_pos.contains(&s.tag))
        .map(|s| s.tag.clone())
        .collect();
    let neg: Vec<_> = normal
        .iter()
        .filter(|s| draw_neg.contains(&s.tag) || kept_neg.contains(&s.tag))
        .map(|s| s.tag.clone())
        .collect();
    let can: HashSet<_> = pos.iter().chain(neg.iter()).collect();
    let splices: Vec<_> = w
        .splices
        .iter()
        .filter(|r| can.contains(&r.a) && can.contains(&r.b))
        .cloned()
        .collect();
    let mut spliced: Vec<_> = splices.iter().map(|r| r.result.clone()).collect();
    for s in &w.stats {
        if s.spliced && seen_pos.contains(&s.tag) && !spliced.contains(&s.tag) {
            spliced.push(s.tag.clone());
        }
    }
    Pool {
        class: w.class.clone(),
        pos,
        neg,
        kept_pos,
        kept_neg,
        spliced,
        splices,
    }
}
