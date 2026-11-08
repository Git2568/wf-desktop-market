<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { HugeiconsIcon } from '@hugeicons/vue'
import SearchSelect, { type SearchOption } from './SearchSelect.vue'
import WeaponSelect from './WeaponSelect.vue'
import RivenCardPreview from './RivenCardPreview.vue'
import { useCatalogStore } from '../stores/catalog'
import { icons } from '../icons'
import { tauriAvailable, type RivenAttribute, type RivenWeapon } from '../api/market'
import { analyzeMarket, analyzeRangeHints, analyzeStats, type AnalysisRequest, type MarketAnalysisResult, type RangeHint, type RangeHintRequest, type StatAnalysisResult, type ValuationResult } from '../api/analysis'
import { weaponOptionsFor, type WeaponOption } from '../ui/weaponOptions'
import { attributePresentation, sortAttributePresentation } from '../ui/attributePresentation'

type StatRow = { id: number; slug: string; value: string }
type StatKind = 'positive' | 'negative'
const catalogStore = useCatalogStore()
const weaponSlug = ref('')
const positives = ref<StatRow[]>([])
const negative = ref<StatRow[]>([])
const modRank = ref('8')
const ruleMode = ref<'current' | 'legacy'>('current')
const legacyDisposition = ref('')
const sampleSize = ref('20')
const showPreview = ref(true)
const statsResult = ref<StatAnalysisResult | null>(null)
const marketResult = ref<MarketAnalysisResult | null>(null)
const statsLoading = ref(false)
const marketLoading = ref(false)
const statsError = ref('')
const marketError = ref('')
const generation = ref(0)
const alive = ref(true)
let nextRowId = 1
const rangeHints = ref<Record<number, RangeHint>>({})
const rangeContextGeneration = ref(0)
const rangeRowGenerations = new Map<number, number>()
const weapons = computed<RivenWeapon[]>(() => catalogStore.catalog?.weapons ?? [])
const attributes = computed<RivenAttribute[]>(() => catalogStore.catalog?.attributes ?? [])
const selectedWeapon = computed(() => weapons.value.find(item => item.slug === weaponSlug.value) ?? null)
const weaponOptions = computed<WeaponOption[]>(() => weaponOptionsFor(weapons.value))
const legacyDefaultDisposition = computed(() => selectedWeapon.value?.disposition ?? null)

function rowFor(kind: StatKind, index: number) { return kind === 'positive' ? positives.value[index] : negative.value[index] }
function attributeAllowed(attribute: RivenAttribute, kind: StatKind) {
  if (kind === 'positive' && attribute.negative_only) return false
  if (kind === 'negative' && attribute.positive_only) return false
  if (kind === 'negative' && attributePresentation(attribute).group === '熔接') return false
  const weapon = selectedWeapon.value
  if (weapon && attribute.exclusive_to.length) {
    const type = weapon.group === 'archgun' ? 'archgun' : weapon.riven_type
    if (!attribute.exclusive_to.includes(type)) return false
  }
  return true
}
function selectedSlugs(kind: StatKind, index: number) {
  return [...positives.value, ...negative.value]
    .filter((_, i) => !(kind === 'positive' && i === index))
    .filter((_, i) => !(kind === 'negative' && i === positives.value.length + index))
    .map(row => row.slug).filter(Boolean)
}
function attributeOptions(kind: StatKind, index: number): SearchOption[] {
  const used = new Set(selectedSlugs(kind, index))
  const current = rowFor(kind, index)?.slug
  return sortAttributePresentation(attributes.value
    .filter(attribute => attributeAllowed(attribute, kind) && (!used.has(attribute.slug) || attribute.slug === current))
  ).map(attribute => ({ value: attribute.slug, label: attribute.name, secondary: attribute.english_name, group: attributePresentation(attribute).group }))
}
function clearResults() { statsResult.value = null; marketResult.value = null; statsError.value = ''; marketError.value = '' }
function clearRangeHints() { rangeContextGeneration.value++; rangeHints.value = {}; rangeRowGenerations.clear() }
function addRow(kind: StatKind) { if (kind === 'positive' && positives.value.length < 3) positives.value.push({ id: nextRowId++, slug: '', value: '' }); if (kind === 'negative' && negative.value.length < 1) negative.value.push({ id: nextRowId++, slug: '', value: '' }); clearRangeHints(); clearResults(); void refreshRangeHints() }
function removeRow(kind: StatKind, index: number) { const rows = kind === 'positive' ? positives.value : negative.value; const row = rows[index]; if (row) rangeRowGenerations.delete(row.id); rows.splice(index, 1); clearRangeHints(); clearResults(); void refreshRangeHints() }
function updateRowSlug(kind: StatKind, index: number, value: string | string[] | null) { const row = rowFor(kind, index); if (!row) return; row.slug = typeof value === 'string' ? value : ''; rangeRowGenerations.set(row.id, (rangeRowGenerations.get(row.id) ?? 0) + 1); delete rangeHints.value[row.id]; clearResults(); void refreshRangeHints(row) }
function isValidNumber(value: string) { return Boolean(value.trim() && /^[+-]?(?:\d+(?:\.\d*)?|\.\d+)$/.test(value.trim()) && Number.isFinite(Number(value))) }
function attributeFor(slug: string) { return attributes.value.find(item => item.slug === slug) }
function unitText(unit: string) { return unit === 'percent' ? '%' : unit === 'multiply' ? '倍' : unit === 'seconds' ? '秒' : unit === 'unknown' ? '' : unit }
function displayValue(row: StatRow) { return !row.slug || !isValidNumber(row.value) ? '—' : `${row.value}${unitText(attributeFor(row.slug)?.unit ?? '')}` }
function rowLabel(kind: StatKind, index: number) { return `${kind === 'positive' ? '正面' : '负面'}词条${index + 1}` }
function rowIsComplete(row: StatRow) { return Boolean(row.slug) && isValidNumber(row.value) }
const validRows = computed(() => positives.value.length >= 2 && positives.value.length <= 3 && positives.value.every(rowIsComplete) && negative.value.every(rowIsComplete) && new Set([...positives.value, ...negative.value].map(row => row.slug).filter(Boolean)).size === positives.value.length + negative.value.length && positives.value.filter(row => attributePresentation(attributeFor(row.slug) ?? { slug: row.slug, name: row.slug }).group === '熔接').length <= 1 && Boolean(selectedWeapon.value))
const validParameters = computed(() => /^(?:[0-8])$/.test(modRank.value.trim()) && /^(?:[1-9]|[1-4]\d|50)$/.test(sampleSize.value.trim()) && (ruleMode.value === 'current' || (isValidNumber(legacyDisposition.value) && Number(legacyDisposition.value) >= 0.5 && Number(legacyDisposition.value) <= 1.55)))
const canAnalyze = computed(() => validRows.value && validParameters.value)
function sanitizeRows() {
  const used = new Set<string>()
  const sanitize = (kind: StatKind, rows: StatRow[]) => rows.map(row => {
    const valid = Boolean(attributeFor(row.slug) && attributeAllowed(attributeFor(row.slug)!, kind) && !used.has(row.slug))
    if (valid) used.add(row.slug)
    return valid ? row : { ...row, slug: '' }
  })
  positives.value = sanitize('positive', positives.value)
  negative.value = sanitize('negative', negative.value)
}
function errorMessage(error: unknown, fallback: string) { return typeof error === 'string' ? error : error instanceof Error ? error.message : fallback }
function requestForAnalysis(): AnalysisRequest {
  return {
    weapon_slug: weaponSlug.value, mod_rank: Number(modRank.value), rule_mode: ruleMode.value,
    disposition: ruleMode.value === 'legacy' ? Number(legacyDisposition.value || legacyDefaultDisposition.value) : null,
    attributes: [...positives.value.map(row => ({ slug: row.slug, value: Number(row.value), positive: true })), ...negative.value.map(row => ({ slug: row.slug, value: Number(row.value), positive: false }))],
    sample_size: Number(sampleSize.value),
  }
}
function rangeRequest(attributes: RangeHintRequest['attributes'] = []): RangeHintRequest {
  return { weapon_slug: weaponSlug.value, mod_rank: Number(modRank.value), rule_mode: ruleMode.value, disposition: ruleMode.value === 'legacy' ? Number(legacyDisposition.value || legacyDefaultDisposition.value) : null, positive_count: positives.value.length, negative_count: negative.value.length, attributes }
}
async function refreshRangeHints(singleRow?: StatRow) {
  if (!alive.value || !tauriAvailable || !weaponSlug.value || !/^[0-8]$/.test(modRank.value.trim()) || (ruleMode.value === 'legacy' && (!isValidNumber(legacyDisposition.value) || Number(legacyDisposition.value) < 0.5 || Number(legacyDisposition.value) > 1.55))) return
  const context = rangeContextGeneration.value
  if (singleRow) {
    const rowGeneration = (rangeRowGenerations.get(singleRow.id) ?? 0) + 1
    rangeRowGenerations.set(singleRow.id, rowGeneration)
    if (!singleRow.slug) return
    const requestedSlug = singleRow.slug
    const result = await analyzeRangeHints(rangeRequest([{ key: String(singleRow.id), slug: requestedSlug, positive: positives.value.includes(singleRow) }])).catch(() => null)
    const currentRow = [...positives.value, ...negative.value].find(row => row.id === singleRow.id)
    if (!result || !alive.value || context !== rangeContextGeneration.value || rangeRowGenerations.get(singleRow.id) !== rowGeneration || !currentRow || currentRow.slug !== requestedSlug) return
    const hint = result.ranges.find(item => item.key === String(singleRow.id))
    if (hint && hint.slug === requestedSlug) rangeHints.value[singleRow.id] = hint
    return
  }
  rangeHints.value = {}
  const rows = [...positives.value, ...negative.value]
  const snapshot = rows.filter(row => row.slug).map(row => {
    const generation = (rangeRowGenerations.get(row.id) ?? 0) + 1
    rangeRowGenerations.set(row.id, generation)
    return { id: row.id, slug: row.slug, positive: positives.value.includes(row), generation }
  })
  if (!snapshot.length) return
  const result = await analyzeRangeHints(rangeRequest(snapshot.map(row => ({ key: String(row.id), slug: row.slug, positive: row.positive })))).catch(() => null)
  if (!result || !alive.value || context !== rangeContextGeneration.value) return
  for (const hint of result.ranges) {
    const id = Number(hint.key)
    const requested = snapshot.find(row => row.id === id)
    const current = rows.find(row => row.id === id)
    if (requested && current && current.slug === requested.slug && rangeRowGenerations.get(id) === requested.generation && hint.slug === requested.slug) rangeHints.value[id] = hint
  }
}
function rangeHintFor(row: StatRow) { return rangeHints.value[row.id] }
function rangePlaceholder(row: StatRow) { const hint = rangeHintFor(row); if (!hint) return '数值'; if (hint.status === 'invalid_layout') return '需2–3个正面'; if (hint.status === 'unsupported') return '暂无范围'; if (!finiteNumber(hint.range_min) || !finiteNumber(hint.range_max) || (hint.range_min === 0 && hint.range_max === 0)) return '数值'; const digits = hint.unit === 'multiply' ? 2 : 1; return `${formatNumber(hint.range_min, digits)}–${formatNumber(hint.range_max, digits)}` }
function rangeDescription(row: StatRow) { const hint = rangeHintFor(row); return hint?.status === 'available' ? `理论范围 ${rangePlaceholder(row)}${unitText(hint.unit)}` : hint?.message || (hint?.status === 'invalid_layout' ? '理论范围需2–3个正面词条' : hint?.status === 'unsupported' ? '理论范围暂无范围' : '理论范围暂不可用') }
function startAnalysis() {
  if (!canAnalyze.value) return
  if (!tauriAvailable) { statsError.value = '请在 WM Market 桌面应用中进行分析。'; marketError.value = statsError.value; return }
  const request = requestForAnalysis(); const token = ++generation.value
  statsResult.value = null; marketResult.value = null; statsError.value = ''; marketError.value = ''; statsLoading.value = true; marketLoading.value = true
  void analyzeStats(request).then(result => { if (alive.value && token === generation.value) statsResult.value = result }).catch(error => { if (alive.value && token === generation.value) statsError.value = errorMessage(error, '词条范围分析失败。') }).finally(() => { if (alive.value && token === generation.value) statsLoading.value = false })
  void analyzeMarket(request).then(result => { if (alive.value && token === generation.value) marketResult.value = result }).catch(error => { if (alive.value && token === generation.value) marketError.value = errorMessage(error, '市场参考加载失败。') }).finally(() => { if (alive.value && token === generation.value) marketLoading.value = false })
}
function clearOnInput() { generation.value++; statsLoading.value = false; marketLoading.value = false; clearResults() }
watch(weaponSlug, () => { sanitizeRows(); clearOnInput(); clearRangeHints(); void refreshRangeHints() })
watch(() => catalogStore.catalog, () => { sanitizeRows(); clearRangeHints(); void refreshRangeHints() }, { deep: true })
watch([modRank, ruleMode, legacyDisposition], () => { clearOnInput(); clearRangeHints(); void refreshRangeHints() })
watch(sampleSize, clearOnInput)
watch([() => positives.value.map(row => row.value), () => negative.value.map(row => row.value)], clearOnInput, { deep: true })
watch([ruleMode, selectedWeapon], ([mode, weapon]) => {
  if (mode === 'legacy') legacyDisposition.value = weapon?.disposition != null ? String(weapon.disposition) : ''
}, { deep: true })
onMounted(() => { void catalogStore.load() })
onUnmounted(() => { alive.value = false; generation.value++; clearRangeHints() })
function formatNumber(value: number, digits = 1) { return Number.isFinite(value) ? value.toFixed(digits).replace(/\.0+$/, '').replace(/(\.\d*?)0+$/, '$1') : '—' }
function rangeText(range: StatAnalysisResult['ranges'][number]) { return range.range_min == null || range.range_max == null ? '—' : `${formatNumber(range.range_min, range.unit === 'multiply' ? 2 : 1)}–${formatNumber(range.range_max, range.unit === 'multiply' ? 2 : 1)}${unitText(range.unit)}` }
function valueText(value: number, unit: string) { return `${formatNumber(value, unit === 'multiply' ? 2 : 1)}${unitText(unit)}` }
function statusText(status: string) { return status === 'at_display_limit' ? '达到显示上限' : status === 'out_of_range' ? '超出计算范围' : status === 'unsupported' ? '暂不支持' : '范围内' }
function progressStyle(value: number | null) { return value == null ? undefined : { width: `${Math.max(0, Math.min(100, value))}%` } }
function analysisSummary(result: StatAnalysisResult) { return `${result.rule_mode === 'current' ? '现行规则' : '历史倍率'} · D${formatNumber(result.disposition, 2)} · 强化等级 ${result.mod_rank}` }
function finiteNumber(value: unknown): value is number { return typeof value === 'number' && Number.isFinite(value) }
function priceText(value: unknown) { return finiteNumber(value) ? `${formatNumber(value, 1)}p` : '暂无' }
function percentText(value: unknown) { return finiteNumber(value) ? `${formatNumber(value * 100, 1)}%` : '暂无' }
function adjustmentText(value: unknown) { return finiteNumber(value) ? `${value > 0 ? '+' : ''}${formatNumber(value * 100, 1)}%` : '暂无' }
function hasPriceRange(result: ValuationResult) { return finiteNumber(result.low) && finiteNumber(result.high) && !(result.low === 0 && result.high === 0) }
function valuationMethod(method: ValuationResult['method']) { return method === 'similar' ? '相近词条参考' : method === 'nearest' ? '最接近词条推算' : '暂无可用方法' }
function valuationProfile(focus: ValuationResult['layout_focus']) { return focus === 'extreme' ? '2正1负偏手感/热门极限' : focus === 'damage' ? '3正1负偏伤害组合' : '当前组合按平衡规则参考' }
function finiteHotStats(result: ValuationResult) { return result.hot_stats.filter(item => finiteNumber(item.count) && finiteNumber(item.frequency) && item.count >= 0).slice(0, 6) }
function finiteReferences(result: ValuationResult) { return result.references.filter(item => finiteNumber(item.price) && finiteNumber(item.similarity) && finiteNumber(item.adjusted_price)) }
function uniqueWarnings(result: ValuationResult) { return [...new Set(result.warnings.filter(warning => warning.trim()))] }
</script>

<template>
  <section class="analysis-page" aria-labelledby="analysis-title">
    <header class="analysis-heading"><h1 id="analysis-title">紫卡词条分析</h1></header>
    <div class="analysis-layout">
      <form class="analysis-form" @submit.prevent="startAnalysis">
        <div id="riven-card-preview" class="analysis-preview-wrapper" v-show="showPreview"><RivenCardPreview :title="selectedWeapon?.i18n['zh-hans']?.name ?? selectedWeapon?.i18n.en?.name ?? '未选择武器'" :subtitle="selectedWeapon ? (selectedWeapon.i18n.en?.name ?? selectedWeapon.slug) : undefined" :positives="positives.map(row => `${attributeFor(row.slug)?.name ?? '待选择'} ${displayValue(row)}`)" :negatives="negative.map(row => `${attributeFor(row.slug)?.name ?? '待选择'} ${displayValue(row)}`)" /></div>
        <div class="analysis-upload-controls"><button class="secondary-button analysis-upload" type="button" disabled aria-label="上传图片，暂未开放"><HugeiconsIcon :icon="icons.upload" :size="17" color="currentColor" :stroke-width="1.7" />上传图片</button><button class="analysis-preview-toggle" type="button" :aria-label="showPreview ? '隐藏紫卡预览' : '显示紫卡预览'" :title="showPreview ? '隐藏紫卡预览' : '显示紫卡预览'" :aria-controls="'riven-card-preview'" :aria-pressed="showPreview" @click="showPreview = !showPreview"><HugeiconsIcon :icon="showPreview ? icons.hidePreview : icons.preview" :size="17" color="currentColor" :stroke-width="1.7" /><span class="sr-only">{{ showPreview ? '当前显示紫卡预览' : '当前隐藏紫卡预览' }}</span></button></div>
        <label class="analysis-label" for="analysis-weapon">武器名称</label><WeaponSelect v-model="weaponSlug" input-id="analysis-weapon" label="武器名称" placeholder="搜索武器" :options="weaponOptions" />
        <div class="analysis-section-title"><span>正面词条</span><button class="analysis-add" type="button" :disabled="positives.length >= 3" aria-label="添加正面词条" @click="addRow('positive')"><HugeiconsIcon :icon="icons.add" :size="16" color="currentColor" :stroke-width="1.8" /></button></div>
        <div class="analysis-rows"><div v-for="(row, index) in positives" :key="row.id" class="analysis-row"><SearchSelect :model-value="row.slug" :input-id="`analysis-positive-${row.id}`" :label="rowLabel('positive', index)" placeholder="选择词条" :options="attributeOptions('positive', index)" grouped @update:model-value="value => updateRowSlug('positive', index, value)" /><input v-model="row.value" type="text" inputmode="decimal" :title="`${row.slug && attributeFor(row.slug)?.unit === 'multiply' ? '输入最终倍率，如1.5倍=+50%；' : ''}${rangeDescription(row)}`" :aria-label="`${rowLabel('positive', index)}数值`" :aria-description="rangeDescription(row)" :placeholder="rangePlaceholder(row)" /><span class="analysis-unit">{{ unitText(rangeHintFor(row)?.unit ?? attributeFor(row.slug)?.unit ?? '') }}</span><button class="analysis-remove" type="button" :aria-label="`移除${rowLabel('positive', index)}`" @click="removeRow('positive', index)"><HugeiconsIcon :icon="icons.delete" :size="15" color="currentColor" :stroke-width="1.8" /></button></div></div>
        <div class="analysis-section-title"><span>负面词条</span><button class="analysis-add" type="button" :disabled="negative.length >= 1" aria-label="添加负面词条" @click="addRow('negative')"><HugeiconsIcon :icon="icons.add" :size="16" color="currentColor" :stroke-width="1.8" /></button></div>
        <div class="analysis-rows"><div v-for="(row, index) in negative" :key="row.id" class="analysis-row"><SearchSelect :model-value="row.slug" :input-id="`analysis-negative-${row.id}`" :label="rowLabel('negative', index)" placeholder="选择词条" :options="attributeOptions('negative', index)" grouped nullable @update:model-value="value => updateRowSlug('negative', index, value)" /><input v-model="row.value" type="text" inputmode="decimal" :aria-label="`${rowLabel('negative', index)}数值`" :aria-description="rangeDescription(row)" :title="`${row.slug && attributeFor(row.slug)?.unit === 'multiply' ? '输入最终倍率，如1.5倍=+50%；' : ''}${rangeDescription(row)}`" :placeholder="rangePlaceholder(row)" /><span class="analysis-unit">{{ unitText(rangeHintFor(row)?.unit ?? attributeFor(row.slug)?.unit ?? '') }}</span><button class="analysis-remove" type="button" :aria-label="`移除${rowLabel('negative', index)}`" @click="removeRow('negative', index)"><HugeiconsIcon :icon="icons.delete" :size="15" color="currentColor" :stroke-width="1.8" /></button></div></div>
        <div class="analysis-parameter-grid"><label>强化等级<input v-model="modRank" type="text" inputmode="numeric" maxlength="1" aria-label="强化等级 0 到 8" placeholder="0–8" /></label><label>高价参考挂单数<input v-model="sampleSize" type="text" inputmode="numeric" maxlength="2" aria-label="高价参考挂单数 1 到 50" placeholder="1–50" /></label><label>规则模式<select v-model="ruleMode" aria-label="规则模式"><option value="current">现行规则（1.2）</option><option value="legacy">历史倾向</option></select></label><label v-if="ruleMode === 'legacy'">历史倍率<input v-model="legacyDisposition" type="text" inputmode="decimal" aria-label="历史倍率 0.5 到 1.55" :placeholder="legacyDefaultDisposition == null ? '需输入 0.5–1.55' : String(legacyDefaultDisposition)" /></label></div>
        <div class="analysis-spacer"></div><label class="analysis-label" for="analysis-riven-name">紫卡名称</label><input id="analysis-riven-name" class="analysis-disabled-input" type="text" placeholder="暂未开放" disabled /><button class="primary-button" type="submit" :disabled="!canAnalyze">开始分析</button>
      </form>
      <section class="analysis-result" aria-labelledby="analysis-result-title">
        <h2 id="analysis-result-title">分析结果</h2>
        <p v-if="!tauriAvailable" class="analysis-guide">请在 WM Market 桌面应用中进行分析。</p>
        <template v-else>
          <section class="analysis-result-section">
            <h3>词条范围</h3>
            <p v-if="statsLoading">正在计算本地词条范围…</p>
            <p v-if="statsError" class="analysis-error">{{ statsError }}</p>
            <p v-if="statsResult" class="analysis-result-note">{{ analysisSummary(statsResult) }}</p>
            <div v-if="statsResult" class="analysis-range-list">
              <div v-for="range in statsResult.ranges" :key="range.slug" class="analysis-range-row">
                <div><strong>{{ range.name }}</strong><span class="analysis-range-meta">{{ range.positive ? '正面' : '负面' }} · 输入 {{ valueText(range.value, range.unit) }} · 区间 {{ rangeText(range) }}</span><p v-if="range.message" class="analysis-range-message">{{ range.message }}</p></div>
                <div class="analysis-range-position"><span v-if="range.position_percent != null">区间位置 {{ formatNumber(range.position_percent, 1) }}%</span><span v-else>区间位置 —</span><div v-if="range.position_percent != null" class="analysis-progress" role="progressbar" :aria-valuemin="0" :aria-valuemax="100" :aria-valuenow="range.position_percent" :aria-label="`${range.name}区间位置`"><i :style="progressStyle(range.position_percent)"></i></div><small>{{ statusText(range.status) }}</small></div>
              </div>
            </div>
            <p v-if="!statsLoading && !statsError && !statsResult" class="analysis-guide">填写有效词条后开始分析。</p>
          </section>
          <section class="analysis-result-section"><h3>武器热度</h3><p v-if="marketLoading">正在加载官方周统计与市场参考…</p><p v-if="marketError" class="analysis-error">{{ marketError }}</p><template v-if="marketResult"><p v-if="marketResult.weekly_error" class="analysis-error">{{ marketResult.weekly_error }}</p><div v-for="group in marketResult.weekly?.groups ?? []" :key="String(group.rerolled)" class="analysis-stat-line"><span>{{ group.rerolled ? '已洗卡' : '未洗卡' }}相对热度指数</span><strong>{{ formatNumber(group.popularity, 1) }}</strong><small>官方周数据 · 不区分词条 · 均价 {{ group.avg == null ? '暂无' : formatNumber(group.avg, 1) }} · 中位价 {{ group.median == null ? '暂无' : formatNumber(group.median, 1) }}</small></div><p v-if="marketResult.weekly && !marketResult.weekly.groups.length" class="analysis-guide">该武器暂无周统计</p><p v-if="marketResult.weekly" class="analysis-result-note">来源：{{ marketResult.weekly.source }} · 官方更新时间：{{ marketResult.weekly.source_updated_at ? new Date(marketResult.weekly.source_updated_at).toLocaleString() : '未知' }} · 最近获取：{{ new Date(marketResult.weekly.fetched_at).toLocaleString() }}；不是这张卡估价。</p></template><p v-if="!marketLoading && !marketError && !marketResult" class="analysis-guide">填写有效词条后开始分析。</p></section>
          <section class="analysis-result-section">
          <h3>高价挂单词条匹配</h3>
          <p v-if="marketLoading">正在加载当前批次高价挂单…</p>
          <p v-if="marketError" class="analysis-error">{{ marketError }}</p>
          <template v-if="marketResult">
          <p v-if="marketResult.comparison_error" class="analysis-error">{{ marketResult.comparison_error }}</p>
          <p v-if="marketResult.comparison && marketResult.comparison.sample_count > 0" class="analysis-result-note">当前批次高价挂单 {{ marketResult.comparison?.sample_count ?? 0 }}/{{ marketResult.comparison.requested_count }} 条；报价范围 {{ marketResult.comparison.quoted_min ?? '暂无' }}–{{ marketResult.comparison.quoted_max ?? '暂无' }}p。{{ marketResult.stale ? '部分市场参考未更新。' : '' }}{{ marketResult.from_cache ? '挂单使用缓存。' : '' }}挂单获取：{{ new Date(marketResult.sampled_at).toLocaleString() }}；卖家要价不是已成交或高价值证明。</p>
          <div v-for="frequency in (marketResult.comparison && marketResult.comparison.sample_count > 0 ? marketResult.comparison.frequencies.filter(item => item.selected) : [])" :key="frequency.slug" class="analysis-frequency-line" :class="frequency.positive ? 'positive' : 'negative'">{{ frequency.name }}：{{ frequency.count }}/{{ marketResult.comparison?.sample_count ?? 0 }}</div>
          <div v-for="listing in (marketResult.comparison && marketResult.comparison.sample_count > 0 ? marketResult.comparison.listings : [])" :key="listing.id" class="analysis-listing-line">
          <span>{{ listing.price }}p · <em v-for="attribute in listing.attributes" :key="attribute.slug" :class="attribute.positive ? 'positive' : 'negative'">{{ attribute.name }}</em>
          </span>
          <strong>{{ listing.positive_matches }}/{{ listing.positive_total }} 正面匹配 · 负面{{ listing.negative_match ? '相同' : '不同' }} · {{ listing.exact_match ? '完全匹配' : '非完全匹配' }}</strong>
          </div>
          </template>
          <p v-if="marketResult?.comparison?.sample_count === 0" class="analysis-guide">当前批次没有可用的一口价样本。</p>
          <p v-if="!marketLoading && !marketError && !marketResult" class="analysis-guide">填写有效词条后开始分析。</p>
          </section>
          <section class="analysis-result-section valuation-section">
          <h3>参考估价</h3>
          <p v-if="marketLoading">正在计算参考估价…</p>
          <p v-if="marketError" class="analysis-error">{{ marketError }}</p>
          <p v-if="marketResult?.valuation_error" class="analysis-error">{{ marketResult.valuation_error }}</p>
          <template v-if="marketResult?.valuation">
            <template v-if="marketResult.valuation.status === 'estimated'">
              <div class="valuation-main"><strong v-if="hasPriceRange(marketResult.valuation)">{{ priceText(marketResult.valuation.low) }}–{{ priceText(marketResult.valuation.high) }}</strong><strong v-else>暂无区间</strong><span v-if="finiteNumber(marketResult.valuation.estimate)">估计中心 {{ priceText(marketResult.valuation.estimate) }}</span></div>
              <p class="analysis-result-note">{{ valuationMethod(marketResult.valuation.method) }} · {{ marketResult.valuation.confidence === 'medium' ? '中' : '低' }}置信度；仅供当前批次参考，不保证成交。</p>
              <p class="analysis-result-note">按该武器样本统计：{{ valuationProfile(marketResult.valuation.layout_focus) }}。</p>
              <div v-if="marketResult.valuation.pool_count > 0 && finiteHotStats(marketResult.valuation).length" class="valuation-hot-list"><strong>样本热门词条</strong><span v-for="item in finiteHotStats(marketResult.valuation)" :key="item.slug">{{ item.name }} {{ item.count }}/{{ marketResult.valuation.pool_count }} <small>频率 {{ percentText(item.frequency) }}</small></span></div>
              <details class="valuation-details">
                <summary>估价依据</summary>
                <p>热度微调 {{ adjustmentText(marketResult.valuation.heat_factor - 1) }} · 热门词条微调 {{ adjustmentText(marketResult.valuation.popularity_factor - 1) }} · 数值微调 {{ adjustmentText(marketResult.valuation.numeric_factor - 1) }}</p>
                <p>样本 {{ finiteNumber(marketResult.valuation.comparable_count) ? marketResult.valuation.comparable_count : '暂无' }}/{{ finiteNumber(marketResult.valuation.pool_count) ? marketResult.valuation.pool_count : '暂无' }} · 数值覆盖 {{ percentText(marketResult.valuation.numeric_coverage) }}</p>
                <div v-for="reference in finiteReferences(marketResult.valuation)" :key="reference.id" class="valuation-reference"><span>{{ priceText(reference.price) }} · 匹配度 {{ percentText(reference.similarity) }}</span><small>调整后 {{ priceText(reference.adjusted_price) }} · {{ reference.positive_matches }}/{{ reference.positive_total }} 正面{{ reference.negative_match ? '，负面相同' : '' }}</small></div>
                <p>参考价格来自当前批次卖家报价，不代表成交价。</p>
              </details>
            </template>
            <p v-else class="analysis-guide">暂无可用参考估价：{{ marketResult.valuation.reason || '当前批次没有足够的相近挂单。' }}</p>
            <p v-for="warning in uniqueWarnings(marketResult.valuation)" :key="warning" class="analysis-result-note">{{ warning }}</p>
          </template>
          <p v-if="marketResult && !marketResult.valuation && !marketResult.valuation_error" class="analysis-guide">当前响应暂无估价数据。</p>
          <p v-if="!marketLoading && !marketError && !marketResult" class="analysis-guide">填写有效词条后开始分析。</p>
          </section>
        </template>
      </section>
    </div>
  </section>
</template>

<style scoped>
.analysis-page{min-width:0}
.analysis-heading{margin-bottom:14px}
.analysis-heading h1{margin:0}
.analysis-layout{display:grid;grid-template-columns:minmax(420px,480px) minmax(0,1fr);gap:18px;align-items:start}
.analysis-form{display:flex;min-width:0;min-height:0;flex-direction:column;gap:8px}
.riven-preview{padding:14px;border:1px solid var(--border);border-radius:10px;background:var(--surface-raised);text-align:center}
.riven-preview strong,.analysis-preview-weapon{display:block}
.analysis-preview-weapon{margin-top:3px;color:var(--text-soft);font-size:12px}
.riven-preview-stats{display:grid;gap:4px;margin-top:10px;font-size:12px;text-align:center}
.riven-preview-stats .positive,.analysis-listing-line .positive,.analysis-frequency-line.positive{color:var(--positive)}
.riven-preview-stats .negative,.analysis-listing-line .negative,.analysis-frequency-line.negative{color:var(--negative)}
.analysis-placeholder,.analysis-guide{color:var(--text-soft)}
.analysis-upload-controls{display:flex;align-items:center;justify-content:center;gap:6px}
.analysis-upload{align-self:center}
.analysis-preview-toggle{display:inline-flex;align-items:center;justify-content:center;width:32px;height:32px;padding:0;border:0;color:var(--text-soft);background:transparent;cursor:pointer}
.analysis-preview-toggle:hover{color:var(--text)}
.sr-only{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
.analysis-label{color:var(--text-soft);font-size:11px;font-weight:700}
.analysis-section-title{display:flex;align-items:center;justify-content:space-between;margin-top:2px;color:var(--text-soft);font-size:11px;font-weight:700}
.analysis-add,.analysis-remove{display:inline-flex;align-items:center;justify-content:center;padding:0;border:0;color:var(--text-muted);background:transparent;cursor:pointer}
.analysis-add{width:24px;height:24px}
.analysis-add:disabled{cursor:not-allowed;opacity:.45}
.analysis-rows{display:grid;gap:6px}
.analysis-row{display:grid;grid-template-columns:minmax(0,1fr) minmax(120px,145px) 22px 24px;gap:6px;align-items:center}
.analysis-row input,.analysis-disabled-input,.analysis-parameter-grid input,.analysis-parameter-grid select{width:100%;min-height:32px;padding:0 8px;border:1px solid var(--border);border-radius:6px;color:var(--text);background:var(--surface-raised);font:inherit;font-size:12px}
.analysis-row input::placeholder{color:var(--analysis-placeholder);opacity:1}
.analysis-parameter-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:6px}
.analysis-parameter-grid label{display:grid;gap:4px;color:var(--text-soft);font-size:11px;font-weight:700}
.analysis-unit{color:var(--text-soft);font-size:11px;text-align:center}
.analysis-spacer{flex:0 0 8px;min-height:8px}
.analysis-form>.primary-button{align-self:stretch}
.analysis-result{min-height:320px;padding:12px 0;border-top:1px solid var(--border)}
.analysis-result h2{margin:0 0 16px;font-size:16px}
.analysis-result-section{padding:12px 0;border-bottom:1px solid var(--border)}
.analysis-result-section h3{margin:0 0 8px;font-size:13px}
.analysis-result-section p{margin:6px 0 0;color:var(--text-soft);font-size:12px}
.analysis-error{color:var(--negative)!important}
.analysis-range-row,.analysis-stat-line,.analysis-listing-line{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:8px 0;color:var(--text-soft);font-size:12px}
.analysis-range-row strong,.analysis-stat-line strong{display:block;color:var(--text)}
.analysis-range-meta,.analysis-range-message{display:block;margin-top:3px;color:var(--text-soft);font-size:11px}
.analysis-range-position{flex:0 0 110px;text-align:right}
.analysis-progress{height:4px;margin:4px 0;overflow:hidden;border-radius:999px;background:var(--surface-soft)}
.analysis-progress i{display:block;height:100%;background:var(--primary)}
.analysis-range-position small,.analysis-stat-line small{display:block;color:var(--text-muted);font-size:10px}
.analysis-listing-line em{margin-left:4px;font-style:normal}
.analysis-frequency-line{padding:4px 0;font-size:12px}
.analysis-result-note{line-height:1.5}
.analysis-guide{margin-top:20px;font-size:13px}
.valuation-main{display:flex;align-items:baseline;gap:10px;flex-wrap:wrap;color:var(--text-soft);font-size:11px}
.valuation-main strong{color:var(--text);font-size:18px;letter-spacing:.01em}
.valuation-hot-list{display:flex;gap:6px 12px;flex-wrap:wrap;margin-top:8px;color:var(--text-soft);font-size:11px}
.valuation-hot-list span{display:inline-flex;gap:4px;align-items:baseline}
.valuation-hot-list small{color:var(--text-muted);font-size:10px}
.valuation-details{margin-top:9px;color:var(--text-soft);font-size:11px}
.valuation-details summary{cursor:pointer;color:var(--text)}
.valuation-details p{font-size:11px}
.valuation-reference{display:flex;justify-content:space-between;gap:8px;padding:4px 0;border-top:1px solid var(--border);font-size:11px}
.valuation-reference small{color:var(--text-muted);text-align:right}
.analysis-form input:focus,.analysis-form select:focus{outline:2px solid var(--primary-soft);outline-offset:1px}
@media(max-width:1100px){.analysis-layout{grid-template-columns:1fr}
.analysis-result{min-height:220px}
}

</style>


