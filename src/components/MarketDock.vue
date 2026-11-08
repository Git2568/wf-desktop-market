<script setup lang="ts">
import { HugeiconsIcon } from '@hugeicons/vue'
import { icons } from '../icons'
import type { MarketPageId } from '../stores/ui'

defineProps<{ modelValue: MarketPageId }>()
const emit = defineEmits<{ 'update:modelValue': [MarketPageId] }>()
const items: Array<{ id: MarketPageId; label: string; icon: typeof icons.market }> = [
  { id: 'purchase', label: '紫卡购买', icon: icons.market },
  { id: 'wm-user', label: 'WM用户', icon: icons.marketUser },
  { id: 'my-listings', label: '我的上架', icon: icons.marketListings },
]
</script>

<template>
  <div class="market-dock-wrap">
    <nav class="market-dock" aria-label="市场页面">
      <div
        v-for="item in items"
        :key="item.id"
        class="market-dock-slot"
      >
        <button
          class="market-dock-item"
          :class="{ active: modelValue === item.id }"
          type="button"
          :aria-current="modelValue === item.id ? 'page' : undefined"
          :aria-label="item.label"
          @click="emit('update:modelValue', item.id)"
        >
          <HugeiconsIcon :icon="item.icon" :size="18" color="currentColor" :stroke-width="1.7" />
          <span>{{ item.label }}</span>
        </button>
      </div>
    </nav>
  </div>
</template>

<style scoped>
.market-dock-wrap {
  display: flex;
  flex: 0 0 auto;
  justify-content: center;
  padding: 6px 12px 0;
}

.market-dock {
  display: grid;
  width: min(288px, 100%);
  height: 36px;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin: 0;
  gap: 0;
  padding: 0;
  border: 0;
  border-radius: 14px 14px 0 0;
  background: color-mix(in srgb, var(--surface-raised) 72%, transparent);
  box-shadow: 0 -3px 10px color-mix(in srgb, var(--shadow-color) 60%, transparent);
  backdrop-filter: blur(18px) saturate(1.15);
}

.market-dock-slot {
  position: relative;
  display: grid;
  min-width: 0;
  height: 100%;
  place-items: center;
}

.market-dock-slot:not(:last-child)::after {
  position: absolute;
  top: 25%;
  right: 0;
  bottom: 25%;
  width: 1px;
  background: color-mix(in srgb, var(--text-muted) 20%, transparent);
  content: '';
  pointer-events: none;
  transform: scaleX(.5);
}

.market-dock-item {
  display: inline-flex;
  width: 95%;
  height: 100%;
  min-width: 0;
  align-items: center;
  justify-content: center;
  gap: 2px;
  margin: 0;
  padding: 0;
  border: 0;
  border-radius: 10px 10px 0 0;
  color: color-mix(in srgb, var(--brand-text) 60%, var(--surface-raised));
  background: transparent;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  box-shadow: none;
  transition: color 140ms ease, background 140ms ease;
}

.market-dock-item:not(.active):hover {
  color: var(--text);
  background: color-mix(in srgb, var(--brand-text) 3%, transparent);
}

.market-dock-item.active {
  color: var(--text);
  background: color-mix(in srgb, var(--brand-text) 6%, transparent);
}

.market-dock-item span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.market-dock-item:focus-visible {
  outline-offset: -2px;
}
</style>
