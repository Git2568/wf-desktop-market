import { invoke } from '@tauri-apps/api/core'

export type AnalysisRuleMode = 'current' | 'legacy'
export type AnalysisAttribute = { slug: string; value: number; positive: boolean }
export type AnalysisRequest = {
  weapon_slug: string
  mod_rank: number
  rule_mode: AnalysisRuleMode
  disposition: number | null
  attributes: AnalysisAttribute[]
  sample_size: number
}
export type StatRange = {
  slug: string
  name: string
  positive: boolean
  value: number
  unit: string
  range_min: number | null
  range_max: number | null
  best_value: number | null
  position_percent: number | null
  status: 'within' | 'at_display_limit' | 'out_of_range' | 'unsupported'
  message: string | null
}
export type StatAnalysisResult = {
  weapon_slug: string
  weapon_name: string
  rule_mode: AnalysisRuleMode
  rule_version: string
  reference_version: string
  disposition: number
  mod_rank: number
  ranges: StatRange[]
  warnings: string[]
}
export type WeeklyGroup = { rerolled: boolean; popularity: number; avg: number; median: number | null; min: number; max: number; stddev: number }
export type WeeklyWeaponSummary = { source: string; source_updated_at: string | null; fetched_at: number; groups: WeeklyGroup[] }
export type ComparisonResult = {
  scope: string
  requested_count: number
  fetched_count: number
  sample_count: number
  quoted_min: number | null
  quoted_max: number | null
  frequencies: Array<{ slug: string; name: string; positive: boolean; count: number; selected: boolean }>
  listings: Array<{ id: string; price: number; positive_matches: number; positive_total: number; negative_match: boolean; exact_match: boolean; attributes: Array<{ slug: string; name: string; positive: boolean }> }>
}
export type ValuationReference = {
  id: string
  price: number
  similarity: number
  adjusted_price: number
  popularity_factor: number
  numeric_factor: number
  numeric_coverage: number
  positive_matches: number
  positive_total: number
  negative_match: boolean
}
export type ValuationHotStat = { slug: string; name: string; count: number; frequency: number }
export type ValuationResult = {
  rule_version: string
  status: 'estimated' | 'unavailable'
  method: 'similar' | 'nearest' | 'none'
  estimate: number | null
  low: number | null
  high: number | null
  confidence: 'low' | 'medium'
  pool_count: number
  comparable_count: number
  hot_stats: ValuationHotStat[]
  popularity_factor: number
  numeric_factor: number
  heat_factor: number
  numeric_coverage: number
  layout_focus: 'damage' | 'extreme' | 'balanced'
  references: ValuationReference[]
  warnings: string[]
  reason: string | null
}
export type MarketAnalysisResult = {
  weapon_slug: string
  sampled_at: number
  from_cache: boolean
  stale: boolean
  weekly: WeeklyWeaponSummary | null
  weekly_error: string | null
  comparison: ComparisonResult | null
  comparison_error: string | null
  valuation?: ValuationResult | null
  valuation_error?: string | null
}
export type AnalysisDataStatus = { reference_version: string; rule_version: string; weekly_source_updated_at: string | null; weekly_fetched_at: number | null; message: string | null }
export type RangeHintRequest = {
  weapon_slug: string
  mod_rank: number
  rule_mode: AnalysisRuleMode
  disposition: number | null
  positive_count: number
  negative_count: number
  attributes: Array<{ key: string; slug: string; positive: boolean }>
}
export type RangeHint = { key: string; slug: string; positive: boolean; unit: string; range_min: number | null; range_max: number | null; status: 'available' | 'unsupported' | 'invalid_layout'; message: string | null }
export type RangeHintsResult = { ranges: RangeHint[] }

export const analyzeStats = (request: AnalysisRequest) => invoke<StatAnalysisResult>('riven_analyze_stats', { request })
export const analyzeMarket = (request: AnalysisRequest) => invoke<MarketAnalysisResult>('riven_analyze_market', { request })
export const fetchAnalysisDataStatus = () => invoke<AnalysisDataStatus>('riven_analysis_data_status')
export const refreshAnalysisData = (force = false) => invoke<AnalysisDataStatus>('riven_refresh_analysis_data', { force })
export const analyzeRangeHints = (request: RangeHintRequest) => invoke<RangeHintsResult>('riven_analysis_range_hints', { request })
