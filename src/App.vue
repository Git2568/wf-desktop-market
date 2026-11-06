<script setup lang="ts">
import { computed } from 'vue'
import { HugeiconsIcon } from '@hugeicons/vue'
import Sidebar from './components/Sidebar.vue'
import AppDialog from './components/AppDialog.vue'
import EmptyState from './components/EmptyState.vue'
import { icons } from './icons'
import { useUiStore, type ModuleId, type Theme } from './stores/ui'

const ui = useUiStore()

const modules: Record<ModuleId, { eyebrow: string; title: string; description: string; emptyTitle: string; emptyDescription: string }> = {
  market: { eyebrow: '实时紫卡市场', title: '实时紫卡市场', description: '关注正在变化的紫卡行情。', emptyTitle: '暂无行情', emptyDescription: '连接后显示市场信息。' },
  analysis: { eyebrow: '紫卡词条分析', title: '紫卡词条分析', description: '把词条放回具体武器与用途里理解。', emptyTitle: '等待一张紫卡', emptyDescription: '选择紫卡后，这里会展示词条与热门成品的对比。' },
  reroll: { eyebrow: '紫卡洗卡指导', title: '紫卡洗卡指导', description: '围绕目标用途梳理洗卡方向。', emptyTitle: '还没有洗卡目标', emptyDescription: '准备好紫卡和目标用途后开始指导。' },
  loadout: { eyebrow: '配装模拟和保存', title: '配装模拟和保存', description: '整理配装思路，保留自己的搭配。', emptyTitle: '还没有配装', emptyDescription: '创建一个配装后，它会出现在这里。' },
}

const current = computed(() => modules[ui.selectedModule])
const themeIcon = computed(() => ui.isDark ? icons.light : icons.dark)
const themeLabel = computed(() => ui.isDark ? '切换到亮色主题' : '切换到暗色主题')

function selectModule(id: ModuleId) {
  ui.selectedModule = id
}

</script>

<template>
  <div class="app-frame" :class="{ 'is-dark': ui.isDark }">
    <Sidebar :selected-module="ui.selectedModule" @select="selectModule" />

    <main class="main-area">
      <header class="topbar">
        <div class="topbar-context">
          <span class="context-dot" aria-hidden="true"></span>
          <span>WM Market</span>
        </div>
        <div class="topbar-actions">
          <button class="icon-button theme-button" type="button" :aria-label="themeLabel" :title="themeLabel" @click="ui.toggleTheme()">
            <HugeiconsIcon :icon="themeIcon" :size="19" color="currentColor" :stroke-width="1.7" />
          </button>
        </div>
      </header>

      <section class="content-scroll">
        <div class="content-wrap">
          <div class="page-heading">
            <div>
              <h1>{{ current.title }}</h1>
              <p class="page-description">{{ current.description }}</p>
            </div>
          </div>

          <div v-if="ui.selectedModule === 'market'" class="market-board">
            <div class="market-toolbar">
              <div class="toolbar-heading"><span>市场列表</span></div>
            </div>
            <div class="market-table" aria-label="市场列表预览">
              <div class="table-row table-header"><span>紫卡</span><span>词条</span><span>价格</span><span>状态</span></div>
              <div class="table-empty"><EmptyState :icon="icons.market" title="暂无行情" description="连接后显示市场信息。" compact /></div>
            </div>
          </div>
          <div v-else class="module-empty"><EmptyState :icon="icons[ui.selectedModule]" :title="current.emptyTitle" :description="current.emptyDescription" /></div>
        </div>
      </section>
    </main>

    <AppDialog v-model="ui.isSettingsOpen" title="设置" labelledby="settings-title" restore-selector="#sidebar-settings-trigger">
      <div class="dialog-section">
        <p class="dialog-label">外观</p>
        <div class="theme-options" role="group" aria-label="选择主题">
          <button type="button" :class="{ active: ui.theme === 'light' }" :aria-pressed="ui.theme === 'light'" @click="ui.setTheme('light' as Theme)"><HugeiconsIcon :icon="icons.light" :size="18" color="currentColor" :stroke-width="1.7" />亮色</button>
          <button type="button" :class="{ active: ui.theme === 'dark' }" :aria-pressed="ui.theme === 'dark'" @click="ui.setTheme('dark' as Theme)"><HugeiconsIcon :icon="icons.dark" :size="18" color="currentColor" :stroke-width="1.7" />暗色</button>
        </div>
      </div>
    </AppDialog>

    <AppDialog v-model="ui.isAboutOpen" title="关于 WM" labelledby="about-title" restore-selector="#sidebar-settings-trigger">
      <div class="about-copy"><div class="about-mark"><span>W</span><span>M</span></div><div><strong>WM Market</strong><p>紫卡市场工作台</p></div></div>
      <p class="version-line">版本 0.1.0</p>
    </AppDialog>
  </div>
</template>
