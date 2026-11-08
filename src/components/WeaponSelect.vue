<script setup lang="ts">
import { computed, ref } from 'vue'
import SearchSelect, { type SearchOption } from './SearchSelect.vue'
import { weaponTypeLabel, type WeaponOption } from '../ui/weaponOptions'

const props = withDefaults(defineProps<{
  modelValue: string | null
  options: WeaponOption[]
  placeholder: string
  label: string
  inputId: string
  disabled?: boolean
}>(), { disabled: false })
const emit = defineEmits<{ 'update:modelValue': [value: string | null] }>()
const typeFilter = ref('')
const filteredTypes = computed(() => {
  const values = Array.from(new Set(props.options.map(option => option.rivenType)))
    .sort((a, b) => weaponTypeLabel(a).localeCompare(weaponTypeLabel(b), 'zh-Hans'))
  return [{ value: '', label: '全部类型' }, ...values.map(value => ({ value, label: weaponTypeLabel(value) }))]
})
function filterOption(option: SearchOption) { return !typeFilter.value || (option as WeaponOption).rivenType === typeFilter.value }
</script>

<template>
  <SearchSelect :model-value="modelValue" :options="options" :placeholder="placeholder" :label="label" :input-id="inputId" :disabled="disabled" :filter-option="filterOption" @update:model-value="value => emit('update:modelValue', typeof value === 'string' ? value : null)">
    <template #header>
      <label class="weapon-select-type-filter"><span>武器类型</span><select v-model="typeFilter" :aria-label="`${label}武器类型`" :disabled="disabled"><option v-for="option in filteredTypes" :key="option.value" :value="option.value">{{ option.label }}</option></select></label>
    </template>
  </SearchSelect>
</template>
