import assert from 'node:assert/strict'
import { after, before, test } from 'node:test'
import { createServer } from 'vite'
import { createPinia, setActivePinia } from 'pinia'

const catalog = {
  weapons: [{ id: 'w1', slug: 'torid', riven_type: 'rifle', i18n: { en: { name: 'Torid' } } }],
  attributes: [{ id: 'a1', slug: 'critical_chance', name: '暴击几率', english_name: 'Critical Chance', unit: '%', positive_only: false, negative_only: false, exclusive_to: [] }],
}
const updatedCatalog = { ...catalog, weapons: [{ ...catalog.weapons[0], slug: 'soma' }] }
const statuses = {
  old: { collection_version: 'old', game_version: null, api_version: '0.25.0', source: 'seed', language: 'zh-hans', platform: 'pc', downloaded_at: 1, last_checked_at: 2, weapons_count: 1, attributes_count: 1, storage: 'sqlite', last_error: null },
  fresh: { collection_version: 'fresh', game_version: null, api_version: '0.25.0', source: 'seed', language: 'zh-hans', platform: 'pc', downloaded_at: 3, last_checked_at: 4, weapons_count: 1, attributes_count: 1, storage: 'sqlite', last_error: null },
}

let server
let useCatalogStore
let invokeImpl

before(async () => {
  globalThis.isTauri = true
  globalThis.window = { __TAURI_INTERNALS__: { invoke: (...args) => invokeImpl(...args) } }
  server = await createServer({ server: { middlewareMode: true, hmr: false }, appType: 'custom' })
  ;({ useCatalogStore } = await server.ssrLoadModule('/src/stores/catalog.ts'))
})

after(async () => {
  await server.close()
  delete globalThis.window
  delete globalThis.isTauri
})

function freshStore() {
  setActivePinia(createPinia())
  invokeImpl = async (command) => {
    if (command === 'market_catalog') return catalog
    if (command === 'market_catalog_status') return statuses.old
    throw new Error(`unexpected invoke ${command}`)
  }
  return useCatalogStore()
}

function deferred() {
  let resolve
  let reject
  const promise = new Promise((res, rej) => { resolve = res; reject = rej })
  return { promise, resolve, reject }
}

test('duplicate load shares one catalog IPC and loaded load does not overwrite', async () => {
  const store = freshStore()
  const pending = deferred()
  let catalogCalls = 0
  invokeImpl = async command => {
    if (command === 'market_catalog') { catalogCalls++; return pending.promise }
    if (command === 'market_catalog_status') return statuses.old
    throw new Error(command)
  }
  const first = store.load()
  const second = store.load()
  assert.equal(catalogCalls, 1)
  pending.resolve(catalog)
  assert.equal(await first, true)
  assert.equal(await second, true)
  const loaded = store.catalog
  assert.equal(await store.load(), true)
  assert.deepEqual(store.catalog, loaded)
  assert.equal(catalogCalls, 1)
})

test('update waits for pending load and late status cannot roll back fresh status', async () => {
  const store = freshStore()
  const loading = deferred()
  const lateStatus = deferred()
  const updateResult = deferred()
  const calls = []
  invokeImpl = async command => {
    calls.push(command)
    if (command === 'market_catalog') return loading.promise
    if (command === 'market_catalog_status') return lateStatus.promise
    if (command === 'market_update_catalog') { assert.fail('update was called before load resolved') }
    throw new Error(command)
  }
  const loadPromise = store.load()
  const updatePromise = store.update(false)
  assert.deepEqual(calls, ['market_catalog'])
  loading.resolve(catalog)
  await new Promise(resolve => queueMicrotask(resolve))
  invokeImpl = async command => {
    calls.push(command)
    if (command === 'market_catalog_status') return lateStatus.promise
    if (command === 'market_update_catalog') return updateResult.promise
    throw new Error(command)
  }
  updateResult.resolve({ catalog: updatedCatalog, status: statuses.fresh, updated: true })
  assert.equal(await loadPromise, true)
  assert.equal(await updatePromise, true)
  lateStatus.resolve(statuses.old)
  await new Promise(resolve => setTimeout(resolve, 0))
  assert.equal(store.catalog.weapons[0].slug, 'soma')
  assert.equal(store.status.collection_version, 'fresh')
  assert.equal(store.updateMessage, '数据已更新')
})

test('duplicate update uses one IPC and preserves force argument', async () => {
  const store = freshStore()
  store.catalog = catalog
  let calls = 0
  const pending = deferred()
  invokeImpl = async (command, args) => {
    if (command === 'market_update_catalog') { calls++; assert.equal(args.force, true); return pending.promise }
    if (command === 'market_catalog_status') return statuses.old
    throw new Error(command)
  }
  const first = store.update(true)
  const second = store.update(true)
  await new Promise(resolve => queueMicrotask(resolve))
  assert.equal(calls, 1)
  pending.resolve({ catalog, status: statuses.fresh, updated: false })
  assert.equal(await first, true)
  assert.equal(await second, true)
  assert.equal(store.updateMessage, '已是最新数据')
})

test('failed update keeps old catalog, resets state, and can retry', async () => {
  const store = freshStore()
  store.catalog = catalog
  let shouldFail = true
  invokeImpl = async command => {
    if (command === 'market_update_catalog') {
      if (shouldFail) throw new Error('offline')
      return { catalog: updatedCatalog, status: statuses.fresh, updated: true }
    }
    if (command === 'market_catalog_status') return statuses.old
    throw new Error(command)
  }
  assert.equal(await store.update(), false)
  assert.equal(store.catalog.weapons[0].slug, 'torid')
  assert.equal(store.updating, false)
  shouldFail = false
  assert.equal(await store.update(), true)
  assert.equal(store.catalog.weapons[0].slug, 'soma')
})

test('non-Tauri environment does not issue catalog IPC', async () => {
  const serverWithoutTauri = await createServer({ server: { middlewareMode: true, hmr: false }, appType: 'custom' })
  const original = globalThis.isTauri
  globalThis.isTauri = false
  let calls = 0
  globalThis.window = { __TAURI_INTERNALS__: { invoke: () => { calls++ } } }
  try {
    const module = await serverWithoutTauri.ssrLoadModule('/src/stores/catalog.ts?without-tauri')
    setActivePinia(createPinia())
    const store = module.useCatalogStore()
    assert.equal(await store.load(), false)
    assert.equal(await store.update(), false)
    assert.equal(calls, 0)
  } finally {
    globalThis.isTauri = original
    await serverWithoutTauri.close()
  }
})
