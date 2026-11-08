import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { fetchCatalog, fetchCatalogStatus, tauriAvailable, updateCatalog, type CatalogDataStatus, type MarketCatalog } from '../api/market'

export const useCatalogStore = defineStore('catalog', () => {
  const catalog = ref<MarketCatalog | null>(null)
  const status = ref<CatalogDataStatus | null>(null)
  const loading = ref(false)
  const updating = ref(false)
  const error = ref('')
  const updateError = ref('')
  const updateMessage = ref('')
  let loadPromise: Promise<boolean> | null = null
  let updatePromise: Promise<boolean> | null = null
  let statusGeneration = 0

  async function load() {
    if (!tauriAvailable) return false
    if (catalog.value) return true
    if (loadPromise) return loadPromise
    loading.value = true
    error.value = ''
    loadPromise = fetchCatalog()
      .then(result => { catalog.value = result; void refreshStatus(); return true })
      .catch(cause => { error.value = typeof cause === 'string' ? cause : '无法加载目录。'; return false })
      .finally(() => { loading.value = false; loadPromise = null })
    return loadPromise
  }

  async function refreshStatus() {
    if (!tauriAvailable) return
    const generation = statusGeneration
    try {
      const result = await fetchCatalogStatus()
      if (generation === statusGeneration) status.value = result
    } catch { /* status is optional for the market view */ }
  }

  async function update(force = false) {
    if (!tauriAvailable) return false
    if (updatePromise) return updatePromise
    updating.value = true
    updateError.value = ''
    updateMessage.value = ''
    statusGeneration++
    updatePromise = (async () => {
      await load()
      const updateGeneration = ++statusGeneration
      const result = await updateCatalog(force)
      catalog.value = result.catalog
      if (updateGeneration === statusGeneration) status.value = result.status
      updateMessage.value = result.updated ? '数据已更新' : '已是最新数据'
      return true
    })()
      .catch(cause => { updateError.value = typeof cause === 'string' ? cause : '数据更新失败，当前目录仍可使用。'; void refreshStatus(); return false })
      .finally(() => { updating.value = false; updatePromise = null })
    return updatePromise
  }

  return { catalog, status, loading, updating, error, updateError, updateMessage, hasCatalog: computed(() => catalog.value !== null), load, update, refreshStatus }
})
