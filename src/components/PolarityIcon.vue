<script setup lang="ts">
import { computed } from 'vue'
import { polarities, type PolarityKey } from '../data/polarities'

const props = defineProps<{ polarity: string | null }>()
const known = computed(() => props.polarity && Object.prototype.hasOwnProperty.call(polarities, props.polarity) ? polarities[props.polarity as PolarityKey] : null)
const label = computed(() => known.value ? `${known.value.name} 极性` : props.polarity ? `未知极性：${props.polarity}` : '极性未知')
</script>

<template>
  <span v-if="known" class="polarity-icon" role="img" :aria-label="label" :title="label" :style="{ '--polarity-mask': `url(${known.asset})` }"></span>
  <span v-else class="polarity-fallback" :title="label">{{ props.polarity || '极性未知' }}</span>
</template>

<style scoped>
.polarity-icon { display: inline-block; width: 14px; height: 14px; flex: 0 0 14px; vertical-align: -2px; background-color: currentColor; -webkit-mask-image: var(--polarity-mask); mask-image: var(--polarity-mask); -webkit-mask-position: center; mask-position: center; -webkit-mask-repeat: no-repeat; mask-repeat: no-repeat; -webkit-mask-size: contain; mask-size: contain; }
.polarity-fallback { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
