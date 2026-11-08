<script setup lang="ts">
import { HugeiconsIcon } from '@hugeicons/vue'
import { icons } from '../icons'
import { useUiStore, type ModuleId } from '../stores/ui'
import { nextTick, onMounted, onUnmounted, ref } from 'vue'

defineProps<{ selectedModule: ModuleId }>()
const emit = defineEmits<{ select: [ModuleId] }>()
const ui = useUiStore()
const menuOpen = ref(false)
const menuRef = ref<HTMLElement | null>(null)
const triggerRef = ref<HTMLButtonElement | null>(null)
const items: Array<{ id: ModuleId; label: string }> = [
  { id: 'market', label: '实时紫卡市场' },
  { id: 'analysis', label: '紫卡词条分析' },
  { id: 'reroll', label: '紫卡洗卡指导' },
  { id: 'loadout', label: '配装模拟和保存' },
]

async function openMenu() { menuOpen.value = !menuOpen.value; if (menuOpen.value) { await nextTick(); menuRef.value?.querySelector<HTMLButtonElement>('[role="menuitem"]')?.focus() } }
function closeMenu() { menuOpen.value = false; triggerRef.value?.focus() }
function handleOutside(event: MouseEvent) {
  if (menuOpen.value && menuRef.value && !menuRef.value.contains(event.target as Node)) closeMenu()
}
function handleKeydown(event: KeyboardEvent) {
  if (!menuOpen.value) return
  const options = Array.from(menuRef.value?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? [])
  const index = options.indexOf(document.activeElement as HTMLButtonElement)
  if (event.key === 'Escape') { event.preventDefault(); closeMenu() }
  if (event.key === 'ArrowUp' || event.key === 'ArrowDown') { event.preventDefault(); options[(index + (event.key === 'ArrowUp' ? -1 : 1) + options.length) % options.length]?.focus() }
  if (event.key === 'Home') { event.preventDefault(); options[0]?.focus() }
  if (event.key === 'End') { event.preventDefault(); options.at(-1)?.focus() }
}
function openSettings() { menuOpen.value = false; ui.isSettingsOpen = true }
function openAbout() { menuOpen.value = false; ui.isAboutOpen = true }
onMounted(() => { document.addEventListener('click', handleOutside); document.addEventListener('keydown', handleKeydown) })
onUnmounted(() => { document.removeEventListener('click', handleOutside); document.removeEventListener('keydown', handleKeydown) })
</script>

<template>
  <aside class="sidebar" :class="{ collapsed: ui.sidebarCollapsed }">
    <div class="sidebar-top" data-tauri-drag-region>
      <div class="brand" aria-label="WM Market" data-tauri-drag-region><span data-tauri-drag-region>W</span><span data-tauri-drag-region>M</span></div>
    </div>
    <button class="collapse-button" type="button" :aria-label="ui.sidebarCollapsed ? '展开侧栏' : '收拢侧栏'" :aria-expanded="!ui.sidebarCollapsed" :title="ui.sidebarCollapsed ? '展开侧栏' : '收拢侧栏'" @click="ui.toggleSidebar()">
      <HugeiconsIcon :icon="ui.sidebarCollapsed ? icons.expand : icons.collapse" :size="15" color="currentColor" :stroke-width="1.6" />
    </button>
    <nav class="nav-list" aria-label="主导航">
      <button v-for="item in items" :key="item.id" class="nav-item" :class="{ active: selectedModule === item.id }" type="button" :aria-label="item.label" :aria-current="selectedModule === item.id ? 'page' : undefined" :title="ui.sidebarCollapsed ? item.label : undefined" @click="emit('select', item.id)">
        <HugeiconsIcon :icon="icons[item.id]" :size="20" color="currentColor" :stroke-width="1.65" /><span>{{ item.label }}</span>
      </button>
    </nav>
    <div ref="menuRef" class="sidebar-bottom">
      <div v-if="menuOpen" class="sidebar-menu quick-menu" role="menu">
        <button type="button" role="menuitem" @click="openSettings"><HugeiconsIcon :icon="icons.settings" :size="17" color="currentColor" :stroke-width="1.7" />设置</button>
        <button type="button" role="menuitem" @click="openAbout"><HugeiconsIcon :icon="icons.about" :size="17" color="currentColor" :stroke-width="1.7" />关于</button>
      </div>
      <button id="sidebar-settings-trigger" ref="triggerRef" class="sidebar-settings" type="button" aria-label="打开设置菜单" title="打开设置菜单" :aria-expanded="menuOpen" @click.stop="openMenu"><HugeiconsIcon :icon="icons.settings" :size="20" color="currentColor" :stroke-width="1.65" /><span>设置</span></button>
    </div>
  </aside>
</template>
