<script setup lang="ts">
import { computed } from 'vue'
import { useUiStore } from '../stores/ui'
import MarketPage from './MarketPage.vue'

const ui = useUiStore()
const page = computed(() => ui.marketPageId)
</script>

<template>
  <div class="market-workspace">
    <div v-show="page === 'purchase'" class="market-workspace-page">
      <MarketPage />
    </div>
    <section v-if="page !== 'purchase'" class="market-workspace-empty" aria-live="polite">
      <h1>{{ page === 'wm-user' ? 'WM用户' : '我的上架' }}</h1>
      <p>暂无内容</p>
    </section>
  </div>
</template>

<style scoped>
.market-workspace {
  position: relative;
  min-width: 0;
  padding-bottom: 8px;
}

.market-workspace-page,
.market-workspace-empty {
  min-width: 0;
}

.market-workspace-empty {
  display: flex;
  min-height: 280px;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  padding: 24px 16px;
  border: 1px dashed var(--border);
  border-radius: 12px;
  background: color-mix(in srgb, var(--surface-raised) 52%, transparent);
  text-align: center;
}

.market-workspace-empty h1 {
  margin: 0;
}

.market-workspace-empty p {
  margin: 8px 0 0;
  color: var(--text-soft);
  font-size: 13px;
}
</style>
