<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { isTauri } from '@tauri-apps/api/core'
import { HugeiconsIcon } from '@hugeicons/vue'
import { icons } from '../icons'

const props = defineProps<{ isDark: boolean }>()
const emit = defineEmits<{ 'toggle-theme': [] }>()
const maximized = ref(false)
const tauriAvailable = isTauri()
let alive = true
let unlistenResized: (() => void) | null = null

const themeLabel = computed(() => props.isDark ? '切换到亮色主题' : '切换到暗色主题')
const maximizeLabel = computed(() => maximized.value ? '还原窗口' : '最大化窗口')
const maximizeIcon = computed(() => maximized.value ? icons.restoreWindow : icons.maximizeWindow)

async function refreshMaximized() {
  if (!tauriAvailable || !alive) return
  try {
    const value = await getCurrentWindow().isMaximized()
    if (alive) maximized.value = value
  } catch { /* closing window */ }
}

async function minimize() {
  if (!tauriAvailable) return
  try { await getCurrentWindow().minimize() } catch { /* closing window */ }
}

async function toggleMaximize() {
  if (!tauriAvailable) return
  try { await getCurrentWindow().toggleMaximize(); await refreshMaximized() } catch { /* closing window */ }
}

async function closeWindow() {
  if (!tauriAvailable) return
  try { await getCurrentWindow().close() } catch { /* closing window */ }
}

onMounted(async () => {
  if (!tauriAvailable) return
  await refreshMaximized()
  if (!alive) return
  try {
    const stop = await getCurrentWindow().onResized(() => { void refreshMaximized() })
    if (alive) unlistenResized = stop
    else stop()
  } catch { /* optional listener */ }
})

onUnmounted(() => {
  alive = false
  unlistenResized?.()
  unlistenResized = null
})
</script>

<template>
  <header class="window-titlebar">
    <div class="window-drag-region" data-tauri-drag-region aria-hidden="true"></div>
    <div class="window-titlebar-actions">
      <button class="window-control" type="button" :aria-label="themeLabel" :title="themeLabel" @click="emit('toggle-theme')">
        <HugeiconsIcon :icon="props.isDark ? icons.light : icons.dark" :size="17" color="currentColor" :stroke-width="1.7" />
      </button>
      <button class="window-control" type="button" :disabled="!tauriAvailable" aria-label="最小化窗口" title="最小化窗口" @click="minimize">
        <HugeiconsIcon :icon="icons.minimizeWindow" :size="16" color="currentColor" :stroke-width="1.7" />
      </button>
      <button class="window-control" type="button" :disabled="!tauriAvailable" :aria-label="maximizeLabel" :title="maximizeLabel" @click="toggleMaximize">
        <HugeiconsIcon :icon="maximizeIcon" :size="15" color="currentColor" :stroke-width="1.7" />
      </button>
      <button class="window-control window-close" type="button" :disabled="!tauriAvailable" aria-label="关闭窗口" title="关闭窗口" @click="closeWindow">
        <HugeiconsIcon :icon="icons.close" :size="16" color="currentColor" :stroke-width="1.7" />
      </button>
    </div>
  </header>
</template>
