import { invoke, isTauri } from '@tauri-apps/api/core'
export { isTauri }

export type RivenWeapon = {
  id: string
  slug: string
  riven_type: string
  group?: string
  disposition?: number | null
  i18n: Record<string, { name: string }>
}
export type RivenAttribute = {
  id: string
  slug: string
  name: string
  english_name: string
  unit: string
  positive_only: boolean
  negative_only: boolean
  exclusive_to: string[]
  game_ref?: string
  positive_is_negative?: boolean
}
export type MarketCatalog = {
  weapons: RivenWeapon[]
  attributes: RivenAttribute[]
}
export type CatalogDataStatus = {
  collection_version: string | null
  game_version: string | null
  api_version: string | null
  source: string
  language: string
  platform: string
  downloaded_at: number
  last_checked_at: number | null
  weapons_count: number
  attributes_count: number
  storage: 'sqlite' | 'memory'
  last_error: string | null
}
export type MarketFilters = {
  weapon_slug: string
  positive_stats: string[]
  negative_stat: string | null
  sort_order: MarketSortOrder
  polarity: string | null
  mastery_rank_max: number | null
  mod_rank: 'maxed' | null
  re_rolls_min: number | null
  re_rolls_max: number | null
}
export type MarketSortOrder = 'asc' | 'desc' | 'positive_attr_asc' | 'positive_attr_desc'
export type DisplayAttribute = {
  slug: string
  name: string
  value: number
  positive: boolean
  unit: string
}
export type Listing = {
  id: string
  riven_name: string
  price: number | null
  price_kind: string
  seller: string
  seller_status: string
  rerolls: number | null
  mod_rank: number | null
  polarity: string | null
  mastery_rank: number | null
  attributes: DisplayAttribute[]
}
export type MarketSnapshot = {
  weapon: RivenWeapon
  listings: Listing[]
  fetched_at: number
  elapsed_ms: number
  batch_count: number
  source: string
  platform: string
  filters: MarketFilters
}

export const tauriAvailable = isTauri()
export const fetchCatalog = () => invoke<MarketCatalog>('market_catalog')
export const fetchCatalogStatus = () => invoke<CatalogDataStatus>('market_catalog_status')
export const updateCatalog = (force = false) => invoke<{ status: CatalogDataStatus; updated: boolean; catalog: MarketCatalog }>('market_update_catalog', { force })
export const searchMarket = (filters: MarketFilters) => invoke<MarketSnapshot>('market_search', { filters })
