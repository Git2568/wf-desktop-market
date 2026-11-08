<script setup lang="ts">
import { nextTick, onUnmounted, ref, watch } from 'vue'
import { HugeiconsIcon } from '@hugeicons/vue'
import { icons } from '../icons'

const props = defineProps<{ listingId: string; rivenName: string; open: boolean }>()
const emit = defineEmits<{ toggle: []; close: [] }>()
const trigger = ref<HTMLButtonElement | null>(null)
const panel = ref<HTMLElement | null>(null)
const top = ref(0)
const left = ref(0)

function close(focus = false) {
  emit('close')
  if (focus) nextTick(() => trigger.value?.focus())
}

async function position() {
  await nextTick()
  if (!trigger.value || !panel.value) return
  const triggerRect = trigger.value.getBoundingClientRect()
  const panelRect = panel.value.getBoundingClientRect()
  left.value = Math.max(8, Math.min(triggerRect.right - panelRect.width, window.innerWidth - panelRect.width - 8))
  top.value = triggerRect.bottom + 6
  if (top.value + panelRect.height > window.innerHeight - 8) top.value = Math.max(8, triggerRect.top - panelRect.height - 6)
  await nextTick()
  if (props.open) panel.value?.focus({ preventScroll: true })
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    close(true)
  } else if (event.key === 'Tab') {
    close()
  }
}

function onPointerDown(event: MouseEvent) {
  const target = event.target as Node
  if (!trigger.value?.contains(target) && !panel.value?.contains(target)) close()
}

function onFocusIn(event: FocusEvent) {
  const target = event.target as Node
  if (!trigger.value?.contains(target) && !panel.value?.contains(target)) close()
}

function onViewportChange() { close() }
function toggle() { emit('toggle') }

watch(() => props.open, open => {
  if (open) {
    document.addEventListener('keydown', onKeydown)
    document.addEventListener('mousedown', onPointerDown)
    document.addEventListener('focusin', onFocusIn)
    window.addEventListener('scroll', onViewportChange, true)
    window.addEventListener('resize', onViewportChange)
    position()
  } else {
    document.removeEventListener('keydown', onKeydown)
    document.removeEventListener('mousedown', onPointerDown)
    document.removeEventListener('focusin', onFocusIn)
    window.removeEventListener('scroll', onViewportChange, true)
    window.removeEventListener('resize', onViewportChange)
  }
})
onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
  document.removeEventListener('mousedown', onPointerDown)
  document.removeEventListener('focusin', onFocusIn)
  window.removeEventListener('scroll', onViewportChange, true)
  window.removeEventListener('resize', onViewportChange)
})
</script>

<template>
  <div class="listing-actions">
    <button ref="trigger" class="listing-actions-trigger" type="button" :aria-label="`打开${rivenName}的操作菜单`" aria-haspopup="menu" :aria-expanded="open" :aria-controls="open ? `listing-actions-${listingId}` : undefined" @click="toggle">
      <HugeiconsIcon :icon="icons.more" :size="18" color="currentColor" :stroke-width="1.8" />
    </button>
    <Teleport to="body">
      <div v-if="open" :id="`listing-actions-${listingId}`" ref="panel" class="listing-actions-menu" role="menu" :aria-label="`${rivenName}的操作菜单`" tabindex="-1" :style="{ top: `${top}px`, left: `${left}px` }">
        <button type="button" role="menuitem" disabled aria-disabled="true">拷贝购买请求</button>
        <button type="button" role="menuitem" disabled aria-disabled="true">分享</button>
        <button type="button" role="menuitem" disabled aria-disabled="true">筛选类似紫卡</button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.listing-actions { display: flex; justify-content: center; }
.listing-actions-trigger { display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; padding: 0; border: 0; border-radius: 6px; color: var(--text-muted); background: transparent; cursor: pointer; }
.listing-actions-trigger:hover { color: var(--text); background: var(--surface-soft); }
.listing-actions-menu { position: fixed; z-index: 100; display: grid; width: 184px; gap: 2px; padding: 4px; border: 1px solid var(--border); border-radius: 8px; background: var(--surface-raised); box-shadow: var(--shadow); }
.listing-actions-menu { max-height: calc(100vh - 16px); overflow-y: auto; }
.listing-actions-menu button { min-height: 30px; padding: 0 8px; border: 0; border-radius: 4px; color: var(--text); background: transparent; cursor: pointer; text-align: left; }
.listing-actions-menu button:disabled { cursor: not-allowed; opacity: .48; }
</style>
