<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { HugeiconsIcon } from '@hugeicons/vue'
import { icons } from '../icons'

const props = defineProps<{ modelValue: boolean; title: string; labelledby: string; restoreSelector?: string; wide?: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [boolean] }>()
const dialogRef = ref<HTMLElement | null>(null)
let restoreFocus: HTMLElement | null = null

function close() {
  emit('update:modelValue', false)
}
function trapFocus(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault()
    close()
    return
  }
  if (event.key !== 'Tab' || !dialogRef.value) return
  const focusables = Array.from(dialogRef.value.querySelectorAll<HTMLElement>('button, [href], input, select, textarea, [tabindex]')).filter(element => element.tabIndex >= 0 && !element.hasAttribute('disabled') && element.offsetParent !== null)
  if (!focusables.length) return
  const first = focusables[0]; const last = focusables[focusables.length - 1]
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first.focus()
  }
}
watch(() => props.modelValue, async (open) => {
  if (open) {
    restoreFocus = props.restoreSelector
      ? document.querySelector<HTMLElement>(props.restoreSelector)
      : document.activeElement as HTMLElement
    await nextTick()
    dialogRef.value?.querySelector<HTMLElement>('button:not([disabled])')?.focus()
  } else {
    await nextTick()
    restoreFocus?.focus()
  }
})
</script>

<template>
  <Teleport to="body">
    <div v-if="modelValue" class="dialog-backdrop" @mousedown.self="close">
      <section ref="dialogRef" class="app-dialog" :class="{ 'app-dialog-wide': wide }" role="dialog" aria-modal="true" :aria-labelledby="labelledby" tabindex="-1" @keydown="trapFocus">
        <header class="dialog-header"><h2 :id="labelledby">{{ title }}</h2><button class="dialog-close" type="button" aria-label="关闭" @click="close"><HugeiconsIcon :icon="icons.close" :size="18" color="currentColor" :stroke-width="1.7" /></button></header>
        <div class="dialog-body"><slot /></div>
      </section>
    </div>
  </Teleport>
</template>
