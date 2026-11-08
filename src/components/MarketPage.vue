<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import AppDialog from './AppDialog.vue'
import EmptyState from './EmptyState.vue'
import SearchSelect, { type SearchOption } from './SearchSelect.vue'
import WeaponSelect from './WeaponSelect.vue'
import ListingActions from './ListingActions.vue'
import PolarityIcon from './PolarityIcon.vue'
import { polarities } from '../data/polarities'
import { useCatalogStore } from '../stores/catalog'
import { icons } from '../icons'
import {
  searchMarket,
  tauriAvailable,
  type MarketFilters,
  type MarketSnapshot,
  type RivenAttribute,
} from '../api/market'
import { weaponOptionsFor, type WeaponOption } from '../ui/weaponOptions'

const catalogStore = useCatalogStore()
const catalog = computed(() => catalogStore.catalog ?? { weapons: [], attributes: [] })
const catalogReady = computed(() => catalogStore.hasCatalog)
const catalogError = computed(() => catalogStore.error)
const draft = ref<MarketFilters>({
  weapon_slug: '',
  positive_stats: [],
  negative_stat: null,
  sort_order: 'asc',
  polarity: null,
  mastery_rank_max: null,
  mod_rank: null,
  re_rolls_min: null,
  re_rolls_max: null,
})
const masteryMaxRaw = ref('')
const rerollsMinRaw = ref('')
const rerollsMaxRaw = ref('')
type RangeKey = 'masteryMax' | 'rerollsMin' | 'rerollsMax'
const rangeRefs: Record<RangeKey, typeof masteryMaxRaw> = {
  masteryMax: masteryMaxRaw,
  rerollsMin: rerollsMinRaw,
  rerollsMax: rerollsMaxRaw,
}
const snapshot = ref<MarketSnapshot | null>(null)
const error = ref('')
const advancedError = ref('')
const loading = ref(false)
const advancedOpen = ref(false)
const cooldownUntil = ref(0)
const now = ref(Date.now())
const requestId = ref(0)
const activeMenuId = ref<string | null>(null)
let timer: number | undefined
let mounted = false

const cooldownSeconds = computed(() => Math.max(0, Math.ceil((cooldownUntil.value - now.value) / 1000)))
const weapon = computed(() => catalog.value.weapons.find(item => item.slug === draft.value.weapon_slug))
const weaponOptions = computed<WeaponOption[]>(() => weaponOptionsFor(catalog.value.weapons))
const attributes = computed(() => catalog.value.attributes.filter(item => (
  !weapon.value || !item.exclusive_to.length || item.exclusive_to.includes(weapon.value.riven_type)
)))
const positiveOptions = computed<SearchOption[]>(() => attributes.value
  .filter(item => !item.negative_only && item.slug !== draft.value.negative_stat)
  .map(attributeOption))
const negativeOptions = computed<SearchOption[]>(() => attributes.value
  .filter(item => !item.positive_only && !draft.value.positive_stats.includes(item.slug))
  .map(attributeOption))
const polarityOptions = computed<SearchOption[]>(() => [
  { value: '', label: '不限' },
  ...Object.entries(polarities).map(([value, polarity]) => ({ value, label: polarity.name })),
])
const stale = computed(() => Boolean(snapshot.value && (
  now.value - snapshot.value.fetched_at > 5 * 60 * 1000 || error.value
)))
const dirty = computed(() => {
  if (!snapshot.value) return false
  const rangeDirty = [
    [masteryMaxRaw.value.trim(), snapshot.value.filters.mastery_rank_max],
    [rerollsMinRaw.value.trim(), snapshot.value.filters.re_rolls_min],
    [rerollsMaxRaw.value.trim(), snapshot.value.filters.re_rolls_max],
  ].some(([raw, value]) => raw !== (value == null ? '' : String(value)))
  const current = normalizeFilters(draft.value)
  const changed = (Object.keys(current) as Array<keyof MarketFilters>)
    .some(key => JSON.stringify(current[key]) !== JSON.stringify(snapshot.value!.filters[key]))
  return rangeDirty || changed
})

function attributeOption(attribute: RivenAttribute): SearchOption {
  return { value: attribute.slug, label: attribute.name, secondary: attribute.english_name }
}

function updatePolarity(value: string | string[] | null) {
  draft.value.polarity = typeof value === 'string' && value ? value : null
}

function errorText(cause: unknown) {
  return typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : '无法加载市场数据。'
}

function setCooldownFromError(message: string) {
  if (!message.includes('冷却')) return
  const match = message.match(/(\d+)\s*秒/)
  if (match) cooldownUntil.value = Math.max(cooldownUntil.value, Date.now() + Number(match[1]) * 1000)
}

function parseRange(value: string, minimum: number, maximum: number, label: string): number | null {
  const trimmed = value.trim()
  if (!trimmed) return null
  if (!/^[0-9]+$/.test(trimmed)) throw new Error(`${label}必须是 ${minimum}–${maximum} 的非负整数，或留空。`)
  const parsed = Number(trimmed)
  if (!Number.isSafeInteger(parsed) || parsed < minimum || parsed > maximum) {
    throw new Error(`${label}必须是 ${minimum}–${maximum} 的非负整数，或留空。`)
  }
  return parsed
}

function setRangeRaw(key: RangeKey, event: Event, maxDigits: number) {
  const input = event.target as HTMLInputElement
  const value = input.value.replace(/[^0-9]/g, '').slice(0, maxDigits)
  input.value = value
  rangeRefs[key].value = value
}

function validateRanges() {
  const masteryMax = parseRange(masteryMaxRaw.value, 7, 16, '需要段位等级')
  const rerollsMin = parseRange(rerollsMinRaw.value, 0, 99_999_999, '洗卡次数')
  const rerollsMax = parseRange(rerollsMaxRaw.value, 0, 99_999_999, '洗卡次数')
  if (rerollsMin != null && rerollsMax != null && rerollsMin > rerollsMax) throw new Error('洗卡次数最小值不能大于最大值。')
  return { masteryMax, rerollsMin, rerollsMax }
}

function normalizeFilters(filters: MarketFilters): MarketFilters {
  return { ...filters, negative_stat: filters.negative_stat || null, polarity: filters.polarity || null, positive_stats: [...filters.positive_stats] }
}

function buildFilters(): MarketFilters {
  if (!draft.value.weapon_slug) throw new Error('请选择一个真实武器。')
  if (draft.value.positive_stats.length > 3) throw new Error('正面特性最多选择 3 个。')
  if ((draft.value.sort_order === 'positive_attr_asc' || draft.value.sort_order === 'positive_attr_desc') && !draft.value.positive_stats.length) {
    throw new Error('请至少选择一个正面词条后按词条排序。')
  }
  const ranges = validateRanges()
  return normalizeFilters({
    ...draft.value,
    negative_stat: draft.value.negative_stat || null,
    polarity: draft.value.polarity || null,
    mastery_rank_max: ranges.masteryMax,
    mod_rank: draft.value.mod_rank,
    re_rolls_min: ranges.rerollsMin,
    re_rolls_max: ranges.rerollsMax,
  })
}

async function query() {
  let filters: MarketFilters
  try {
    filters = buildFilters()
    advancedError.value = ''
  } catch (cause) {
    const message = errorText(cause)
    error.value = message
    advancedError.value = message
    return false
  }
  if (!tauriAvailable || !catalogReady.value || loading.value || catalogStore.loading || cooldownSeconds.value > 0) return false
  const id = ++requestId.value
  draft.value = { ...draft.value, ...filters }
  loading.value = true
  error.value = ''
  advancedError.value = ''
  cooldownUntil.value = Math.max(cooldownUntil.value, Date.now() + 6000)
  try {
    const result = await searchMarket(filters)
    if (mounted && id === requestId.value) snapshot.value = result
    return true
  } catch (cause) {
    if (mounted && id === requestId.value) {
      error.value = snapshot.value ? `刷新失败，数据已过期，保留上次结果：${errorText(cause)}` : errorText(cause)
      setCooldownFromError(error.value)
      advancedError.value = error.value
    }
    return false
  } finally {
    if (mounted && id === requestId.value) loading.value = false
  }
}

async function queryAdvanced() {
  if (await query()) advancedOpen.value = false
}

async function loadCatalog() {
  if (!tauriAvailable) return false
  return catalogStore.load()
}

async function retryMarket() {
  if (loading.value || cooldownSeconds.value > 0) return
  if (!catalogReady.value) await loadCatalog()
  if (catalogReady.value) await query()
}

function resetInvalidStats() {
  const positive = draft.value.positive_stats.filter(slug => positiveOptions.value.some(option => option.value === slug))
  if (positive.length !== draft.value.positive_stats.length) draft.value.positive_stats = positive
  if (draft.value.negative_stat && !negativeOptions.value.some(option => option.value === draft.value.negative_stat)) draft.value.negative_stat = null
}

function sellerStatus(status: string) {
  return status === 'online' ? '网页在线' : status === 'offline' ? '网页离线' : status === 'ingame' ? '游戏在线' : '状态未知'
}

function listingMeta(listing: MarketSnapshot['listings'][number]) {
  const polarity = listing.polarity ? polarities[listing.polarity as keyof typeof polarities]?.name ?? listing.polarity : '极性未知'
  return `${polarity}·段位${listing.mastery_rank ?? '未知'}·强化${listing.mod_rank ?? '未知'}·洗卡${listing.rerolls ?? '未知'}`
}

function attributeText(attribute: MarketSnapshot['listings'][number]['attributes'][number]) {
  return `${attribute.name} ${attribute.value}${attribute.unit}`
}

watch(() => draft.value.weapon_slug, resetInvalidStats)
watch(() => draft.value.positive_stats, resetInvalidStats, { deep: true })
watch([snapshot, advancedOpen], () => { activeMenuId.value = null })
onMounted(() => {
  mounted = true
  timer = window.setInterval(() => { now.value = Date.now() }, 1000)
  loadCatalog()
})
onUnmounted(() => {
  mounted = false
  requestId.value++
  if (timer) window.clearInterval(timer)
})
</script>

<template>
  <section class="market-page">
    <div class="market-heading">
      <h1>紫卡购买</h1>
      <div class="market-heading-actions">
        <button id="market-advanced-trigger" class="secondary-button" type="button" @click="advancedOpen = true">高级筛选</button>
        <button class="primary-button" type="button" :disabled="loading || catalogStore.loading || cooldownSeconds > 0 || !draft.weapon_slug" @click="query">
          {{ catalogStore.loading ? '加载目录…' : loading ? '查询中…' : cooldownSeconds ? `${cooldownSeconds}s 后可查询` : '查询' }}
        </button>
      </div>
    </div>

    <div class="market-filters simple-filters">
      <div class="filter-field filter-wide"><label for="simple-weapon">名称</label><WeaponSelect input-id="simple-weapon" label="名称" v-model="draft.weapon_slug" :options="weaponOptions" placeholder="搜索并选择中英文武器" /></div>
      <label class="filter-field"><span>正面特性（最多 3 个）</span><SearchSelect input-id="simple-positive" label="正面特性" v-model="draft.positive_stats" :options="positiveOptions" placeholder="选择正面特性" multiple :max="3" /></label>
      <label class="filter-field"><span>负面特性（最多 1 个）</span><SearchSelect input-id="simple-negative" label="负面特性" v-model="draft.negative_stat" :options="negativeOptions" placeholder="选择负面特性" nullable /></label>
      <label class="filter-field sort-filter"><span>排序</span><select v-model="draft.sort_order"><option value="asc">价格升序</option><option value="desc">价格降序</option><option value="positive_attr_asc">正面词条升序</option><option value="positive_attr_desc">正面词条降序</option></select></label>
    </div>

    <p v-if="dirty" class="market-dirty">筛选已修改，点击查询更新。</p>
    <p v-if="error && snapshot" class="market-stale">{{ error }}</p>
    <p v-if="snapshot" class="market-meta">来源：{{ snapshot.source }} · 平台：{{ snapshot.platform }} · {{ stale ? '数据已过期' : '最后成功' }}：{{ new Date(snapshot.fetched_at).toLocaleString() }} · 本批 {{ snapshot.batch_count }} 条挂单{{ snapshot.filters.sort_order === 'positive_attr_asc' || snapshot.filters.sort_order === 'positive_attr_desc' ? ' · 词条排序限当前批次' : '' }}</p>
    <div v-if="!tauriAvailable" class="market-notice">请在 WM Market 桌面应用中加载真实行情。</div>
    <div v-else-if="(error || catalogError) && !snapshot" class="market-notice is-error">{{ error || catalogError }} <button type="button" :disabled="loading || catalogStore.loading || cooldownSeconds > 0" @click="retryMarket">重试</button></div>
    <div v-else class="market-table" aria-label="市场列表">
      <div class="table-row table-header">
        <span>紫卡</span><span>词条</span><span>价格</span><span>用户状态</span><span>操作</span>
      </div>
      <div v-if="!snapshot?.listings.length" class="table-empty"><EmptyState :icon="icons.market" :title="snapshot ? '暂无符合条件的挂单' : '设置条件后查询'" :description="snapshot ? '可以调整筛选条件后重新查询。' : '选择武器并点击查询获取真实市场数据。'" compact /></div>
      <div v-for="listing in snapshot?.listings" :key="listing.id" class="table-row table-data">
        <div>
          <strong>{{ listing.riven_name }}</strong>
          <small :title="listingMeta(listing)"><PolarityIcon :polarity="listing.polarity" />·段位{{ listing.mastery_rank ?? '未知' }}·强化{{ listing.mod_rank ?? '未知' }}·洗卡{{ listing.rerolls ?? '未知' }}</small>
        </div>
        <div class="attribute-list">
          <span v-for="attribute in listing.attributes" :key="attribute.slug" :class="attribute.positive ? 'positive' : 'negative'" :aria-label="`${attribute.positive ? '正面' : '负面'}词条：${attributeText(attribute)}`">{{ attributeText(attribute) }}</span>
        </div>
        <div><strong v-if="listing.price != null">{{ listing.price }}p</strong><small>{{ listing.price_kind }}</small></div>
        <div><strong>{{ listing.seller }}</strong><small>{{ sellerStatus(listing.seller_status) }}</small></div>
        <ListingActions :listing-id="listing.id" :riven-name="listing.riven_name" :open="activeMenuId === listing.id" @toggle="activeMenuId = activeMenuId === listing.id ? null : listing.id" @close="activeMenuId = null" />
      </div>
    </div>

    <AppDialog v-model="advancedOpen" title="高级筛选" labelledby="market-advanced-title" restore-selector="#market-advanced-trigger" wide>
      <div class="market-advanced-content">
        <p v-if="advancedError" class="market-stale">{{ advancedError }}</p>
        <div class="market-filters advanced-filters">
          <div class="filter-field"><label for="advanced-weapon">名称</label><WeaponSelect input-id="advanced-weapon" label="高级筛选武器" v-model="draft.weapon_slug" :options="weaponOptions" placeholder="搜索并选择武器" /></div>
          <label class="filter-field"><span>极性</span><SearchSelect input-id="advanced-polarity" label="极性" :model-value="draft.polarity ?? ''" :options="polarityOptions" placeholder="不限" nullable @update:model-value="updatePolarity">
            <template #selected="{ value, option }"><span class="polarity-select-item"><PolarityIcon v-if="value" :polarity="value" aria-hidden="true" />{{ option?.label ?? '不限' }}</span></template>
            <template #option="{ option }"><span class="polarity-select-item"><PolarityIcon v-if="option.value" :polarity="option.value" aria-hidden="true" />{{ option.label }}</span></template>
          </SearchSelect></label>
          <label class="filter-field full-row"><span>正面特性</span><SearchSelect input-id="advanced-positive" label="高级正面特性" v-model="draft.positive_stats" :options="positiveOptions" placeholder="选择正面特性" multiple :max="3" /></label>
          <label class="filter-field full-row"><span>负面特性</span><SearchSelect input-id="advanced-negative" label="高级负面特性" v-model="draft.negative_stat" :options="negativeOptions" placeholder="选择负面特性" nullable /></label>
          <label class="filter-field"><span>强化等级</span><select v-model="draft.mod_rank"><option :value="null">不限</option><option value="maxed">满级（8级）</option></select></label>
          <div class="filter-field" role="group" aria-labelledby="mastery-range-label">
            <span id="mastery-range-label">需要段位等级</span>
            <input :value="masteryMaxRaw" type="text" inputmode="numeric" maxlength="2" pattern="[0-9]*" min="7" max="16" placeholder="最高/不限" aria-label="需要段位等级最高值" @input="setRangeRaw('masteryMax', $event, 2)" />
          </div>
          <div class="filter-field full-row" role="group" aria-labelledby="rerolls-range-label">
            <span id="rerolls-range-label">洗卡次数</span>
            <div class="range-fields">
              <input :value="rerollsMinRaw" type="text" inputmode="numeric" maxlength="8" pattern="[0-9]*" min="0" placeholder="最小值" aria-label="洗卡次数最小值" @input="setRangeRaw('rerollsMin', $event, 8)" />
              <span class="range-separator" aria-hidden="true">—</span>
              <input :value="rerollsMaxRaw" type="text" inputmode="numeric" maxlength="8" pattern="[0-9]*" min="0" placeholder="最大值" aria-label="洗卡次数最大值" @input="setRangeRaw('rerollsMax', $event, 8)" />
            </div>
          </div>
          <label class="filter-field full-row"><span>排序</span><select v-model="draft.sort_order"><option value="asc">价格升序</option><option value="desc">价格降序</option><option value="positive_attr_asc">正面词条升序</option><option value="positive_attr_desc">正面词条降序</option></select></label>
        </div>
        <button class="primary-button dialog-query-button" type="button" :disabled="loading || cooldownSeconds > 0" @click="queryAdvanced">查询并关闭</button>
      </div>
    </AppDialog>
  </section>
</template>
