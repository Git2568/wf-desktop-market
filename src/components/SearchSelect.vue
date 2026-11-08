<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { HugeiconsIcon } from '@hugeicons/vue'
import { icons } from '../icons'

export type SearchOption = { value: string; label: string; secondary?: string; group?: string }

const props = withDefaults(defineProps<{
  modelValue: string | string[] | null
  options: SearchOption[]
  placeholder: string
  label: string
  inputId: string
  multiple?: boolean
  max?: number
  nullable?: boolean
  disabled?: boolean
  filterOption?: (option: SearchOption) => boolean
  grouped?: boolean
}>(), { multiple: false, max: 99, nullable: false, disabled: false, grouped: false })

const emit = defineEmits<{
  'update:modelValue': [value: string | string[] | null]
}>()
const root = ref<HTMLElement | null>(null)
const panelElement = ref<HTMLElement | null>(null)
const listboxElement = ref<HTMLElement | null>(null)
const inputElement = ref<HTMLInputElement | null>(null)
const keyword = ref('')
const open = ref(false)
const active = ref(-1)
const inputWasEdited = ref(false)
const suppressNextFocusOpen = ref(false)
const panelStyle = ref<Record<string, string>>({})
const listboxStyle = ref<Record<string, string>>({})

const selected = computed(() => {
  if (Array.isArray(props.modelValue)) {
    return props.modelValue
  }
  return props.modelValue ? [props.modelValue] : []
})

const normalizedKeyword = computed(() => keyword.value.trim().toLocaleLowerCase())

const filtered = computed(() => props.options.filter((option) => {
  if (props.filterOption && !props.filterOption(option)) return false
  if (selected.value.includes(option.value)) {
    return false
  }

  const searchable = `${option.label} ${option.secondary ?? ''} ${option.value}`
    .toLocaleLowerCase()
  return searchable.includes(normalizedKeyword.value)
}))

const groupedFiltered = computed(() => {
  const groups = new Map<string, SearchOption[]>()
  for (const option of filtered.value) {
    const key = option.group ?? 'other'
    const items = groups.get(key) ?? []
    items.push(option)
    groups.set(key, items)
  }
  return [...groups.entries()].map(([key, options]) => ({ key, options }))
})

const hasCapacity = computed(() => !props.multiple || selected.value.length < props.max)

function optionId(option: SearchOption) {
  return `${props.inputId}-option-${encodeURIComponent(option.value)}`
}

const activeOption = computed(() => {
  if (!open.value) {
    return undefined
  }

  const option = filtered.value[active.value]
  return option && hasCapacity.value ? option : undefined
})

function optionLabel(value: string) {
  return props.options.find((option) => option.value === value)?.label ?? value
}

function optionFor(value: string) {
  return props.options.find((option) => option.value === value)
}

function focusInput() {
  if (!props.disabled) {
    open.value = true
    inputElement.value?.focus()
  }
}

function openList() {
  if (!props.disabled) {
    open.value = true
  }
}

function updatePanelPosition() {
  if (!props.grouped || !root.value || !open.value) return
  const rect = root.value.getBoundingClientRect()
  const width = Math.max(rect.width, 660)
  const maxWidth = Math.max(280, window.innerWidth - 24)
  const left = Math.min(Math.max(12, rect.left), Math.max(12, window.innerWidth - Math.min(width, maxWidth) - 12))
  if (panelElement.value) panelElement.value.style.width = `${Math.min(width, maxWidth)}px`
  const listbox = listboxElement.value
  const panelChrome = panelElement.value && listbox ? Math.max(0, panelElement.value.offsetHeight - listbox.offsetHeight) : 34
  const naturalListHeight = listbox?.scrollHeight ?? 0
  const belowTop = rect.bottom + 6
  const aboveBottom = rect.top - 6
  const belowSpace = window.innerHeight - 12 - belowTop
  const aboveSpace = aboveBottom - 12
  const naturalPanelHeight = naturalListHeight + panelChrome
  const useAbove = naturalPanelHeight > belowSpace && aboveSpace > belowSpace
  const available = Math.max(80, Math.min(540, useAbove ? aboveSpace : belowSpace))
  const listMax = Math.max(64, Math.min(480, available - panelChrome))
  const panelHeight = Math.min(naturalListHeight, listMax) + panelChrome
  const top = useAbove ? aboveBottom - panelHeight : belowTop
  panelStyle.value = { left: `${left}px`, top: `${Math.max(12, top)}px`, width: `${Math.min(width, maxWidth)}px` }
  listboxStyle.value = { maxHeight: `${listMax}px` }
}

function updateKeyword(event: Event) {
  if (props.disabled) {
    return
  }

  keyword.value = (event.target as HTMLInputElement).value
  inputWasEdited.value = true
  if (!props.multiple && selected.value.length > 0) {
    emit('update:modelValue', props.nullable ? null : '')
  }
  queueMicrotask(() => {
    inputWasEdited.value = false
  })
}

function choose(option: SearchOption) {
  if (props.disabled || (props.multiple && !hasCapacity.value)) {
    return
  }

  inputWasEdited.value = false
  keyword.value = ''
  open.value = false
  active.value = -1
  if (props.multiple) {
    emit('update:modelValue', [...selected.value, option.value])
  } else {
    emit('update:modelValue', option.value)
  }
}

function remove(value: string) {
  if (props.disabled) {
    return
  }

  inputWasEdited.value = false
  keyword.value = ''
  if (props.multiple) {
    emit('update:modelValue', selected.value.filter((item) => item !== value))
  } else {
    emit('update:modelValue', props.nullable ? null : '')
  }
}

function moveActive(direction: 1 | -1) {
  if (props.disabled) {
    return
  }

  open.value = true
  if (!filtered.value.length) {
    active.value = -1
    return
  }
  const next = active.value + direction
  active.value = next < 0 ? filtered.value.length - 1 : next >= filtered.value.length ? 0 : next
}

function keydown(event: KeyboardEvent) {
  if (props.disabled) {
    return
  }

  if (event.key === 'ArrowDown') {
    event.preventDefault()
    moveActive(1)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    moveActive(-1)
  } else if (event.key === 'Enter' && open.value) {
    event.preventDefault()
    event.stopPropagation()
    const option = activeOption.value ?? filtered.value.find(() => hasCapacity.value)
    if (option) choose(option)
  } else if (event.key === 'Escape' && open.value) {
    event.preventDefault()
    event.stopPropagation()
    open.value = false
    suppressNextFocusOpen.value = true
  } else if (event.key === 'Backspace' && !keyword.value && selected.value.length > 0) {
    remove(selected.value[selected.value.length - 1])
  }
}

function handleRootKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && open.value && event.target !== inputElement.value) {
    event.preventDefault()
    event.stopPropagation()
    open.value = false
    suppressNextFocusOpen.value = true
  }
}

function handleInputFocus() {
  if (suppressNextFocusOpen.value) {
    suppressNextFocusOpen.value = false
    return
  }
  openList()
}

function outsidePointer(event: PointerEvent) {
  if (root.value && !root.value.contains(event.target as Node)) {
    open.value = false
  }
}

function handleWindowScroll(event: Event) {
  if (props.grouped && panelElement.value?.contains(event.target as Node)) return
  updatePanelPosition()
}

function handleFocusout(event: FocusEvent) {
  const next = event.relatedTarget as Node | null
  window.setTimeout(() => {
    const activeElement = document.activeElement
    if (!root.value?.contains(next ?? activeElement)) open.value = false
  }, 0)
}

function scrollActiveIntoView() {
  if (!root.value || active.value < 0) {
    return
  }

  const option = filtered.value[active.value]
  if (!option) {
    return
  }

  const element = root.value.querySelector<HTMLElement>(`#${CSS.escape(optionId(option))}`)
  if (!element) return
  if (props.grouped && listboxElement.value) {
    const listRect = listboxElement.value.getBoundingClientRect()
    const optionRect = element.getBoundingClientRect()
    if (optionRect.top < listRect.top) listboxElement.value.scrollTop -= listRect.top - optionRect.top
    else if (optionRect.bottom > listRect.bottom) listboxElement.value.scrollTop += optionRect.bottom - listRect.bottom
    return
  }
  element.scrollIntoView({ block: 'nearest' })
}

watch(filtered, (options) => {
  active.value = options.length ? 0 : -1
  if (open.value) void nextTick(scrollActiveIntoView)
})

watch(() => props.modelValue, () => {
  if (!inputWasEdited.value) keyword.value = ''
})

watch(() => props.disabled, (disabled) => {
  if (disabled) {
    open.value = false
  }
})

watch(active, () => {
  if (open.value) void nextTick(scrollActiveIntoView)
})

onMounted(() => document.addEventListener('pointerdown', outsidePointer))
onMounted(() => {
  window.addEventListener('resize', updatePanelPosition)
  window.addEventListener('scroll', handleWindowScroll, true)
})
onUnmounted(() => {
  document.removeEventListener('pointerdown', outsidePointer)
  window.removeEventListener('resize', updatePanelPosition)
  window.removeEventListener('scroll', handleWindowScroll, true)
})
watch([open, () => props.grouped, filtered], () => void nextTick(updatePanelPosition))
</script>

<template>
  <div ref="root" class="search-select" :class="{ 'is-open': open }" @focusout="handleFocusout" @keydown="handleRootKeydown">
    <div class="search-select-control" :class="{ 'is-disabled': disabled }" @click="focusInput">
      <span v-for="value in selected" :key="value" class="select-token">
        <slot name="selected" :value="value" :option="optionFor(value)">{{ optionLabel(value) }}</slot>
        <button type="button" :aria-label="`移除${optionLabel(value)}`" :disabled="disabled" @click.stop="remove(value)">
          <HugeiconsIcon :icon="icons.close" :size="13" color="currentColor" :stroke-width="1.8" />
        </button>
      </span>
      <input
        ref="inputElement"
        :id="inputId"
        :value="keyword"
        type="search"
        :placeholder="selected.length ? '' : placeholder"
        :disabled="disabled"
        role="combobox"
        :aria-label="label"
        aria-autocomplete="list"
        :aria-expanded="open"
        :aria-controls="`${inputId}-listbox`"
        :aria-activedescendant="activeOption ? optionId(activeOption) : undefined"
        @input="updateKeyword"
        @focus="handleInputFocus"
        @keydown="keydown"
      />
    </div>
    <div v-if="open" ref="panelElement" class="search-select-options" :class="{ 'is-grouped': grouped }" :style="grouped ? panelStyle : undefined">
      <div v-if="$slots.header" class="search-select-header"><slot name="header" /></div>
      <div ref="listboxElement" :id="`${inputId}-listbox`" class="search-select-listbox" :class="{ 'is-grouped': grouped }" :style="grouped ? listboxStyle : undefined" role="listbox" :aria-label="label" :aria-multiselectable="multiple">
      <template v-if="grouped">
        <section v-for="group in groupedFiltered" :key="group.key" class="search-select-group" role="group" :aria-label="group.key">
          <h4>{{ group.key }}</h4>
          <button v-for="option in group.options" :id="optionId(option)" :key="option.value" type="button" role="option" tabindex="-1" :aria-selected="selected.includes(option.value)" :aria-disabled="multiple && !hasCapacity ? 'true' : undefined" :disabled="multiple && !hasCapacity" :class="{ active: filtered[active]?.value === option.value }" @mousedown.prevent @click="choose(option)">
            <slot name="option" :option="option"><span>{{ option.label }}</span><small>{{ option.secondary ?? option.value }}</small></slot>
          </button>
        </section>
      </template>
      <template v-else>
      <button v-for="(option, index) in filtered" :id="optionId(option)" :key="option.value" type="button" role="option" tabindex="-1" :aria-selected="selected.includes(option.value)" :aria-disabled="multiple && !hasCapacity ? 'true' : undefined" :disabled="multiple && !hasCapacity" :class="{ active: index === active }" @mousedown.prevent @click="choose(option)">
        <slot name="option" :option="option"><span>{{ option.label }}</span><small>{{ option.secondary ?? option.value }}</small></slot>
      </button>
      </template>
      <p v-if="!filtered.length" class="search-select-empty">无匹配选项</p>
      </div>
    </div>
  </div>
</template>
