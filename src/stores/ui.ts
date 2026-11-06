import { computed, ref, watch } from 'vue'
import { defineStore } from 'pinia'

export type Theme = 'light' | 'dark'
export type ModuleId = 'market' | 'analysis' | 'reroll' | 'loadout'

export const useUiStore = defineStore('ui', () => {
  let storedTheme: string | null = null
  let storedCollapsed: string | null = null
  try {
    storedTheme = localStorage.getItem('wm-theme')
    storedCollapsed = localStorage.getItem('wm-sidebar-collapsed')
  } catch { /* use system defaults when storage is unavailable */ }
  const safeTheme: Theme = storedTheme === 'light' || storedTheme === 'dark' ? storedTheme : (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
  const theme = ref<Theme>(safeTheme)
  const sidebarCollapsed = ref(storedCollapsed === 'true')
  const selectedModule = ref<ModuleId>('market')
  const isSettingsOpen = ref(false)
  const isAboutOpen = ref(false)

  const isDark = computed(() => theme.value === 'dark')

  function setTheme(next: Theme) {
    theme.value = next
  }

  function toggleTheme() {
    setTheme(theme.value === 'light' ? 'dark' : 'light')
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  watch(theme, (value) => {
    try { localStorage.setItem('wm-theme', value) } catch { /* preferences remain in memory */ }
  }, { immediate: true })

  watch(sidebarCollapsed, (value) => { try { localStorage.setItem('wm-sidebar-collapsed', String(value)) } catch { /* preferences remain in memory */ } }, { immediate: true })

  return { theme, isDark, sidebarCollapsed, selectedModule, isSettingsOpen, isAboutOpen, setTheme, toggleTheme, toggleSidebar }
})
