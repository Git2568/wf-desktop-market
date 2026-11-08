import { invoke } from '@tauri-apps/api/core'

export type RerollRuleMode = 'current' | 'legacy'
export type RerollAttribute = { slug: string; value: number; positive: boolean; spliced: boolean }
export type RerollTargetGroup = { slugs: string[]; min_w: number; min_values: Record<string, number> | null }
export type RerollTarget = { groups: RerollTargetGroup[]; sets: number[][] | null; layouts: Array<{ pos: 2 | 3; neg: 0 | 1 }>; allowed_negatives: string[] | null; at_least: { k: number; w: number } | null }
export type RerollPlanRequest = {
  weapon_slug: string
  mod_rank: number
  rule_mode: RerollRuleMode
  disposition: number | null
  current_attributes: RerollAttribute[]
  rerolls: number
  kuva_budget: number | null
  objective: 'kuva' | 'cycles'
  target: RerollTarget
  new_attributes: RerollAttribute[] | null
}

export type RerollQuantiles = { mean:number; p10:number; p25:number; p50:number; p75:number; p90:number }
export type RerollOutcome = { prob:number; needs:Array<{stats:string[];min_w:number;neg?:boolean}>; neg_allowed:boolean|null; layout:{pos:2|3;neg:0|1} }
export type RerollRoute = { action: { kind:'cycle'; lock:'none'|'neg'|{tag:string;stats:string[];w:number}; drop?:boolean } | { kind:'splice'; result:string; a:string; b:string }; step_chance:number; step_cycles:number; step_cycles_p90:number; expected:number; optimal:boolean; outcomes:RerollOutcome[]; worse:RerollOutcome[] }
export type RerollRouteStats = { route:RerollRoute; kuva:RerollQuantiles; cycles:RerollQuantiles; splicers:number; within_budget:number|null; next_cycle_cost:number }
export type RerollPlanResult = { done:boolean; reachable:boolean; routes:RerollRouteStats[]; states:number; ms:number; compare_values:Array<number|null>; compare_neg:Array<string|null>; current_value:number|null; objective:'kuva'|'cycles'; warnings:string[]; version:string; samples:number; ok?:boolean; error?:string }
export type RerollOptionsRequest = { weapon_slug:string; current_attributes:RerollAttribute[] }
export type RerollOptionsResult = { weapon_slug:string; game_ref:string; positive:string[]; negative:string[]; spliced:string[]; version:string }

export const planReroll = (request: RerollPlanRequest) => invoke<RerollPlanResult>('riven_reroll_plan', { request })
export const fetchRerollOptions = (request: RerollOptionsRequest) => invoke<RerollOptionsResult>('riven_reroll_options', { request })
