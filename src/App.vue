<script setup lang="ts">
import { computed } from 'vue'
import { HugeiconsIcon } from '@hugeicons/vue'
import Sidebar from './components/Sidebar.vue'
import AppDialog from './components/AppDialog.vue'
import EmptyState from './components/EmptyState.vue'
import MarketWorkspace from './components/MarketWorkspace.vue'
import MarketDock from './components/MarketDock.vue'
import DataSettings from './components/DataSettings.vue'
import RivenAnalysis from './components/RivenAnalysis.vue'
import RerollGuide from './components/RerollGuide.vue'
import WindowTitlebar from './components/WindowTitlebar.vue'
import { icons } from './icons'
import { useUiStore, type ModuleId, type Theme } from './stores/ui'

const ui = useUiStore()
const modules: Record<ModuleId, { title: string; description: string; emptyTitle: string; emptyDescription: string }> = {
  market: { title: '实时紫卡市场', description: '', emptyTitle: '暂无行情', emptyDescription: '设置条件后查询市场信息。' },
  analysis: { title: '紫卡词条分析', description: '把词条放回具体武器与用途里理解。', emptyTitle: '等待一张紫卡', emptyDescription: '选择紫卡后，这里会展示词条与热门成品的对比。' },
  reroll: { title: '紫卡洗卡指导', description: '围绕目标用途梳理洗卡方向。', emptyTitle: '还没有洗卡目标', emptyDescription: '准备好紫卡和目标用途后开始指导。' },
  loadout: { title: '配装模拟和保存', description: '整理配装思路，保留自己的搭配。', emptyTitle: '还没有配装', emptyDescription: '创建一个配装后，它会出现在这里。' },
}
const current = computed(() => modules[ui.selectedModule])
function selectModule(id: ModuleId) { ui.selectedModule = id }
</script>

<template>
  <div class="app-frame" :class="{ 'is-dark': ui.isDark }">
    <Sidebar :selected-module="ui.selectedModule" @select="selectModule" />
    <main class="main-area">
      <WindowTitlebar :is-dark="ui.isDark" @toggle-theme="ui.toggleTheme()" />
      <section class="content-scroll">
        <div class="content-wrap">
          <div v-if="ui.selectedModule !== 'market' && ui.selectedModule !== 'analysis' && ui.selectedModule !== 'reroll'" class="page-heading">
            <div><h1>{{ current.title }}</h1><p class="page-description">{{ current.description }}</p></div>
          </div>
          <MarketWorkspace v-show="ui.selectedModule === 'market'" />
          <RivenAnalysis v-show="ui.selectedModule === 'analysis'" />
          <RerollGuide v-show="ui.selectedModule === 'reroll'" />
          <div v-if="ui.selectedModule !== 'market' && ui.selectedModule !== 'analysis' && ui.selectedModule !== 'reroll'" class="module-empty">
            <EmptyState :icon="icons[ui.selectedModule]" :title="current.emptyTitle" :description="current.emptyDescription" />
          </div>
        </div>
      </section>
      <MarketDock v-if="ui.selectedModule === 'market'" v-model="ui.marketPageId" />
    </main>
    <AppDialog v-model="ui.isSettingsOpen" title="设置" labelledby="settings-title" restore-selector="#sidebar-settings-trigger">
      <div class="dialog-section">
        <p class="dialog-label">外观</p>
        <div class="theme-options" role="group" aria-label="选择主题">
          <button type="button" :class="{ active: ui.theme === 'light' }" :aria-pressed="ui.theme === 'light'" @click="ui.setTheme('light' as Theme)"><HugeiconsIcon :icon="icons.light" :size="18" color="currentColor" :stroke-width="1.7" />亮色</button>
          <button type="button" :class="{ active: ui.theme === 'dark' }" :aria-pressed="ui.theme === 'dark'" @click="ui.setTheme('dark' as Theme)"><HugeiconsIcon :icon="icons.dark" :size="18" color="currentColor" :stroke-width="1.7" />暗色</button>
        </div>
      </div>
      <DataSettings />
    </AppDialog>
    <AppDialog v-model="ui.isAboutOpen" title="关于 WM" labelledby="about-title" restore-selector="#sidebar-settings-trigger">
      <div class="about-copy"><div class="about-mark"><span>W</span><span>M</span></div><div><strong>WM Market</strong><p>紫卡市场工作台</p></div></div>
      <p class="version-line">版本 0.1.0</p>
    </AppDialog>
  </div>
</template>
