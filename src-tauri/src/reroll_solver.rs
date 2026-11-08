//! PROPOSAL ONLY: faithful port of riven-roll solver.ts, commit 986f37c7ae0057be4a4c3e8251cb03080507241a.
//! No catalog/formula duplication. All finite solver probabilities use f64.
use crate::reroll_pool::Pool;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub const W_MIN: f64 = 0.9;
pub const W_MAX: f64 = 1.1;
pub const KUVA_UNLOCKED: u64 = 3500;
pub const DISCOUNTED_CYCLES: usize = 9;
pub const WORK_BUDGET: usize = 2_000_000;
const KUVA_STEPS: [u64; 9] = [900, 1000, 1200, 1400, 1700, 2000, 2350, 2750, 3150];
const JUNK: i32 = -1;
const OTHER_SPLICED: &str = "*:0";
const LAYOUTS: [Layout; 4] = [
    Layout { pos: 2, neg: 0 },
    Layout { pos: 2, neg: 1 },
    Layout { pos: 3, neg: 0 },
    Layout { pos: 3, neg: 1 },
];
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Layout {
    pub pos: u8,
    pub neg: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    pub tags: Vec<String>,
    pub min_w: f64,
    #[serde(default)]
    pub min_w_by: Option<HashMap<String, HashMap<String, f64>>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Target {
    pub groups: Vec<Group>,
    #[serde(default)]
    pub sets: Vec<Vec<usize>>,
    pub layouts: Vec<Layout>,
    #[serde(default)]
    pub allowed_neg: Vec<String>,
    pub at_least: Option<AtLeast>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AtLeast {
    pub k: usize,
    pub w: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardTrait {
    pub tag: String,
    pub w: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    pub layout: Layout,
    pub pos: Vec<CardTrait>,
    pub neg: Option<CardTrait>,
    pub spliced: Option<CardTrait>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Objective {
    Kuva,
    Cycles,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SimpleLock {
    None,
    Neg,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositiveLock {
    pub tag: String,
    pub stats: Vec<String>,
    pub w: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Lock {
    Simple(SimpleLock),
    Pos(PositiveLock),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Action {
    Cycle {
        lock: Lock,
        #[serde(default, skip_serializing_if = "is_false")]
        drop: bool,
    },
    Splice {
        result: String,
        a: String,
        b: String,
    },
}
fn is_false(v: &bool) -> bool {
    !*v
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Need {
    pub stats: Vec<String>,
    pub min_w: f64,
    #[serde(default, skip_serializing_if = "is_false")]
    pub neg: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Outcome {
    pub prob: f64,
    pub needs: Vec<Need>,
    pub neg_allowed: Option<bool>,
    pub layout: Layout,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Route {
    pub action: Action,
    pub step_chance: f64,
    pub step_cycles: f64,
    pub step_cycles_p90: f64,
    pub expected: f64,
    pub outcomes: Vec<Outcome>,
    pub worse: Vec<Outcome>,
    pub optimal: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Simulation {
    pub kuva: Vec<f64>,
    pub cycles: Vec<f64>,
    pub splicers: Vec<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum SolveError {
    TooComplex { work: usize, states: usize },
    Invalid(String),
}
impl std::fmt::Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooComplex { .. } => write!(f, "too-complex"),
            Self::Invalid(s) => write!(f, "{s}"),
        }
    }
}
impl std::error::Error for SolveError {}
type Res<T> = Result<T, SolveError>;
#[derive(Debug)]
pub struct WorkAccount {
    pub used: usize,
    pub limit: usize,
}
pub type SharedWorkAccount = Arc<Mutex<WorkAccount>>;
#[derive(Clone, Debug)]
pub struct Role {
    pub id: usize,
    pub stats: Vec<String>,
    pub can_pos: bool,
    pub can_neg: bool,
    pub allowed: bool,
    pub groups: Vec<usize>,
    pub recipes: Vec<usize>,
    pub th: Vec<f64>,
    pub nth: Vec<f64>,
}
#[derive(Clone, Debug)]
pub struct State {
    pub id: usize,
    pub key: String,
    pub pos: u8,
    pub neg: u8,
    pub spl: String,
    pub types: Vec<i32>,
    pub neg_role: i32,
    pub neg_b: usize,
}
#[derive(Clone, Debug)]
struct Dist {
    key: String,
    to: Vec<(usize, f64)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CoreLock {
    None,
    Neg,
    Pos(i32),
}
#[derive(Clone, Debug)]
struct CoreAction {
    splice: bool,
    lock: CoreLock,
    recipe: usize,
    dist: usize,
    cost: f64,
    locked: bool,
    drop: bool,
}
#[derive(Clone, Debug)]
struct Keep {
    q: f64,
    expected: f64,
    keep: Vec<(usize, f64, f64)>,
    cumulative: Vec<f64>,
}
#[derive(Clone, Debug)]
struct SortedDist {
    ids: Vec<usize>,
    ps: Vec<f64>,
    values: Vec<f64>,
    chance: Vec<f64>,
    need: usize,
    sums: Vec<f64>,
    filled: usize,
}
#[derive(Clone, Debug)]
enum PolicyStep {
    Cycle {
        id: usize,
        locked: bool,
        s: usize,
        k: usize,
        q: f64,
    },
    Splice {
        id: usize,
        dist: usize,
    },
}
impl PolicyStep {
    fn id(&self) -> usize {
        match *self {
            Self::Cycle { id, .. } | Self::Splice { id, .. } => id,
        }
    }
}
#[derive(Clone, Debug)]
struct Steps {
    list: Vec<PolicyStep>,
    sorted: Vec<SortedDist>,
    by_dist: HashMap<usize, usize>,
}
pub struct Solver {
    pub pool: Pool,
    pub target: Target,
    pub objective: Objective,
    pub roles: Vec<Role>,
    role_of_stat: HashMap<String, usize>,
    states: Vec<State>,
    by_key: HashMap<String, usize>,
    dists: Vec<Dist>,
    dist_by_key: HashMap<String, usize>,
    cycles: HashMap<String, usize>,
    core_actions: Vec<CoreAction>,
    actions: Vec<Vec<usize>>,
    pub value: Vec<f64>,
    best: Vec<Option<usize>>,
    relevant_recipes: Vec<usize>,
    spliced_groups: HashMap<String, Vec<usize>>,
    sets: Vec<Vec<usize>>,
    type_stride: i32,
    keep_cache: HashMap<(usize, u64, u64), Keep>,
    kuva_policy: Option<Vec<f64>>,
    steps: Option<Steps>,
    work: usize,
    account: SharedWorkAccount,
}
pub fn cycle_cost(done: u64, locked: bool) -> u64 {
    KUVA_STEPS
        .get(done as usize)
        .copied()
        .unwrap_or(KUVA_UNLOCKED)
        * if locked { 2 } else { 1 }
}
pub fn cycle_cost_total(done: u64, count: u64, locked: bool) -> f64 {
    let cheap = count.min(9u64.saturating_sub(done));
    let mut total = 0f64;
    for j in 0..cheap {
        total += KUVA_STEPS[(done + j) as usize] as f64;
    }
    (total + (count - cheap) as f64 * KUVA_UNLOCKED as f64) * if locked { 2.0 } else { 1.0 }
}
impl Solver {
    pub fn new(pool: Pool, target: Target, objective: Objective) -> Self {
        Self::with_account(
            pool,
            target,
            objective,
            Arc::new(Mutex::new(WorkAccount {
                used: 0,
                limit: WORK_BUDGET,
            })),
        )
    }
    pub fn with_account(
        pool: Pool,
        target: Target,
        objective: Objective,
        account: SharedWorkAccount,
    ) -> Self {
        let sets = if target.sets.is_empty() {
            vec![(0..target.groups.len()).collect()]
        } else {
            target.sets.clone()
        };
        let mut s = Self {
            pool,
            target,
            objective,
            roles: vec![],
            role_of_stat: HashMap::new(),
            states: vec![],
            by_key: HashMap::new(),
            dists: vec![],
            dist_by_key: HashMap::new(),
            cycles: HashMap::new(),
            core_actions: vec![],
            actions: vec![],
            value: vec![],
            best: vec![],
            relevant_recipes: vec![],
            spliced_groups: HashMap::new(),
            sets,
            type_stride: 1,
            keep_cache: HashMap::new(),
            kuva_policy: None,
            steps: None,
            work: 0,
            account,
        };
        s.build_roles();
        s.type_stride = 1 + s.roles.iter().map(|r| r.th.len() as i32).max().unwrap_or(0);
        s
    }
    pub fn state_count(&self) -> usize {
        self.states.len()
    }
    pub fn work_used(&self) -> usize {
        self.work
    }
    pub fn state(&self, id: usize) -> Option<&State> {
        self.states.get(id)
    }
    fn type_code(&self, r: usize, b: usize) -> i32 {
        r as i32 * self.type_stride + b as i32
    }
    fn type_role(&self, t: i32) -> Option<usize> {
        if t == JUNK {
            None
        } else {
            Some((t / self.type_stride) as usize)
        }
    }
    fn type_bucket(&self, t: i32) -> usize {
        if t == JUNK {
            0
        } else {
            (t % self.type_stride) as usize
        }
    }
    fn need(&self, g: usize, tag: &str, lk: &str) -> f64 {
        self.target.groups[g]
            .min_w_by
            .as_ref()
            .and_then(|by| by.get(tag))
            .and_then(|x| x.get(lk))
            .copied()
            .unwrap_or(self.target.groups[g].min_w)
    }
    fn tag_th(&self, g: usize, tag: &str) -> Vec<f64> {
        let mut bars: Vec<f64> = self
            .target
            .layouts
            .iter()
            .map(|l| self.need(g, tag, &format!("{}{}", l.pos, l.neg)))
            .collect();
        if let Some(a) = self.target.at_least {
            let high: Vec<_> = bars.iter().map(|b| b.max(a.w)).collect();
            bars.extend(high)
        }
        bars.retain(|x| *x > W_MIN && *x <= W_MAX);
        unique_floats(bars)
    }
    fn spl_th(&self, tag: &str) -> Vec<f64> {
        unique_floats(
            self.spliced_groups
                .get(tag)
                .into_iter()
                .flatten()
                .flat_map(|&g| self.tag_th(g, tag))
                .collect(),
        )
    }
    fn build_roles(&mut self) {
        let mut groups_of: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, g) in self.target.groups.iter().enumerate() {
            for t in &g.tags {
                groups_of.entry(t.clone()).or_default().push(i);
            }
        }
        for (t, gs) in &groups_of {
            if self.pool.spliced.contains(t) {
                self.spliced_groups.insert(t.clone(), gs.clone());
            }
        }
        self.relevant_recipes = self
            .pool
            .splices
            .iter()
            .enumerate()
            .filter_map(|(i, r)| groups_of.contains_key(&r.result).then_some(i))
            .collect();
        let mut tags = self.pool.pos.clone();
        for t in &self.pool.neg {
            if !tags.contains(t) {
                tags.push(t.clone())
            }
        }
        let mut sig: HashMap<String, usize> = HashMap::new();
        for tag in tags {
            let recipes: Vec<_> = self
                .relevant_recipes
                .iter()
                .copied()
                .filter(|&i| self.pool.splices[i].a == tag || self.pool.splices[i].b == tag)
                .collect();
            let groups = groups_of.get(&tag).cloned().unwrap_or_default();
            let can_pos = self.pool.pos.contains(&tag);
            let can_neg = self.pool.neg.contains(&tag);
            let allowed = self.target.allowed_neg.contains(&tag);
            let nth = unique_floats(
                recipes
                    .iter()
                    .flat_map(|&i| self.spl_th(&self.pool.splices[i].result))
                    .collect(),
            );
            let mut th: Vec<_> = groups.iter().flat_map(|&g| self.tag_th(g, &tag)).collect();
            th.extend(nth.iter().copied());
            let th = unique_floats(th);
            let own = !recipes.is_empty()
                || groups
                    .iter()
                    .any(|&g| self.target.groups[g].min_w_by.is_some());
            let key = format!(
                "{:?}|{:?}|{can_pos}|{can_neg}|{allowed}|{:?}|{}|{}",
                if own { tag.as_str() } else { "" },
                groups,
                th,
                self.pool.kept_pos.contains(&tag),
                self.pool.kept_neg.contains(&tag)
            );
            let id = if let Some(&id) = sig.get(&key) {
                id
            } else {
                let id = self.roles.len();
                sig.insert(key, id);
                self.roles.push(Role {
                    id,
                    stats: vec![],
                    can_pos,
                    can_neg,
                    allowed,
                    groups,
                    recipes,
                    th,
                    nth,
                });
                id
            };
            self.roles[id].stats.push(tag.clone());
            self.role_of_stat.insert(tag, id);
        }
    }
    fn bucket(&self, r: usize, w: f64) -> usize {
        self.roles[r].th.iter().filter(|&&t| w >= t).count()
    }
    fn bucket_probs(th: &[f64]) -> Vec<f64> {
        let mut e = vec![W_MIN];
        e.extend(th);
        e.push(W_MAX);
        e.windows(2)
            .map(|x| (x[1] - x[0]) / (W_MAX - W_MIN))
            .collect()
    }
    fn spliced_meets(&self, tag: &str, level: usize, t: f64) -> bool {
        if t <= W_MIN {
            return true;
        }
        let th = self.spl_th(tag);
        level > 0 && th.get(level - 1).is_some_and(|x| *x >= t)
    }
    fn neg_meets(&self, s: &State, t: f64) -> bool {
        t <= W_MIN
            || (s.neg_role >= 0
                && s.neg_b > 0
                && self.roles[s.neg_role as usize].nth[s.neg_b - 1] >= t)
    }
    fn neg_draws(&self, r: usize) -> Vec<(usize, f64)> {
        if self.roles[r].nth.is_empty() {
            vec![(0, 1.0)]
        } else {
            Self::bucket_probs(&self.roles[r].nth)
                .into_iter()
                .enumerate()
                .filter(|(_, p)| *p > 0.)
                .collect()
        }
    }
    fn meets(&self, t: i32, w: f64) -> bool {
        w <= W_MIN
            || self.type_role(t).is_some_and(|r| {
                let b = self.type_bucket(t);
                b > 0 && self.roles[r].th[b - 1] >= w
            })
    }
    fn useful(&self, t: i32) -> bool {
        let Some(ri) = self.type_role(t) else {
            return false;
        };
        let r = &self.roles[ri];
        !r.recipes.is_empty()
            || (!r.groups.is_empty()
                && self.meets(
                    t,
                    r.groups
                        .iter()
                        .flat_map(|&g| {
                            self.target.layouts.iter().map(move |l| {
                                self.need(g, &r.stats[0], &format!("{}{}", l.pos, l.neg))
                            })
                        })
                        .fold(f64::INFINITY, f64::min),
                ))
    }
    fn charge(&mut self, n: usize) -> Res<()> {
        self.work += n;
        let mut a = self.account.lock().expect("work account poisoned");
        a.used += n;
        if a.used > a.limit {
            return Err(SolveError::TooComplex {
                work: a.used,
                states: self.states.len(),
            });
        }
        Ok(())
    }
    fn intern(
        &mut self,
        pos: u8,
        neg: u8,
        spl: String,
        mut types: Vec<i32>,
        neg_role: i32,
        neg_b: usize,
    ) -> Res<usize> {
        types.sort_unstable();
        let key = format!("{pos}{neg}|{spl}|{:?}|{neg_role}:{neg_b}", types);
        if let Some(&id) = self.by_key.get(&key) {
            return Ok(id);
        }
        self.charge(1)?;
        let id = self.states.len();
        self.states.push(State {
            id,
            key: key.clone(),
            pos,
            neg,
            spl,
            types,
            neg_role,
            neg_b,
        });
        self.by_key.insert(key, id);
        self.actions.push(vec![]);
        Ok(id)
    }
    pub fn state_of(&mut self, c: &Card) -> Res<usize> {
        let types = c.pos.iter().map(|t| self.type_of_trait(t)).collect();
        let spl = if let Some(t) = &c.spliced {
            if self.spliced_groups.contains_key(&t.tag) {
                format!(
                    "{}:{}",
                    t.tag,
                    self.spl_th(&t.tag).iter().filter(|&&w| t.w >= w).count()
                )
            } else {
                OTHER_SPLICED.into()
            }
        } else {
            String::new()
        };
        let nr = c.neg.as_ref().map_or(-1, |t| {
            self.role_of_stat.get(&t.tag).map_or(-2, |&r| r as i32)
        });
        let nb = if nr >= 0 {
            self.roles[nr as usize]
                .nth
                .iter()
                .filter(|&&w| c.neg.as_ref().is_some_and(|t| t.w >= w))
                .count()
        } else {
            0
        };
        self.intern(c.layout.pos, c.layout.neg, spl, types, nr, nb)
    }
    fn type_of_trait(&self, t: &CardTrait) -> i32 {
        self.role_of_stat
            .get(&t.tag)
            .map_or(JUNK, |&r| self.type_code(r, self.bucket(r, t.w)))
    }
    pub fn is_goal(&self, id: usize) -> bool {
        let s = &self.states[id];
        self.target
            .layouts
            .iter()
            .any(|l| l.pos == s.pos && l.neg == s.neg)
            && (s.neg == 0 || (s.neg_role >= 0 && self.roles[s.neg_role as usize].allowed))
            && self.sets.iter().any(|set| self.meets_set(s, set))
    }
    pub fn is_goal_state(&self, id: usize) -> bool {
        self.is_goal(id)
    }
    fn meets_set(&self, s: &State, set: &[usize]) -> bool {
        let (tag, level) = split_spl(&s.spl);
        let lk = format!("{}{}", s.pos, s.neg);
        let mut matches = vec![];
        for &gi in set {
            if gi >= self.target.groups.len() {
                return false;
            }
            let mut choices = vec![];
            if !tag.is_empty()
                && self
                    .spliced_groups
                    .get(tag)
                    .is_some_and(|gs| gs.contains(&gi))
            {
                let need = self.need(gi, tag, &lk);
                if self.spliced_meets(tag, level, need) {
                    choices.push((
                        s.types.len(),
                        self.target
                            .at_least
                            .is_some_and(|a| self.spliced_meets(tag, level, need.max(a.w))),
                    ));
                }
            }
            for (slot, &t) in s.types.iter().enumerate() {
                if let Some(r) = self.type_role(t) {
                    if !self.roles[r].groups.contains(&gi) {
                        continue;
                    }
                    let need = self.need(gi, &self.roles[r].stats[0], &lk);
                    if self.meets(t, need) {
                        choices.push((
                            slot,
                            self.target
                                .at_least
                                .is_some_and(|a| self.meets(t, need.max(a.w))),
                        ));
                    }
                }
            }
            if choices.is_empty() {
                return false;
            }
            matches.push(choices)
        }
        if set.len() > s.types.len() + usize::from(!tag.is_empty()) {
            return false;
        }
        matches.sort_by_key(Vec::len);
        let high = self.target.at_least.map_or(0, |a| a.k.min(set.len()));
        match_slots(&matches, 0, 0, 0, high)
    }
    fn role_counts(&self, pos: bool) -> Vec<usize> {
        self.roles
            .iter()
            .map(|r| {
                r.stats
                    .iter()
                    .filter(|t| {
                        if pos {
                            r.can_pos
                                && self.pool.pos.contains(t)
                                && !self.pool.kept_pos.contains(t)
                        } else {
                            r.can_neg
                                && self.pool.neg.contains(t)
                                && !self.pool.kept_neg.contains(t)
                        }
                    })
                    .count()
            })
            .collect()
    }
    fn taken(counts: &mut [usize], r: usize, n: usize) {
        counts[r] = counts[r].saturating_sub(n)
    }
    fn bucket_draws(&self, r: usize, c: usize) -> Vec<(Vec<i32>, f64)> {
        let role = &self.roles[r];
        if role.groups.is_empty() && role.recipes.is_empty() {
            return vec![(vec![self.type_code(r, 0); c], 1.)];
        }
        let probs = Self::bucket_probs(&role.th);
        let mut out = vec![];
        fn rec(
            s: &Solver,
            r: usize,
            c: usize,
            i: usize,
            left: usize,
            pick: &mut [usize],
            probs: &[f64],
            out: &mut Vec<(Vec<i32>, f64)>,
        ) {
            if i + 1 == probs.len() {
                pick[i] = left;
                let mut p = factorial(c);
                let mut types = vec![];
                for (b, &n) in pick.iter().enumerate() {
                    p *= probs[b].powi(n as i32) / factorial(n);
                    for _ in 0..n {
                        types.push(s.type_code(r, b));
                    }
                }
                if p > 0. {
                    out.push((types, p))
                }
                return;
            }
            for n in 0..=left {
                pick[i] = n;
                rec(s, r, c, i + 1, left - n, pick, probs, out)
            }
        }
        rec(
            self,
            r,
            c,
            0,
            c,
            &mut vec![0; probs.len()],
            &probs,
            &mut out,
        );
        out
    }
    fn store_dist(&mut self, key: String, to: Vec<(usize, f64)>) -> usize {
        let id = self.dists.len();
        self.dists.push(Dist {
            key: key.clone(),
            to,
        });
        self.dist_by_key.insert(key, id);
        id
    }
    fn add_outcome(
        &mut self,
        to: &mut Vec<(usize, f64)>,
        indices: &mut HashMap<usize, usize>,
        id: usize,
        p: f64,
    ) -> Res<()> {
        if let Some(&i) = indices.get(&id) {
            to[i].1 += p
        } else {
            self.charge(1)?;
            indices.insert(id, to.len());
            to.push((id, p));
        }
        Ok(())
    }
    fn cycle_dist(&mut self, s: &State, lock: CoreLock, drop: bool) -> Res<usize> {
        let spl = if drop { String::new() } else { s.spl.clone() };
        let fixed = lock != CoreLock::None || !spl.is_empty();
        let lock_key = match lock {
            CoreLock::None => "none".into(),
            CoreLock::Neg => format!("n{}:{}", s.neg_role, s.neg_b),
            CoreLock::Pos(t) => t.to_string(),
        };
        let key = format!(
            "c|{}|{spl}|{lock_key}",
            if fixed {
                format!("{}{}", s.pos, s.neg)
            } else {
                "*".into()
            }
        );
        if let Some(&id) = self.dist_by_key.get(&key) {
            return Ok(id);
        }
        let mut to = vec![];
        let mut indices = HashMap::new();
        let layouts = if fixed {
            vec![Layout {
                pos: s.pos,
                neg: s.neg,
            }]
        } else {
            LAYOUTS.to_vec()
        };
        let lt = if let CoreLock::Pos(t) = lock {
            Some(t)
        } else {
            None
        };
        let lr = lt.and_then(|t| self.type_role(t));
        for l in &layouts {
            let pl = 1. / layouts.len() as f64;
            let used = usize::from(!spl.is_empty()) + usize::from(lt.is_some());
            if used > l.pos as usize {
                continue;
            }
            let free = l.pos as usize - used;
            let mut pc = self.role_counts(true);
            if let Some(r) = lr {
                Self::taken(&mut pc, r, 1)
            }
            if lock == CoreLock::Neg && s.neg_role >= 0 && self.roles[s.neg_role as usize].can_pos {
                Self::taken(&mut pc, s.neg_role as usize, 1)
            }
            for (pick, pp) in draws(&pc, free) {
                let mut combos = vec![(lt.into_iter().collect::<Vec<_>>(), 1.)];
                for (r, &c) in pick.iter().enumerate() {
                    if c == 0 {
                        continue;
                    }
                    let mut next = vec![];
                    for (types, p) in combos {
                        for (bt, bp) in self.bucket_draws(r, c) {
                            let mut ts = types.clone();
                            ts.extend(bt);
                            next.push((ts, p * bp))
                        }
                    }
                    combos = next
                }
                let mut negs = vec![(-1, 0, 1.)];
                if l.neg == 1 {
                    if lock == CoreLock::Neg {
                        negs = vec![(s.neg_role, s.neg_b, 1.)]
                    } else {
                        let mut nc = self.role_counts(false);
                        for (r, &c) in pick.iter().enumerate() {
                            if self.roles[r].can_neg {
                                Self::taken(&mut nc, r, c)
                            }
                        }
                        if let Some(r) = lr {
                            if self.roles[r].can_neg {
                                Self::taken(&mut nc, r, 1)
                            }
                        }
                        let total: usize = nc.iter().sum();
                        negs.clear();
                        for (r, &c) in nc.iter().enumerate() {
                            if c > 0 {
                                for (b, bp) in self.neg_draws(r) {
                                    negs.push((r as i32, b, c as f64 / total as f64 * bp))
                                }
                            }
                        }
                    }
                }
                for (types, p) in combos {
                    for &(nr, nb, np) in &negs {
                        let id = self.intern(l.pos, l.neg, spl.clone(), types.clone(), nr, nb)?;
                        self.add_outcome(&mut to, &mut indices, id, pl * pp * p * np)?;
                    }
                }
            }
        }
        Ok(self.store_dist(key, to))
    }
    fn splice_dist(&mut self, s: &State, ri: usize) -> Res<Option<usize>> {
        let r = self.pool.splices[ri].clone();
        let (Some(&ra), Some(&rb)) = (self.role_of_stat.get(&r.a), self.role_of_stat.get(&r.b))
        else {
            return Ok(None);
        };
        if !s.spl.is_empty() {
            return Ok(None);
        }
        let mut types = s.types.clone();
        let take = |ts: &mut Vec<i32>, role: usize| -> Option<i32> {
            let i = ts.iter().position(|&t| self.type_role(t) == Some(role))?;
            Some(ts.remove(i))
        };
        let ta = take(&mut types, ra);
        let tb = take(&mut types, rb);
        let positives = match (ta, tb) {
            (Some(a), Some(b)) => vec![a, b],
            (Some(a), _) if s.neg_role == rb as i32 => vec![a],
            (_, Some(b)) if s.neg_role == ra as i32 => vec![b],
            _ => return Ok(None),
        };
        let used_neg = positives.len() == 1;
        if !self.spliced_groups.contains_key(&r.result) {
            return Ok(None);
        }
        let level = self
            .spl_th(&r.result)
            .iter()
            .filter(|&&t| {
                positives.iter().any(|&p| self.meets(p, t)) || (used_neg && self.neg_meets(s, t))
            })
            .count();
        if !self.spliced_counts(&r.result, level, s.pos, s.neg) {
            return Ok(None);
        }
        let key = format!("s|{}|{ri}", s.key);
        if let Some(&id) = self.dist_by_key.get(&key) {
            return Ok(Some(id));
        }
        let to = self.splice_to(s, types, used_neg, format!("!{}:{level}", r.result))?;
        Ok(Some(self.store_dist(key, to)))
    }
    fn splice_to(
        &mut self,
        s: &State,
        types: Vec<i32>,
        used_neg: bool,
        spl: String,
    ) -> Res<Vec<(usize, f64)>> {
        let mut to = vec![];
        let mut indices = HashMap::new();
        if used_neg {
            let mut nc = self.role_counts(false);
            for &t in &types {
                if let Some(r) = self.type_role(t) {
                    if self.roles[r].can_neg {
                        Self::taken(&mut nc, r, 1)
                    }
                }
            }
            let total: usize = nc.iter().sum();
            for (r, &c) in nc.iter().enumerate() {
                if c > 0 {
                    for (b, bp) in self.neg_draws(r) {
                        let id =
                            self.intern(s.pos, s.neg, spl.clone(), types.clone(), r as i32, b)?;
                        self.add_outcome(&mut to, &mut indices, id, c as f64 / total as f64 * bp)?;
                    }
                }
            }
        } else {
            let mut pc = self.role_counts(true);
            for &t in &types {
                if let Some(r) = self.type_role(t) {
                    Self::taken(&mut pc, r, 1)
                }
            }
            if s.neg_role >= 0 && self.roles[s.neg_role as usize].can_pos {
                Self::taken(&mut pc, s.neg_role as usize, 1)
            }
            for (pick, pp) in draws(&pc, 1) {
                let Some(role) = pick.iter().position(|&c| c > 0) else {
                    continue;
                };
                for (bt, bp) in self.bucket_draws(role, 1) {
                    let mut ts = types.clone();
                    ts.extend(bt);
                    let id = self.intern(s.pos, s.neg, spl.clone(), ts, s.neg_role, s.neg_b)?;
                    self.add_outcome(&mut to, &mut indices, id, pp * bp)?;
                }
            }
        }
        Ok(to)
    }
    fn cycle_action(&mut self, s: &State, lock: CoreLock, drop: bool) -> Res<usize> {
        let dist = self.cycle_dist(s, lock, drop)?;
        let key = format!("{}|{drop}", self.dists[dist].key);
        if let Some(&id) = self.cycles.get(&key) {
            return Ok(id);
        }
        let locked = lock != CoreLock::None;
        let cost = if self.objective == Objective::Cycles {
            1.
        } else {
            KUVA_UNLOCKED as f64 * if locked { 2. } else { 1. }
        };
        let id = self.core_actions.len();
        self.core_actions.push(CoreAction {
            splice: false,
            lock,
            recipe: usize::MAX,
            dist,
            cost,
            locked,
            drop,
        });
        self.cycles.insert(key, id);
        Ok(id)
    }
    fn actions_for(&mut self, s: &State) -> Res<Vec<usize>> {
        let mut out = vec![];
        let drop_choices = if self.can_unlock_spliced(s) {
            vec![false, true]
        } else {
            vec![false]
        };
        for drop in drop_choices {
            out.push(self.cycle_action(s, CoreLock::None, drop)?);
            let mut types = HashSet::new();
            for &t in &s.types {
                if t != JUNK && types.insert(t) {
                    out.push(self.cycle_action(s, CoreLock::Pos(t), drop)?);
                }
            }
            if s.neg == 1 && s.neg_role >= 0 {
                out.push(self.cycle_action(s, CoreLock::Neg, drop)?);
            }
        }
        if s.spl.is_empty() {
            for ri in self.relevant_recipes.clone() {
                if let Some(dist) = self.splice_dist(s, ri)? {
                    let id = self.core_actions.len();
                    self.core_actions.push(CoreAction {
                        splice: true,
                        lock: CoreLock::None,
                        recipe: ri,
                        dist,
                        cost: 0.,
                        locked: false,
                        drop: false,
                    });
                    out.push(id)
                }
            }
        }
        Ok(out)
    }
    fn can_unlock_spliced(&self, s: &State) -> bool {
        if s.spl.is_empty() || s.spl.starts_with('!') {
            return false;
        }
        let (tag, level) = split_spl(&s.spl);
        !self.spliced_counts(tag, level, s.pos, s.neg)
    }
    fn spliced_counts(&self, tag: &str, level: usize, pos: u8, neg: u8) -> bool {
        if !self
            .target
            .layouts
            .iter()
            .any(|l| l.pos == pos && l.neg == neg)
        {
            return false;
        }
        if neg == 1 && !self.roles.iter().any(|r| r.can_neg && r.allowed) {
            return false;
        }
        let mine = self.spliced_groups.get(tag).cloned().unwrap_or_default();
        let lk = format!("{pos}{neg}");
        let plain = |g: usize| {
            self.target.groups[g].tags.iter().any(|t| {
                self.pool.pos.contains(t)
                    && !self.pool.spliced.contains(t)
                    && self.need(g, t, &lk) <= W_MAX
            })
        };
        self.sets.iter().any(|set| {
            set.len() <= pos as usize
                && set.iter().any(|&gi| {
                    if !mine.contains(&gi) {
                        return false;
                    }
                    let need = self.need(gi, tag, &lk);
                    if !self.spliced_meets(tag, level, need)
                        || !set.iter().all(|&g| g == gi || plain(g))
                    {
                        return false;
                    }
                    self.target.at_least.is_none_or(|a| {
                        self.spliced_meets(tag, level, need.max(a.w))
                            || set.len().saturating_sub(1) >= a.k.min(set.len())
                    })
                })
        })
    }
    pub fn solve(&mut self, start: &Card) -> Res<usize> {
        self.keep_cache.clear();
        self.kuva_policy = None;
        self.steps = None;
        let s0 = self.state_of(start)?;
        let mut queue = vec![s0];
        let mut seen = HashSet::from([s0]);
        let mut walked = HashSet::new();
        while let Some(id) = queue.pop() {
            let s = self.states[id].clone();
            let acts = self.actions_for(&s)?;
            self.charge(acts.len())?;
            self.actions[id] = acts.clone();
            for aid in acts {
                let d = self.core_actions[aid].dist;
                if !walked.insert(d) {
                    continue;
                }
                for &(to, _) in &self.dists[d].to {
                    if seen.insert(to) {
                        queue.push(to)
                    }
                }
            }
        }
        let n = self.states.len();
        self.value = vec![f64::INFINITY; n];
        self.best = vec![None; n];
        for _ in 0..50 {
            let before = self.value.clone();
            self.sweep(true);
            self.sweep(false);
            let changed = (0..n).any(|i| {
                let a = before[i];
                let b = self.value[i];
                a != b && (a - b).abs() > 1e-12 * b.abs().max(1.)
            });
            if !changed {
                break;
            }
        }
        Ok(s0)
    }
    fn sweep(&mut self, spliced: bool) {
        let n = self.states.len();
        let ids: Vec<_> = self
            .states
            .iter()
            .filter(|s| !s.spl.is_empty() == spliced && !self.actions[s.id].is_empty())
            .map(|s| s.id)
            .collect();
        let mut in_set = vec![false; n];
        for &id in &ids {
            in_set[id] = true;
            self.value[id] = f64::INFINITY;
            self.best[id] = None
        }
        let mut settled = vec![false; n];
        let mut outside = vec![false; n];
        let mut acc_of: HashMap<(usize, u64), usize> = HashMap::new();
        let mut acc_at: HashMap<usize, usize> = HashMap::new();
        let mut acc_dist = vec![];
        let mut acc_cost = vec![];
        let mut users: Vec<Vec<(usize, usize)>> = vec![];
        let mut heap = BinaryHeap::new();
        let mut serial = 0usize;
        for &id in &ids {
            let mut t = if self.is_goal(id) { 0. } else { f64::INFINITY };
            for (ai, &aid) in self.actions[id].iter().enumerate() {
                let a = &self.core_actions[aid];
                if a.splice {
                    let v = self.dists[a.dist]
                        .to
                        .iter()
                        .map(|&(to, p)| p * self.value[to])
                        .sum();
                    if v < t {
                        t = v;
                        self.best[id] = Some(ai)
                    }
                    continue;
                }
                let k = if let Some(&k) = acc_at.get(&aid) {
                    k
                } else {
                    let key = (a.dist, a.cost.to_bits());
                    let k = if let Some(&k) = acc_of.get(&key) {
                        k
                    } else {
                        let k = acc_dist.len();
                        acc_of.insert(key, k);
                        acc_dist.push(a.dist);
                        acc_cost.push(a.cost);
                        users.push(vec![]);
                        for &(to, _) in &self.dists[a.dist].to {
                            if !in_set[to] && !outside[to] && self.value[to].is_finite() {
                                outside[to] = true;
                                heap_push(&mut heap, &mut serial, to as i64, self.value[to]);
                            }
                        }
                        k
                    };
                    acc_at.insert(aid, k);
                    k
                };
                users[k].push((id, ai));
            }
            if self.is_goal(id) {
                self.best[id] = None
            }
            if t.is_finite() {
                heap_push(&mut heap, &mut serial, id as i64, t)
            }
        }
        let ac = acc_dist.len();
        let mut num = vec![0.; ac];
        let mut den = vec![0.; ac];
        let mut left: Vec<_> = users.iter().map(Vec::len).collect();
        let mut rev: Vec<Vec<(usize, f64)>> = vec![vec![]; n];
        for (k, &di) in acc_dist.iter().enumerate() {
            for &(to, p) in &self.dists[di].to {
                if in_set[to] || self.value[to].is_finite() {
                    rev[to].push((k, p))
                }
            }
        }
        while let Some(h) = heap.pop() {
            let code = h.code;
            let v = h.value;
            if code < 0 {
                let k = (-1 - code) as usize;
                let cand = if den[k] > 0. {
                    (acc_cost[k] + num[k]) / den[k]
                } else {
                    f64::INFINITY
                };
                if left[k] == 0 || v != cand {
                    continue;
                }
                for &(id, ai) in &users[k] {
                    if settled[id] {
                        continue;
                    }
                    self.best[id] = Some(ai);
                    settled[id] = true;
                    self.value[id] = v;
                    for aid in &self.actions[id] {
                        if let Some(&a) = acc_at.get(aid) {
                            left[a] -= 1
                        }
                    }
                    reach(
                        id,
                        v,
                        &rev,
                        &mut num,
                        &mut den,
                        &left,
                        &acc_cost,
                        &mut heap,
                        &mut serial,
                    );
                }
                continue;
            }
            let id = code as usize;
            if outside[id] {
                reach(
                    id,
                    v,
                    &rev,
                    &mut num,
                    &mut den,
                    &left,
                    &acc_cost,
                    &mut heap,
                    &mut serial,
                )
            } else if !settled[id] {
                settled[id] = true;
                self.value[id] = v;
                for aid in &self.actions[id] {
                    if let Some(&k) = acc_at.get(aid) {
                        left[k] -= 1
                    }
                }
                reach(
                    id,
                    v,
                    &rev,
                    &mut num,
                    &mut den,
                    &left,
                    &acc_cost,
                    &mut heap,
                    &mut serial,
                );
            }
        }
    }
    fn keep_set(&mut self, aid: usize, v0: f64) -> Res<Keep> {
        let a = self.core_actions[aid].clone();
        let key = (a.dist, a.cost.to_bits(), v0.to_bits());
        if let Some(k) = self.keep_cache.get(&key) {
            return Ok(k.clone());
        }
        let bar = v0 - 1e-7 * v0.abs().max(1.);
        let entries = self.dists[a.dist].to.clone();
        let mut keep = vec![];
        for (to, p) in entries {
            let v = self.value[to];
            if p > 0. && v < bar {
                self.charge(1)?;
                keep.push((to, p, v));
            }
        }
        let mut num = a.cost;
        let mut den = 0.;
        for &(_, p, v) in &keep {
            num += p * v;
            den += p
        }
        let mut cumulative = vec![];
        let mut p = 0.;
        for &(_, q, _) in &keep {
            p += q / den;
            cumulative.push(p)
        }
        if let Some(last) = cumulative.last_mut() {
            *last = 1.
        }
        let result = Keep {
            q: den.min(1.),
            expected: if den > 0. { num / den } else { f64::INFINITY },
            keep,
            cumulative,
        };
        self.keep_cache.insert(key, result.clone());
        Ok(result)
    }
    fn describe(
        &self,
        keep: &[(usize, f64)],
        a: &CoreAction,
        lock_tag: Option<&str>,
    ) -> Vec<Outcome> {
        if keep.is_empty() {
            return vec![];
        }
        let total: f64 = keep.iter().map(|x| x.1).sum();
        let mut agg: Vec<Outcome> = vec![];
        let mut indices: HashMap<String, usize> = HashMap::new();
        for &(to, p) in keep {
            let s = &self.states[to];
            let mut types = s.types.clone();
            if let CoreLock::Pos(t) = a.lock {
                if let Some(i) = types.iter().position(|&v| v == t) {
                    types.remove(i);
                }
            }
            let mut needs = vec![];
            let mut by_role: Vec<(usize, f64, Vec<String>, usize)> = vec![];
            for t in types {
                if !self.useful(t) {
                    continue;
                }
                let r = &self.roles[self.type_role(t).unwrap()];
                let b = self.type_bucket(t);
                let min_w = if b > 0 { r.th[b - 1] } else { W_MIN };
                let stats = r
                    .stats
                    .iter()
                    .filter(|x| lock_tag.is_none_or(|t| x.as_str() != t))
                    .cloned()
                    .collect::<Vec<_>>();
                if let Some(g) = by_role.iter_mut().find(|g| g.0 == r.id && g.1 == min_w) {
                    g.3 += 1
                } else {
                    by_role.push((r.id, min_w, stats, 1));
                }
            }
            for (_, min_w, stats, n) in by_role {
                if n == stats.len() {
                    for x in stats {
                        needs.push(Need {
                            stats: vec![x],
                            min_w,
                            neg: false,
                        })
                    }
                } else {
                    for _ in 0..n {
                        needs.push(Need {
                            stats: stats.clone(),
                            min_w,
                            neg: false,
                        })
                    }
                }
            }
            needs.sort_by(|a, b| a.stats.first().cmp(&b.stats.first()));
            if s.neg == 1
                && s.neg_role >= 0
                && !self.roles[s.neg_role as usize].recipes.is_empty()
                && a.lock != CoreLock::Neg
            {
                let r = &self.roles[s.neg_role as usize];
                needs.push(Need {
                    stats: r.stats.clone(),
                    min_w: if s.neg_b > 0 {
                        r.nth[s.neg_b - 1]
                    } else {
                        W_MIN
                    },
                    neg: true,
                });
            }
            let neg_allowed = if s.neg == 1 {
                Some(s.neg_role >= 0 && self.roles[s.neg_role as usize].allowed)
            } else {
                None
            };
            let layout = Layout {
                pos: s.pos,
                neg: s.neg,
            };
            let key = format!("{:?}|{:?}|{:?}", needs, neg_allowed, layout);
            if let Some(&i) = indices.get(&key) {
                agg[i].prob += p / total
            } else {
                indices.insert(key, agg.len());
                agg.push(Outcome {
                    prob: p / total,
                    needs,
                    neg_allowed,
                    layout,
                });
            }
        }
        agg.sort_by(|a, b| b.prob.total_cmp(&a.prob));
        agg
    }
    pub fn routes(&mut self, card: &Card) -> Res<Vec<Route>> {
        let sid = self.state_of(card)?;
        let Some(&v0) = self.value.get(sid) else {
            return Err(invalid("Card has not been solved"));
        };
        let acts = self.actions[sid].clone();
        let mut out = vec![];
        for (ai, aid) in acts.into_iter().enumerate() {
            let a = self.core_actions[aid].clone();
            if a.splice {
                let expected = self.dists[a.dist]
                    .to
                    .iter()
                    .map(|&(to, p)| p * self.value[to])
                    .sum::<f64>();
                if !expected.is_finite() {
                    continue;
                }
                let r = &self.pool.splices[a.recipe];
                out.push(Route {
                    action: Action::Splice {
                        result: r.result.clone(),
                        a: r.a.clone(),
                        b: r.b.clone(),
                    },
                    step_chance: 1.,
                    step_cycles: 0.,
                    step_cycles_p90: 0.,
                    expected,
                    outcomes: vec![],
                    worse: vec![],
                    optimal: self.best[sid] == Some(ai),
                });
                continue;
            }
            let k = self.keep_set(aid, v0)?;
            if k.q <= 0. || !k.expected.is_finite() {
                continue;
            }
            let lock = match a.lock {
                CoreLock::None => Lock::Simple(SimpleLock::None),
                CoreLock::Neg => Lock::Simple(SimpleLock::Neg),
                CoreLock::Pos(t) => {
                    let ct = card
                        .pos
                        .iter()
                        .find(|x| self.type_of_trait(x) == t)
                        .ok_or_else(|| invalid("Locked trait not on card"))?;
                    Lock::Pos(PositiveLock {
                        tag: ct.tag.clone(),
                        stats: self.roles[self.type_role(t).unwrap()].stats.clone(),
                        w: ct.w,
                    })
                }
            };
            let lock_tag = match &lock {
                Lock::Pos(t) => Some(t.tag.as_str()),
                _ => None,
            };
            let keep: Vec<_> = k.keep.iter().map(|&(to, p, _)| (to, p)).collect();
            let worse: Vec<_> = self.dists[a.dist]
                .to
                .iter()
                .copied()
                .filter(|&(to, p)| p > 0. && self.value[to] > v0 + 1e-7 * v0.abs().max(1.))
                .collect();
            let outcomes = self.describe(&keep, &a, lock_tag);
            let worse = self.describe(&worse, &a, lock_tag);
            out.push(Route {
                action: Action::Cycle { lock, drop: a.drop },
                step_chance: k.q,
                step_cycles: 1. / k.q,
                step_cycles_p90: if k.q >= 1. {
                    1.
                } else {
                    (0.1f64.ln() / (-k.q).ln_1p()).ceil()
                },
                expected: k.expected,
                outcomes,
                worse,
                optimal: self.best[sid] == Some(ai),
            });
        }
        out.sort_by(|a, b| a.expected.total_cmp(&b.expected));
        let mut i = 0;
        while i < out.len() {
            let mut j = i + 1;
            while j < out.len()
                && out[j].expected - out[i].expected <= 1e-9 * out[i].expected.abs().max(1.)
            {
                j += 1
            }
            let indices: Vec<_> = (i..j).filter(|&x| lock_w(&out[x]).is_some()).collect();
            let recommended = indices.iter().any(|&x| out[x].optimal);
            let mut locks: Vec<_> = indices.iter().map(|&x| out[x].clone()).collect();
            locks.sort_by(|a, b| lock_w(b).unwrap().total_cmp(&lock_w(a).unwrap()));
            if recommended {
                for r in &mut locks {
                    r.optimal = false
                }
                if let Some(r) = locks.first_mut() {
                    r.optimal = true
                }
            }
            for (x, r) in indices.into_iter().zip(locks) {
                out[x] = r
            }
            i = j;
        }
        Ok(out)
    }
    fn sorted_of(&mut self, di: usize, steps: &mut Steps) -> Res<usize> {
        if let Some(&si) = steps.by_dist.get(&di) {
            return Ok(si);
        }
        let mut list: Vec<_> = self.dists[di]
            .to
            .iter()
            .copied()
            .filter(|x| x.1 > 0.)
            .collect();
        list.sort_by(|a, b| self.value[a.0].total_cmp(&self.value[b.0]));
        self.charge(list.len())?;
        let n = list.len();
        let mut chance = vec![0.; n + 1];
        for i in 0..n {
            chance[i + 1] = chance[i] + list[i].1
        }
        let si = steps.sorted.len();
        steps.sorted.push(SortedDist {
            ids: list.iter().map(|x| x.0).collect(),
            ps: list.iter().map(|x| x.1).collect(),
            values: list.iter().map(|x| self.value[x.0]).collect(),
            chance,
            need: 0,
            sums: vec![0.; n + 1],
            filled: 0,
        });
        steps.by_dist.insert(di, si);
        Ok(si)
    }
    fn kept_prefix(&mut self, aid: usize, v0: f64, steps: &mut Steps) -> Res<(usize, usize, f64)> {
        let si = self.sorted_of(self.core_actions[aid].dist, steps)?;
        let s = &mut steps.sorted[si];
        let bar = v0 - 1e-7 * v0.abs().max(1.);
        let k = s.values.partition_point(|v| *v < bar);
        s.need = s.need.max(k);
        Ok((si, k, s.chance[k].min(1.)))
    }
    fn policy_steps(&mut self) -> Res<Steps> {
        if let Some(s) = &self.steps {
            return Ok(s.clone());
        }
        let mut steps = Steps {
            list: vec![],
            sorted: vec![],
            by_dist: HashMap::new(),
        };
        let mut order: Vec<_> = self
            .states
            .iter()
            .filter(|s| self.value[s.id].is_finite() && !self.is_goal(s.id))
            .map(|s| s.id)
            .collect();
        order.sort_by(|&a, &b| self.value[a].total_cmp(&self.value[b]));
        for id in order {
            let ai = self.best[id].ok_or_else(|| invalid("Solved policy lacks action"))?;
            let aid = self.actions[id][ai];
            let a = self.core_actions[aid].clone();
            let step = if a.splice {
                PolicyStep::Splice { id, dist: a.dist }
            } else {
                let (s, k, q) = self.kept_prefix(aid, self.value[id], &mut steps)?;
                PolicyStep::Cycle {
                    id,
                    locked: a.locked,
                    s,
                    k,
                    q,
                }
            };
            steps.list.push(step)
        }
        self.steps = Some(steps.clone());
        Ok(steps)
    }
    fn policy_kuva(&mut self) -> Res<Vec<f64>> {
        if self.objective == Objective::Kuva {
            return Ok(self.value.clone());
        }
        if let Some(v) = &self.kuva_policy {
            return Ok(v.clone());
        }
        let mut steps = self.policy_steps()?;
        let mut values: Vec<_> = self
            .value
            .iter()
            .map(|v| v * KUVA_UNLOCKED as f64)
            .collect();
        for _ in 0..1000 {
            let mut converged = true;
            for s in &mut steps.sorted {
                s.filled = 0
            }
            for step in &steps.list {
                let id = step.id();
                let next = match *step {
                    PolicyStep::Splice { dist, .. } => self.dists[dist]
                        .to
                        .iter()
                        .filter(|x| x.1 > 0.)
                        .map(|&(to, p)| p * values[to])
                        .sum(),
                    PolicyStep::Cycle {
                        locked, s, k, q, ..
                    } => {
                        let sd = &mut steps.sorted[s];
                        while sd.filled < k {
                            let i = sd.filled;
                            sd.sums[i + 1] = sd.sums[i] + sd.ps[i] * values[sd.ids[i]];
                            sd.filled += 1;
                        }
                        if q > 0. {
                            (KUVA_UNLOCKED as f64 * if locked { 2. } else { 1. } + sd.sums[k]) / q
                        } else {
                            f64::INFINITY
                        }
                    }
                };
                if !next.is_finite() {
                    return Err(invalid(
                        "The selected policy has no finite Kuva expectation",
                    ));
                }
                if (next - values[id]).abs() > 1e-11 * next.abs().max(1.) {
                    converged = false
                }
                values[id] = next;
            }
            if converged {
                self.kuva_policy = Some(values.clone());
                self.steps = Some(steps);
                return Ok(values);
            }
        }
        Err(invalid(
            "The selected policy's Kuva expectation did not converge",
        ))
    }
    fn full_price_action(&mut self, id: usize, aid: usize, values: &[f64]) -> Res<f64> {
        let a = self.core_actions[aid].clone();
        if a.splice {
            return Ok(self.dists[a.dist]
                .to
                .iter()
                .filter(|x| x.1 > 0.)
                .map(|&(to, p)| p * values[to])
                .sum());
        }
        let k = self.keep_set(aid, self.value[id])?;
        if k.q <= 0. {
            return Ok(f64::INFINITY);
        }
        let cost = KUVA_UNLOCKED as f64 * if a.locked { 2. } else { 1. };
        Ok((cost + k.keep.iter().map(|&(to, p, _)| p * values[to]).sum::<f64>()) / k.q)
    }
    fn same_action(&self, aid: usize, r: &Route, card: &Card) -> bool {
        let a = &self.core_actions[aid];
        match &r.action {
            Action::Splice { result, a: ra, b } => {
                a.splice && {
                    let sp = &self.pool.splices[a.recipe];
                    sp.result == *result && sp.a == *ra && sp.b == *b
                }
            }
            Action::Cycle { lock, drop } => {
                !a.splice
                    && a.drop == *drop
                    && match lock {
                        Lock::Simple(SimpleLock::None) => a.lock == CoreLock::None,
                        Lock::Simple(SimpleLock::Neg) => a.lock == CoreLock::Neg,
                        Lock::Pos(t) => card
                            .pos
                            .iter()
                            .find(|x| x.tag == t.tag)
                            .is_some_and(|x| a.lock == CoreLock::Pos(self.type_of_trait(x))),
                    }
            }
        }
    }
    pub fn expected_kuva(&mut self, card: &Card, first: &Route, cycles_done: u64) -> Res<f64> {
        Ok(self.expected_kuvas(card, std::slice::from_ref(first), cycles_done)?[0])
    }
    pub fn expected_kuvas(
        &mut self,
        card: &Card,
        routes: &[Route],
        cycles_done: u64,
    ) -> Res<Vec<f64>> {
        let start = self.state_of(card)?;
        if self.is_goal(start) {
            return Ok(vec![0.; routes.len()]);
        }
        let mut firsts = vec![];
        for r in routes {
            let aid = self.actions[start]
                .iter()
                .copied()
                .find(|&aid| self.same_action(aid, r, card))
                .ok_or_else(|| invalid("Unknown route for this card"))?;
            firsts.push(aid)
        }
        let terminal = self.policy_kuva()?;
        let best = self.best[start].map(|i| self.actions[start][i]);
        let mut route = vec![];
        for &aid in &firsts {
            route.push(if Some(aid) == best {
                terminal[start]
            } else {
                self.full_price_action(start, aid, &terminal)?
            })
        }
        if cycles_done >= 9 {
            return Ok(route);
        }
        let mut steps = self.policy_steps()?;
        let mut first_cycles = vec![];
        for &aid in &firsts {
            let a = self.core_actions[aid].clone();
            first_cycles.push(if a.splice {
                None
            } else {
                let (s, k, q) = self.kept_prefix(aid, self.value[start], &mut steps)?;
                Some((a.locked, s, k, q))
            })
        }
        let mut next = terminal;
        for done in (cycles_done..9).rev() {
            for s in &mut steps.sorted {
                for i in 0..s.need {
                    s.sums[i + 1] = s.sums[i] + s.ps[i] * next[s.ids[i]]
                }
            }
            let mut cur = next.clone();
            for c in &steps.list {
                if let PolicyStep::Cycle {
                    id,
                    locked,
                    s,
                    k,
                    q,
                } = *c
                {
                    cur[id] = cycle_cost(done, locked) as f64
                        + if q < 1. { (1. - q) * next[id] } else { 0. }
                        + steps.sorted[s].sums[k];
                }
            }
            for c in &steps.list {
                if let PolicyStep::Splice { id, dist } = *c {
                    cur[id] = self.dists[dist]
                        .to
                        .iter()
                        .filter(|x| x.1 > 0.)
                        .map(|&(to, p)| p * cur[to])
                        .sum();
                }
            }
            for (r, c) in first_cycles.iter().enumerate() {
                if let Some((locked, s, k, q)) = *c {
                    route[r] = cycle_cost(done, locked) as f64
                        + if q < 1. { (1. - q) * route[r] } else { 0. }
                        + steps.sorted[s].sums[k];
                }
            }
            next = cur;
        }
        let mut out = vec![];
        for (r, &aid) in firsts.iter().enumerate() {
            if let Some((_, _, _, q)) = first_cycles[r] {
                out.push(if q > 0. { route[r] } else { f64::INFINITY })
            } else {
                out.push(
                    self.dists[self.core_actions[aid].dist]
                        .to
                        .iter()
                        .filter(|x| x.1 > 0.)
                        .map(|&(to, p)| p * next[to])
                        .sum(),
                )
            }
        }
        self.steps = Some(steps);
        Ok(out)
    }
    pub fn simulate<F: FnMut() -> f64>(
        &mut self,
        card: &Card,
        first: &Route,
        cycles_done: u64,
        runs: usize,
        rng: &mut F,
    ) -> Res<Simulation> {
        let s0 = self.state_of(card)?;
        let first_index = self.actions[s0]
            .iter()
            .position(|&a| self.same_action(a, first, card))
            .ok_or_else(|| invalid("Unknown route for this card"))?;
        let mut output = Simulation {
            kuva: vec![],
            cycles: vec![],
            splicers: vec![],
        };
        for _ in 0..runs {
            let mut id = s0;
            let mut done = cycles_done;
            let mut ai = Some(first_index);
            let (mut kuva, mut cycles, mut splicers) = (0., 0., 0.);
            for _ in 0..200 {
                if self.is_goal(id) {
                    break;
                }
                let aid = *self
                    .actions
                    .get(id)
                    .and_then(|a| ai.and_then(|i| a.get(i)))
                    .ok_or_else(|| invalid("The selected route has no continuation"))?;
                let a = self.core_actions[aid].clone();
                if a.splice {
                    id = sample_map(&self.dists[a.dist].to, rng)?;
                    splicers += 1.
                } else {
                    let k = self.keep_set(aid, self.value[id])?;
                    if k.q <= 0. {
                        return Err(invalid("The selected route cannot make progress"));
                    }
                    let n = if k.q >= 1. {
                        1.
                    } else {
                        ((-checked_rng(rng)?).ln_1p() / (-k.q).ln_1p())
                            .ceil()
                            .max(1.)
                    };
                    if !n.is_finite()
                        || n > 9_007_199_254_740_991.
                        || (done as f64 + n) > 9_007_199_254_740_991.
                    {
                        return Err(invalid(
                            "The requested roll is too rare to simulate at this precision",
                        ));
                    }
                    let n = n as u64;
                    kuva += cycle_cost_total(done, n, a.locked);
                    done += n;
                    cycles += n as f64;
                    id = k.keep[sample_cumulative(&k.cumulative, checked_rng(rng)?)].0;
                }
                ai = self.best[id];
            }
            if !self.is_goal(id) {
                return Err(invalid(
                    "The simulated policy did not finish within the transition limit",
                ));
            }
            output.kuva.push(kuva);
            output.cycles.push(cycles);
            output.splicers.push(splicers);
        }
        Ok(output)
    }
}
fn invalid(s: &str) -> SolveError {
    SolveError::Invalid(s.into())
}
fn unique_floats(mut x: Vec<f64>) -> Vec<f64> {
    x.sort_by(f64::total_cmp);
    x.dedup_by(|a, b| *a == *b);
    x
}
fn split_spl(s: &str) -> (&str, usize) {
    let s = s.strip_prefix('!').unwrap_or(s);
    if let Some((t, n)) = s.rsplit_once(':') {
        (t, n.parse().unwrap_or(0))
    } else {
        (s, 0)
    }
}
fn factorial(n: usize) -> f64 {
    (1..=n).fold(1., |a, b| a * b as f64)
}
fn choose(n: usize, k: usize) -> f64 {
    if k > n {
        return 0.;
    }
    (1..=k).fold(1., |a, i| a * (n - k + i) as f64 / i as f64)
}
fn draws(counts: &[usize], k: usize) -> Vec<(Vec<usize>, f64)> {
    let total: usize = counts.iter().sum();
    if k == 0 {
        return vec![(vec![0; counts.len()], 1.)];
    }
    if total < k {
        return vec![];
    }
    let denom = choose(total, k);
    let mut out = vec![];
    fn rec(
        c: &[usize],
        i: usize,
        left: usize,
        w: f64,
        denom: f64,
        pick: &mut [usize],
        out: &mut Vec<(Vec<usize>, f64)>,
    ) {
        if i == c.len() {
            if left == 0 {
                out.push((pick.to_vec(), w / denom))
            }
            return;
        }
        for n in 0..=left.min(c[i]) {
            pick[i] = n;
            rec(c, i + 1, left - n, w * choose(c[i], n), denom, pick, out)
        }
        pick[i] = 0;
    }
    rec(
        counts,
        0,
        k,
        1.,
        denom,
        &mut vec![0; counts.len()],
        &mut out,
    );
    out
}
fn match_slots(
    m: &[Vec<(usize, bool)>],
    group: usize,
    used: u64,
    high: usize,
    needed: usize,
) -> bool {
    if group == m.len() {
        return high >= needed;
    }
    m[group].iter().any(|&(slot, h)| {
        slot < 64
            && (used & (1 << slot)) == 0
            && match_slots(
                m,
                group + 1,
                used | (1 << slot),
                high + usize::from(h),
                needed,
            )
    })
}
fn lock_w(r: &Route) -> Option<f64> {
    if let Action::Cycle {
        lock: Lock::Pos(t), ..
    } = &r.action
    {
        Some(t.w)
    } else {
        None
    }
}
fn checked_rng<F: FnMut() -> f64>(rng: &mut F) -> Res<f64> {
    let x = rng();
    if x.is_finite() && (0.0..1.0).contains(&x) {
        Ok(x)
    } else {
        Err(invalid("RNG must return a finite value in [0,1)"))
    }
}
fn sample_map<F: FnMut() -> f64>(m: &[(usize, f64)], rng: &mut F) -> Res<usize> {
    let mut u = checked_rng(rng)?;
    let mut last = None;
    for &(id, p) in m {
        last = Some(id);
        u -= p;
        if u <= 0. {
            return Ok(id);
        }
    }
    last.ok_or_else(|| invalid("Cannot sample empty distribution"))
}
fn sample_cumulative(c: &[f64], u: f64) -> usize {
    c.partition_point(|&x| x <= u).min(c.len() - 1)
}
#[derive(Clone, Copy, Debug)]
struct HeapEntry {
    code: i64,
    value: f64,
    serial: usize,
}
impl PartialEq for HeapEntry {
    fn eq(&self, o: &Self) -> bool {
        self.code == o.code && self.value == o.value && self.serial == o.serial
    }
}
impl Eq for HeapEntry {}
impl Ord for HeapEntry {
    fn cmp(&self, o: &Self) -> Ordering {
        o.value
            .total_cmp(&self.value)
            .then_with(|| o.serial.cmp(&self.serial))
    }
}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
fn heap_push(h: &mut BinaryHeap<HeapEntry>, serial: &mut usize, code: i64, value: f64) {
    h.push(HeapEntry {
        code,
        value,
        serial: *serial,
    });
    *serial += 1;
}
#[allow(clippy::too_many_arguments)]
fn reach(
    id: usize,
    v: f64,
    rev: &[Vec<(usize, f64)>],
    num: &mut [f64],
    den: &mut [f64],
    left: &[usize],
    cost: &[f64],
    heap: &mut BinaryHeap<HeapEntry>,
    serial: &mut usize,
) {
    for &(k, p) in &rev[id] {
        num[k] += p * v;
        den[k] += p;
        if left[k] > 0 {
            heap_push(heap, serial, -1 - k as i64, (cost[k] + num[k]) / den[k]);
        }
    }
}
