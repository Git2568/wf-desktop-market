<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { tauriAvailable } from '../api/market'
import { fetchAnalysisDataStatus, refreshAnalysisData, type AnalysisDataStatus } from '../api/analysis'
import { useCatalogStore } from '../stores/catalog'

const catalog = useCatalogStore()
const message = computed(() => catalog.updateError || (catalog.status?.last_error ? '上次本地保存警告：当前目录仍可使用。' : ''))
const analysisStatus = ref<AnalysisDataStatus | null>(null)
const analysisUpdating = ref(false)
const analysisError = ref('')
const alive = ref(true)
function formatTime(value: number | null | undefined) { return value ? new Date(value).toLocaleString() : '未记录' }
function errorText(cause: unknown, fallback: string) { return typeof cause === 'string' ? cause : cause instanceof Error ? cause.message : fallback }
async function update(force = false) {
  if (!tauriAvailable || catalog.updating || analysisUpdating.value) return
  analysisUpdating.value = true
  analysisError.value = ''
  const results = await Promise.allSettled([catalog.update(force), refreshAnalysisData(force)])
  const analysis = results[1]
  if (analysis.status === 'fulfilled') analysisStatus.value = analysis.value
  else analysisError.value = errorText(analysis.reason, '分析数据更新失败，当前本地规则仍可使用。')
  analysisUpdating.value = false
}
onMounted(async () => {
  if (!tauriAvailable) return
  try { const status = await fetchAnalysisDataStatus(); if (alive.value) analysisStatus.value = status }
  catch (cause) { if (alive.value) analysisError.value = errorText(cause, '无法读取分析数据状态。') }
})
onUnmounted(() => { alive.value = false })
</script>

<template>
  <section class="data-settings" aria-labelledby="data-settings-title">
    <h3 id="data-settings-title">数据更新</h3>
    <p v-if="!tauriAvailable" class="data-settings-note">请在桌面应用中更新数据。</p>
    <dl v-else-if="catalog.status" class="data-settings-meta">
      <div><dt>目录版本</dt><dd>{{ catalog.status.collection_version ?? '未记录' }}</dd></div>
      <div><dt>武器/词条</dt><dd>{{ catalog.status.weapons_count }} / {{ catalog.status.attributes_count }}</dd></div>
      <div><dt>最后更新时间</dt><dd>{{ formatTime(catalog.status.downloaded_at) }}</dd></div>
      <div><dt>最近检查</dt><dd>{{ formatTime(catalog.status.last_checked_at) }}</dd></div>
      <div><dt>游戏版本</dt><dd>{{ catalog.status.game_version ?? '未记录' }}</dd></div>
    </dl>
    <p v-else class="data-settings-note">尚未读取目录版本。</p>
    <p v-if="catalog.updating" class="data-settings-note" aria-live="polite">正在检查并更新…</p>
    <p v-else-if="catalog.updateMessage" class="data-settings-note" aria-live="polite">{{ catalog.updateMessage }}</p>
    <p v-if="message" class="data-settings-error">{{ message }}</p>
    <dl v-if="tauriAvailable && analysisStatus" class="data-settings-meta analysis-data-meta">
      <div><dt>分析规则版本</dt><dd>{{ analysisStatus.rule_version }}</dd></div>
      <div><dt>官方周数据更新时间</dt><dd>{{ analysisStatus.weekly_source_updated_at ? new Date(analysisStatus.weekly_source_updated_at).toLocaleString() : '官方更新时间未知' }}</dd></div>
      <div><dt>分析数据最近获取</dt><dd>{{ formatTime(analysisStatus.weekly_fetched_at) }}</dd></div>
    </dl>
    <p v-if="analysisError" class="data-settings-error">{{ analysisError }}</p>
    <p v-if="analysisStatus?.message" class="data-settings-note">{{ analysisStatus.message }}</p>
    <div class="data-settings-actions">
      <button class="primary-button" type="button" :disabled="catalog.updating || analysisUpdating || !tauriAvailable" @click="update(false)">检查并更新</button>
      <button class="secondary-button" type="button" :disabled="catalog.updating || analysisUpdating || !tauriAvailable" @click="update(true)">重新下载</button>
    </div>
  </section>
</template>

<style scoped>
.data-settings { margin-top: 18px; padding-top: 16px; border-top: 1px solid var(--border); }
.data-settings h3 { margin: 0 0 9px; font-size: 14px; }
.data-settings-meta { display: grid; gap: 6px; margin: 0; font-size: 12px; }
.data-settings-meta div { display: flex; justify-content: space-between; gap: 9px; }
.data-settings-meta dt { color: var(--text-soft); }
.data-settings-meta dd { max-width: 62%; margin: 0; overflow-wrap: anywhere; color: var(--text); text-align: right; }
.data-settings-note, .data-settings-error { margin: 8px 0 0; color: var(--text-soft); font-size: 12px; }
.data-settings-error { color: var(--negative); }
.data-settings-actions { display: flex; gap: 6px; margin-top: 11px; }
.data-settings-actions button { min-height: 32px; padding-inline: 8px; border-radius: 6px; font-size: 12px; }
</style>
