//! Complete planner adapter proposal. Only validated catalog slugs cross the public boundary.
use crate::{
    market::{MarketCatalog, RivenWeapon},
    reroll_pool::{pool_for, Pool, PoolStat, Splice, WeaponPoolInput},
    reroll_solver::{
        self as core, Action, Card, CardTrait, Group, Layout, Lock, Objective, Route,
        SharedWorkAccount, SolveError, Solver, Target, WorkAccount,
    },
    riven_analysis::{self, ReferenceData},
    riven_roll_data::{self as data, RivenClass, StatDef, Weapon},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};
pub const SAMPLES: usize = 3000;
const VERSION: &str = "riven-roll-rust-core/986f37c";
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RerollPlanRequest {
    pub weapon_slug: String,
    pub mod_rank: u8,
    pub rule_mode: String,
    pub disposition: Option<f64>,
    pub current_attributes: Vec<InputAttribute>,
    pub rerolls: u32,
    pub kuva_budget: Option<f64>,
    pub objective: Objective,
    pub target: TargetInput,
    pub new_attributes: Option<Vec<InputAttribute>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputAttribute {
    pub slug: String,
    pub value: f64,
    pub positive: bool,
    #[serde(default)]
    pub spliced: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetInput {
    pub groups: Vec<TargetGroupInput>,
    pub sets: Option<Vec<Vec<usize>>>,
    pub layouts: Vec<Layout>,
    pub allowed_negatives: Option<Vec<String>>,
    pub at_least: Option<core::AtLeast>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetGroupInput {
    pub slugs: Vec<String>,
    pub min_w: f64,
    pub min_values: Option<HashMap<String, f64>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quantiles {
    pub mean: f64,
    pub p10: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteStats {
    pub route: Route,
    pub next_cycle_cost: f64,
    pub kuva: Quantiles,
    pub cycles: Quantiles,
    pub splicers: f64,
    pub within_budget: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RerollPlanResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub done: bool,
    pub reachable: bool,
    pub routes: Vec<RouteStats>,
    pub states: usize,
    pub ms: f64,
    pub compare_values: Vec<Option<f64>>,
    pub compare_neg: Vec<Option<String>>,
    pub current_value: Option<f64>,
    pub objective: Objective,
    pub warnings: Vec<String>,
    pub version: String,
    pub samples: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RerollOptionsResponse {
    pub weapon_slug: String,
    pub game_ref: Option<String>,
    pub positive: Vec<String>,
    pub negative: Vec<String>,
    pub spliced: Vec<String>,
    pub version: String,
}
#[derive(Clone)]
struct Mapping {
    class: RivenClass,
    slug_tag: HashMap<String, String>,
    tag_slug: HashMap<String, String>,
}
static REFERENCE: OnceLock<Result<ReferenceData, String>> = OnceLock::new();
fn reference() -> Result<&'static ReferenceData, String> {
    REFERENCE
        .get_or_init(riven_analysis::embedded_reference)
        .as_ref()
        .map_err(Clone::clone)
}
fn class_key(c: RivenClass) -> &'static str {
    match c {
        RivenClass::Rifle => "rifle",
        RivenClass::Shotgun => "shotgun",
        RivenClass::Pistol => "pistol",
        RivenClass::Kitgun => "kitgun",
        RivenClass::Archgun => "archgun",
        RivenClass::Melee => "melee",
        RivenClass::Zaw => "zaw",
    }
}
fn exact_weapon<'a>(
    c: &'a MarketCatalog,
    slug: &str,
) -> Result<(&'a RivenWeapon, &'static Weapon), String> {
    let w = c
        .weapons
        .iter()
        .find(|w| w.slug == slug)
        .ok_or_else(|| format!("未找到紫卡武器：{slug}"))?;
    if w.game_ref.is_empty() {
        return Err(format!("武器 {slug} 缺少已核实的游戏引用"));
    }
    let g = data::game();
    let mut id = w.game_ref.as_str();
    let mut visited = HashSet::new();
    while let Some(next) = g.aliases.get(id) {
        if !visited.insert(id) {
            return Err("武器游戏引用别名存在循环".into());
        }
        id = next;
    }
    let gw = g
        .weapons
        .iter()
        .find(|x| x.id == id)
        .ok_or_else(|| format!("武器 {slug} 的游戏引用尚未支持：{}", w.game_ref))?;
    let market_class = if w.group == "archgun" {
        "archgun"
    } else {
        w.riven_type.as_str()
    };
    if market_class != class_key(gw.cls) {
        return Err(format!("武器 {slug} 的目录类型与游戏引用不一致"));
    }
    Ok((w, gw))
}
impl Mapping {
    fn new(c: &MarketCatalog, cls: RivenClass) -> Result<Self, String> {
        let reference = reference()?;
        let mut slug_tag = HashMap::new();
        let mut tag_slug = HashMap::new();
        for a in &c.attributes {
            let tag0 = if a.game_ref.is_empty() {
                let Some(r) = reference.attributes.get(&a.slug) else {
                    continue;
                };
                r.game_ref.as_str()
            } else {
                a.game_ref.as_str()
            };
            let tag = if matches!(cls, RivenClass::Melee | RivenClass::Zaw) {
                reference
                    .aliases
                    .get(tag0)
                    .map(String::as_str)
                    .unwrap_or(tag0)
            } else {
                tag0
            };
            if data::stat(cls, tag).is_none() {
                continue;
            }
            let key = class_key(cls);
            if !a.exclusive_to.is_empty()
                && !a.exclusive_to.iter().any(|x| x == key || x == "default")
            {
                continue;
            }
            if let Some(old) = tag_slug.insert(tag.to_string(), a.slug.clone()) {
                if old != a.slug {
                    return Err(format!("词条游戏引用重复：{old}、{}", a.slug));
                }
            }
            slug_tag.insert(a.slug.clone(), tag.to_string());
        }
        Ok(Self {
            class: cls,
            slug_tag,
            tag_slug,
        })
    }
    fn tag(&self, slug: &str) -> Result<&str, String> {
        self.slug_tag
            .get(slug)
            .map(String::as_str)
            .ok_or_else(|| format!("词条 {slug} 未能映射到该武器类型的游戏数据"))
    }
    fn slug(&self, tag: &str) -> Result<String, String> {
        self.tag_slug
            .get(tag)
            .cloned()
            .ok_or_else(|| format!("游戏词条 {tag} 缺少可用的目录标识"))
    }
    fn stat(&self, slug: &str) -> Result<&'static StatDef, String> {
        data::stat(self.class, self.tag(slug)?).ok_or_else(|| format!("未找到词条：{slug}"))
    }
    fn tags_to_slugs(&self, tags: &[String]) -> Result<Vec<String>, String> {
        tags.iter().map(|t| self.slug(t)).collect()
    }
    fn route(&self, mut r: Route) -> Result<Route, String> {
        match &mut r.action {
            Action::Cycle {
                lock: Lock::Pos(t), ..
            } => {
                t.tag = self.slug(&t.tag)?;
                t.stats = self.tags_to_slugs(&t.stats)?
            }
            Action::Splice { result, a, b } => {
                *result = self.slug(result)?;
                *a = self.slug(a)?;
                *b = self.slug(b)?
            }
            _ => {}
        }
        for o in r.outcomes.iter_mut().chain(r.worse.iter_mut()) {
            for n in &mut o.needs {
                n.stats = self.tags_to_slugs(&n.stats)?
            }
        }
        Ok(r)
    }
}
fn pool_input(gw: &Weapon) -> WeaponPoolInput {
    let cl = &data::game().classes[&gw.cls];
    WeaponPoolInput {
        class: class_key(gw.cls).into(),
        tags: gw.tags.clone(),
        physical: gw.physical.clone(),
        sentinel: gw.sentinel,
        grimoire: gw.grimoire,
        stats: cl
            .stats
            .iter()
            .map(|s| PoolStat {
                tag: s.tag.clone(),
                buff: s.buff,
                curse: s.curse,
                spliced: s.spliced,
                compat: s.compat.clone(),
                buff_restr: s.buff_restr.clone(),
                curse_restr: s.curse_restr.clone(),
            })
            .collect(),
        splices: cl
            .splices
            .iter()
            .map(|s| Splice {
                result: s.result.clone(),
                a: s.a.clone(),
                b: s.b.clone(),
            })
            .collect(),
    }
}
fn disposition(r: &RerollPlanRequest, w: &RivenWeapon) -> Result<f64, String> {
    if !matches!(r.rule_mode.as_str(), "current" | "legacy") {
        return Err("规则模式必须是 current 或 legacy".into());
    }
    if r.mod_rank > data::game().layout.max_rank {
        return Err("强化等级必须是 0 到 8".into());
    }
    if r.disposition.is_some_and(|d| !d.is_finite()) {
        return Err("倾向必须是有限数".into());
    }
    if r.rule_mode == "current" {
        return Ok(
            data::game().layout.disposition as f64, /* normalized below */
        );
    }
    let d = r.disposition.or(w.disposition).ok_or("历史倾向不可用")?;
    if !d.is_finite() || !(0.5..=1.55).contains(&d) {
        return Err("历史倾向必须在 0.5 到 1.55 之间".into());
    }
    Ok((d * 1e4).round() / 1e4)
}
fn current_disposition(d: f64) -> f64 {
    (d * 1e4).round() / 1e4
}
fn read_card(
    c: &MarketCatalog,
    m: &Mapping,
    attrs: &[InputAttribute],
    rank: u8,
    d: f64,
    label: &str,
) -> Result<Card, String> {
    let pos_count = attrs.iter().filter(|a| a.positive).count();
    let neg_count = attrs.len() - pos_count;
    if !(2..=3).contains(&pos_count) || neg_count > 1 || attrs.len() > 4 {
        return Err(format!(
            "{label}必须包含 2 到 3 个正面词条和 0 到 1 个负面词条（熔接占一个正面槽位）"
        ));
    }
    let layout = Layout {
        pos: pos_count as u8,
        neg: neg_count as u8,
    };
    let dl = data::Layout {
        pos: layout.pos,
        neg: layout.neg,
    };
    let mut seen = HashSet::new();
    let (mut pos, mut neg, mut spliced) = (vec![], None, None);
    for a in attrs {
        if !a.value.is_finite() {
            return Err(format!("{label}词条 {} 的数值必须是有限数", a.slug));
        }
        let tag = m.tag(&a.slug)?;
        if !seen.insert(tag) {
            return Err(format!("{label}词条不能重复：{}", a.slug));
        }
        let stat = m.stat(&a.slug)?;
        let ca = c
            .attributes
            .iter()
            .find(|x| x.slug == a.slug)
            .ok_or_else(|| format!("未找到词条 {}", a.slug))?;
        let ra = reference()?.attributes.get(&a.slug);
        if a.positive {
            if !stat.buff || ca.negative_only || ra.is_some_and(|x| x.negative_only) {
                return Err(format!("{} 不能作为正面词条", a.slug));
            }
        } else if !stat.curse
            || ca.positive_only
            || ra.is_some_and(|x| x.positive_only)
            || stat.spliced
        {
            return Err(format!("{} 不能作为负面词条", a.slug));
        }
        if a.spliced != stat.spliced {
            return Err(format!("{} 的熔接标记与游戏词条类型不一致", a.slug));
        }
        if a.spliced && (!a.positive || spliced.is_some()) {
            return Err("每张紫卡最多一个正面熔接词条".into());
        }
        let shown =
            data::signed_display_input(stat, !a.positive, Some(a.value)).ok_or("词条数值无效")?;
        let q = data::quality_from_display(stat, dl, !a.positive, shown, rank, d);
        if !q.valid || !q.w.is_finite() {
            return Err(format!(
                "{label}词条 {} 的显示数值不属于此布局、强化等级和倾向下的有效范围",
                a.slug
            ));
        }
        let w = if stat.spliced {
            q.w.max(q.hi - 1e-9).min(core::W_MAX)
        } else {
            q.w
        };
        let trait_ = CardTrait { tag: tag.into(), w };
        if a.spliced {
            spliced = Some(trait_)
        } else if a.positive {
            pos.push(trait_)
        } else {
            neg = Some(trait_)
        }
    }
    Ok(Card {
        layout,
        pos,
        neg,
        spliced,
    })
}
fn build_target(
    c: &MarketCatalog,
    m: &Mapping,
    t: &TargetInput,
    pool: &Pool,
    rank: u8,
    d: f64,
) -> Result<Target, String> {
    if t.groups.is_empty() {
        return Err("请至少配置一个目标词条组".into());
    }
    if t.layouts.is_empty() {
        return Err("请至少选择一个目标布局".into());
    }
    let mut layouts = HashSet::new();
    for l in &t.layouts {
        if !matches!(l.pos, 2 | 3) || l.neg > 1 || !layouts.insert(*l) {
            return Err("目标布局无效或重复".into());
        }
    }
    let sets = t
        .sets
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| vec![(0..t.groups.len()).collect()]);
    for set in &sets {
        let mut seen = HashSet::new();
        if set.is_empty()
            || set.len() > 3
            || set.iter().any(|&i| i >= t.groups.len() || !seen.insert(i))
        {
            return Err("目标组合必须包含 1 到 3 个不同且有效的词条组索引".into());
        }
    }
    if let Some(a) = t.at_least {
        if a.k == 0 || a.k > 3 || !a.w.is_finite() || !(core::W_MIN..=core::W_MAX).contains(&a.w) {
            return Err("至少达标规则的数量或 W 阈值无效".into());
        }
    }
    let mut groups = vec![];
    for g in &t.groups {
        if g.slugs.is_empty()
            || !g.min_w.is_finite()
            || !(core::W_MIN..=core::W_MAX).contains(&g.min_w)
        {
            return Err("目标词条组及 W 阈值无效（W 应为 0.9 到 1.1）".into());
        }
        let mut seen = HashSet::new();
        let mut tags = vec![];
        for slug in &g.slugs {
            let stat = m.stat(slug)?;
            let ca = c
                .attributes
                .iter()
                .find(|x| x.slug == *slug)
                .ok_or("目标词条不存在")?;
            if !stat.buff || ca.negative_only {
                return Err(format!("目标词条 {slug} 不能作为正面词条"));
            }
            if !seen.insert(stat.tag.clone()) {
                return Err(format!("目标词条组内不能重复：{slug}"));
            }
            tags.push(stat.tag.clone());
        }
        let mut min_w_by = None;
        let mut min_w = g.min_w;
        if let Some(values) = &g.min_values {
            if values
                .iter()
                .any(|(slug, value)| !g.slugs.contains(slug) || !value.is_finite())
            {
                return Err("目标显示值必须是组内词条的有限数".into());
            }
            let mut by = HashMap::new();
            let mut all = vec![];
            for (slug, tag) in g.slugs.iter().zip(&tags) {
                let s = m.stat(slug)?;
                let mut by_layout = HashMap::new();
                for l in &t.layouts {
                    let w = if let Some(&value) = values.get(slug) {
                        let shown = data::signed_display_input(s, false, Some(value))
                            .ok_or("目标显示值无效")?;
                        g.min_w.max(data::minimum_quality_for_display(
                            s,
                            data::Layout {
                                pos: l.pos,
                                neg: l.neg,
                            },
                            false,
                            shown,
                            rank,
                            d,
                        ))
                    } else {
                        g.min_w
                    };
                    all.push(w);
                    by_layout.insert(format!("{}{}", l.pos, l.neg), w);
                }
                by.insert(tag.clone(), by_layout);
            }
            min_w = all.into_iter().fold(f64::INFINITY, f64::min);
            min_w_by = Some(by);
        }
        groups.push(Group {
            tags,
            min_w,
            min_w_by,
        });
    }
    let allowed_neg = if let Some(slugs) = &t.allowed_negatives {
        let mut out = vec![];
        let mut seen = HashSet::new();
        for slug in slugs {
            let s = m.stat(slug)?;
            if !s.curse || s.spliced {
                return Err(format!("{slug} 不能作为可接受的负面词条"));
            }
            let tag = m.tag(slug)?.to_string();
            if !seen.insert(tag.clone()) {
                return Err("可接受负面词条不能重复".into());
            }
            out.push(tag)
        }
        out
    } else {
        pool.neg.clone()
    };
    Ok(Target {
        groups,
        sets,
        layouts: t.layouts.clone(),
        allowed_neg,
        at_least: t.at_least,
    })
}
struct Prepared {
    pool: Pool,
    target: Target,
    current: Card,
    compared: Option<Card>,
    mapping: Mapping,
    model_key: String,
    warnings: Vec<String>,
}
fn prepare(c: &MarketCatalog, r: &RerollPlanRequest) -> Result<Prepared, String> {
    let (w, gw) = exact_weapon(c, &r.weapon_slug)?;
    let d = current_disposition(disposition(r, w)?);
    if r.kuva_budget.is_some_and(|b| !b.is_finite() || b < 0.) {
        return Err("赤毒预算必须是非负有限数".into());
    }
    let mapping = Mapping::new(c, gw.cls)?;
    let current = read_card(c, &mapping, &r.current_attributes, r.mod_rank, d, "当前卡")?;
    let compared = r
        .new_attributes
        .as_ref()
        .map(|a| read_card(c, &mapping, a, r.mod_rank, d, "新卡"))
        .transpose()?;
    let mut seen_pos = vec![];
    let mut seen_neg = vec![];
    for card in std::iter::once(&current).chain(compared.iter()) {
        for p in card.pos.iter().chain(card.spliced.iter()) {
            seen_pos.push(p.tag.clone())
        }
        if let Some(n) = &card.neg {
            seen_neg.push(n.tag.clone())
        }
    }
    seen_pos.sort();
    seen_pos.dedup();
    seen_neg.sort();
    seen_neg.dedup();
    let pool = pool_for(&pool_input(gw), &seen_pos, &seen_neg);
    let target = build_target(c, &mapping, &r.target, &pool, r.mod_rank, d)?;
    let model_key = serde_json::to_string(&serde_json::json!([
        gw.id,
        r.target,
        r.objective,
        r.mod_rank,
        r.rule_mode,
        d,
        seen_pos,
        seen_neg,
        pool,
        target,
        mapping.slug_tag
    ]))
    .map_err(|e| format!("规划参数编码失败：{e}"))?;
    let warnings = if r.rule_mode == "legacy" {
        vec!["当前游戏快照的词条池与熔接规则配合指定历史倾向计算；不还原历史版本的词条池。".into()]
    } else {
        vec![]
    };
    Ok(Prepared {
        pool,
        target,
        current,
        compared,
        mapping,
        model_key,
        warnings,
    })
}
pub fn options_default(c: &MarketCatalog, slug: &str) -> Result<RerollOptionsResponse, String> {
    options(c, slug, &[])
}
pub fn options(
    c: &MarketCatalog,
    slug: &str,
    seen: &[InputAttribute],
) -> Result<RerollOptionsResponse, String> {
    let (w, gw) = exact_weapon(c, slug)?;
    let mapping = Mapping::new(c, gw.cls)?;
    let mut pos = vec![];
    let mut neg = vec![];
    for a in seen {
        let s = mapping.stat(&a.slug)?;
        if a.spliced != s.spliced
            || (a.positive && !s.buff)
            || (!a.positive && (!s.curse || s.spliced))
        {
            return Err(format!("词条 {} 的方向或熔接标记无效", a.slug));
        }
        if a.positive {
            pos.push(s.tag.clone())
        } else {
            neg.push(s.tag.clone())
        }
    }
    let pool = pool_for(&pool_input(gw), &pos, &neg);
    Ok(RerollOptionsResponse {
        weapon_slug: w.slug.clone(),
        game_ref: Some(gw.id.clone()),
        positive: mapping.tags_to_slugs(&pool.pos)?,
        negative: mapping.tags_to_slugs(&pool.neg)?,
        spliced: mapping.tags_to_slugs(&pool.spliced)?,
        version: VERSION.into(),
    })
}

struct SolvedGraph {
    key: String,
    solver: Solver,
    start: usize,
}
#[derive(Clone)]
struct Cached {
    key: String,
    response: RerollPlanResponse,
    samples: Vec<Vec<f64>>,
}
#[derive(Debug)]
enum Failure {
    Input(String),
    Core(SolveError),
}
impl From<SolveError> for Failure {
    fn from(e: SolveError) -> Self {
        Self::Core(e)
    }
}
impl From<String> for Failure {
    fn from(e: String) -> Self {
        Self::Input(e)
    }
}
type PlanResult<T> = Result<T, Failure>;
pub struct Planner {
    model_key: String,
    solved: VecDeque<SolvedGraph>,
    cached: Option<Cached>,
    account: SharedWorkAccount,
    seed: u64,
}
impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}
impl Planner {
    pub fn new() -> Self {
        Self::with_budget(core::WORK_BUDGET)
    }
    pub fn with_budget(limit: usize) -> Self {
        Self {
            model_key: String::new(),
            solved: VecDeque::new(),
            cached: None,
            account: Arc::new(Mutex::new(WorkAccount { used: 0, limit })),
            seed: 7,
        }
    }
    pub fn work_used(&self) -> usize {
        self.account.lock().expect("planner account poisoned").used
    }
    pub fn graph_count(&self) -> usize {
        self.solved.len()
    }
    fn forget(&mut self) {
        self.solved.clear();
        self.cached = None;
        self.account.lock().expect("planner account poisoned").used = 0;
    }
    fn evict(&mut self, index: usize) {
        if let Some(entry) = self.solved.remove(index) {
            let mut a = self.account.lock().expect("planner account poisoned");
            a.used = a.used.saturating_sub(entry.solver.work_used());
        }
    }
    pub fn plan(&mut self, c: &MarketCatalog, r: RerollPlanRequest) -> RerollPlanResponse {
        let st = Instant::now();
        let prepared = match prepare(c, &r) {
            Ok(p) => p,
            Err(e) => return failed(r.objective, Failure::Input(e), st),
        };
        let had_graphs = !self.solved.is_empty();
        let result = self.work(&prepared, &r);
        let result = match result {
            Err(Failure::Core(SolveError::TooComplex { .. })) => {
                self.forget();
                if had_graphs {
                    let retry = self.work(&prepared, &r);
                    if retry.is_err() {
                        self.forget()
                    }
                    retry
                } else {
                    result
                }
            }
            other => other,
        };
        match result {
            Ok(mut response) => {
                response.ms = st.elapsed().as_secs_f64() * 1000.;
                response
            }
            Err(e) => failed(r.objective, e, st),
        }
    }
    fn work(&mut self, p: &Prepared, r: &RerollPlanRequest) -> PlanResult<RerollPlanResponse> {
        let mut states = Solver::new(p.pool.clone(), p.target.clone(), r.objective);
        let current_id = states.state_of(&p.current)?;
        let current_key = states
            .state(current_id)
            .expect("interned current state")
            .key
            .clone();
        if p.compared.is_none() && states.is_goal(current_id) {
            return Ok(success(
                r.objective,
                true,
                true,
                states.state_count(),
                Some(0.),
                vec![],
                vec![],
                vec![],
                p.warnings.clone(),
                0,
            ));
        }
        let compared_key = if let Some(c) = &p.compared {
            let id = states.state_of(c)?;
            Some(
                states
                    .state(id)
                    .expect("interned compared state")
                    .key
                    .clone(),
            )
        } else {
            None
        };
        let priced = r.rerolls.min(core::DISCOUNTED_CYCLES as u32) as u64;
        if p.model_key != self.model_key {
            self.model_key = p.model_key.clone();
            self.forget();
        }
        let key = serde_json::to_string(&serde_json::json!([
            p.model_key,
            p.current,
            p.compared,
            priced,
            SAMPLES
        ]))
        .map_err(|e| Failure::Input(format!("规划缓存参数编码失败：{e}")))?;
        if self.cached.as_ref().is_none_or(|c| c.key != key) {
            let mut compared_value = None;
            if let (Some(card), Some(ck)) = (&p.compared, &compared_key) {
                if ck != &current_key {
                    let gi = self.graph_for(p, card, ck, r.objective)?;
                    let g = &self.solved[gi];
                    compared_value = Some(finite(g.solver.value[g.start]));
                    self.evict(gi);
                }
            }
            let gi = self.graph_for(p, &p.current, &current_key, r.objective)?;
            let graph = &mut self.solved[gi];
            let current = graph.solver.value[graph.start];
            let done = graph.solver.is_goal(graph.start);
            let reachable = current.is_finite();
            let compare_values = if p.compared.is_some() {
                vec![compared_value.unwrap_or_else(|| finite(current))]
            } else {
                vec![]
            };
            let compare_neg = if let Some(c) = &p.compared {
                vec![compare_negative(p, c, &mut states)?]
            } else {
                vec![]
            };
            let routes = if !done && reachable {
                graph.solver.routes(&p.current)?
            } else {
                vec![]
            };
            let means = graph.solver.expected_kuvas(&p.current, &routes, priced)?;
            let mut samples = vec![];
            let mut stats = vec![];
            for (route, mean) in routes.into_iter().zip(means) {
                let seed = &mut self.seed;
                let mut rng = || {
                    *seed = (*seed * 48271) % 2147483647;
                    *seed as f64 / 2147483647.
                };
                let sim = graph
                    .solver
                    .simulate(&p.current, &route, priced, SAMPLES, &mut rng)?;
                let mut kuva = sim.kuva;
                let mut cycles = sim.cycles;
                kuva.sort_by(f64::total_cmp);
                cycles.sort_by(f64::total_cmp);
                let mut kuva_stats = quantiles(&kuva)?;
                let mut cycle_stats = quantiles(&cycles)?;
                kuva_stats.mean = mean;
                if r.objective == Objective::Cycles {
                    cycle_stats.mean = route.expected;
                }
                let next_cycle_cost = match &route.action {
                    Action::Splice { .. } => 0.,
                    Action::Cycle { lock, .. } => core::cycle_cost(
                        r.rerolls as u64,
                        !matches!(lock, Lock::Simple(core::SimpleLock::None)),
                    ) as f64,
                };
                let mapped = p.mapping.route(route)?;
                stats.push(RouteStats {
                    route: mapped,
                    next_cycle_cost,
                    kuva: kuva_stats,
                    cycles: cycle_stats,
                    splicers: sim.splicers.iter().sum::<f64>() / SAMPLES as f64,
                    within_budget: None,
                });
                samples.push(kuva);
            }
            let count = if stats.is_empty() { 0 } else { SAMPLES as u32 };
            self.cached = Some(Cached {
                key,
                response: success(
                    r.objective,
                    done,
                    reachable,
                    graph.solver.state_count(),
                    finite(current),
                    compare_values,
                    compare_neg,
                    stats,
                    p.warnings.clone(),
                    count,
                ),
                samples,
            });
        }
        let cached = self.cached.as_ref().expect("computed response");
        let mut response = cached.response.clone();
        for (stats, samples) in response.routes.iter_mut().zip(&cached.samples) {
            stats.within_budget = r.kuva_budget.map(|b| within_budget(samples, b));
        }
        Ok(response)
    }
    fn graph_for(
        &mut self,
        p: &Prepared,
        card: &Card,
        key: &str,
        objective: Objective,
    ) -> PlanResult<usize> {
        if let Some(i) = self.solved.iter().position(|g| g.key == key) {
            let entry = self.solved.remove(i).expect("existing graph");
            self.solved.push_back(entry);
            return Ok(self.solved.len() - 1);
        }
        let limit = self.account.lock().expect("planner account poisoned").limit;
        while !self.solved.is_empty() && (self.solved.len() >= 2 || self.work_used() > limit / 2) {
            self.evict(0);
        }
        let mut solver = Solver::with_account(
            p.pool.clone(),
            p.target.clone(),
            objective,
            self.account.clone(),
        );
        let start = match solver.solve(card) {
            Ok(id) => id,
            Err(e) => {
                let mut account = self.account.lock().expect("planner account poisoned");
                account.used = account.used.saturating_sub(solver.work_used());
                return Err(e.into());
            }
        };
        self.solved.push_back(SolvedGraph {
            key: key.into(),
            solver,
            start,
        });
        Ok(self.solved.len() - 1)
    }
}
fn compare_negative(p: &Prepared, card: &Card, states: &mut Solver) -> PlanResult<Option<String>> {
    let Some(n) = &card.neg else { return Ok(None) };
    if p.target.groups.iter().any(|g| g.tags.contains(&n.tag)) {
        return Ok(None);
    }
    let id = states.state_of(card)?;
    if states.is_goal(id) {
        return Ok(None);
    }
    let mut target = p.target.clone();
    target.allowed_neg.push(n.tag.clone());
    let mut accepting = Solver::new(p.pool.clone(), target, states.objective);
    let id = accepting.state_of(card)?;
    if accepting.is_goal(id) {
        Ok(Some(p.mapping.slug(&n.tag)?))
    } else {
        Ok(None)
    }
}
fn finite(v: f64) -> Option<f64> {
    v.is_finite().then_some(v)
}
fn quantiles(sorted: &[f64]) -> PlanResult<Quantiles> {
    if sorted.is_empty() || sorted.iter().any(|v| !v.is_finite()) {
        return Err(Failure::Input("模拟结果为空或包含非有限数".into()));
    }
    let q = |p: f64| sorted[(p * (sorted.len() - 1) as f64).floor() as usize];
    Ok(Quantiles {
        mean: sorted.iter().sum::<f64>() / sorted.len() as f64,
        p10: q(0.1),
        p25: q(0.25),
        p50: q(0.5),
        p75: q(0.75),
        p90: q(0.9),
    })
}
fn within_budget(sorted: &[f64], budget: f64) -> f64 {
    sorted.partition_point(|&v| v <= budget) as f64 / sorted.len() as f64
}
#[allow(clippy::too_many_arguments)]
fn success(
    objective: Objective,
    done: bool,
    reachable: bool,
    states: usize,
    current_value: Option<f64>,
    compare_values: Vec<Option<f64>>,
    compare_neg: Vec<Option<String>>,
    routes: Vec<RouteStats>,
    warnings: Vec<String>,
    samples: u32,
) -> RerollPlanResponse {
    RerollPlanResponse {
        ok: true,
        error: None,
        done,
        reachable,
        routes,
        states,
        ms: 0.,
        compare_values,
        compare_neg,
        current_value,
        objective,
        warnings,
        version: VERSION.into(),
        samples,
    }
}
fn failed(objective: Objective, e: Failure, st: Instant) -> RerollPlanResponse {
    let(error,states,warning)=match e{Failure::Input(s)=>(s.clone(),0,s),Failure::Core(SolveError::TooComplex{work,states})=>("too-complex".into(),states,format!("目标计算量超过预算（已用 {work}，已生成 {states} 个状态），请减少目标备选或质量阈值。")),Failure::Core(e)=>{let s=format!("洗卡策略计算失败：{e}");(s.clone(),0,s)}};
    RerollPlanResponse {
        ok: false,
        error: Some(error),
        done: false,
        reachable: false,
        routes: vec![],
        states,
        ms: st.elapsed().as_secs_f64() * 1000.,
        compare_values: vec![],
        compare_neg: vec![],
        current_value: None,
        objective,
        warnings: vec![warning],
        version: VERSION.into(),
        samples: 0,
    }
}
pub fn plan(c: &MarketCatalog, r: RerollPlanRequest) -> RerollPlanResponse {
    Planner::new().plan(c, r)
}

#[cfg(test)]
#[path = "reroll_planner_tests.rs"]
mod tests;
