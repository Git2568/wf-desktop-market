import { nextTick } from 'vue'

const tokens = [
  '--canvas', '--surface', '--surface-raised', '--surface-soft', '--text',
  '--text-muted', '--text-soft', '--primary', '--primary-soft', '--primary-wash',
  '--positive', '--positive-wash', '--negative', '--negative-wash', '--border',
  '--shadow-color', '--brand-accent', '--brand-text', '--nav-active-text',
  '--nav-active-background', '--primary-button-text',
]
const duration = 240
const easing = 'cubic-bezier(.2,0,0,1)'
let currentAnimation: Animation | null = null
let generation = 0
let registered = false

function registerTokens() {
  if (registered) return true
  try {
    if (typeof CSS === 'undefined' || typeof CSS.registerProperty !== 'function') return false
    for (const name of tokens) {
      try {
        CSS.registerProperty({ name, syntax: '<color>', inherits: true, initialValue: 'transparent' })
      } catch (error) {
        if (!(error instanceof DOMException) || error.name !== 'InvalidModificationError') return false
      }
    }
    registered = true
    return true
  } catch {
    return false
  }
}

function sample(element: HTMLElement) {
  try {
    const computed = getComputedStyle(element)
    return Object.fromEntries(tokens.map(token => [token, computed.getPropertyValue(token).trim()]))
  } catch {
    return null
  }
}

function cancelCurrent() {
  try { currentAnimation?.cancel() } catch { /* animation support is optional */ }
  currentAnimation = null
}

export async function transitionTheme(applyTheme: () => void) {
  const applied = { value: false }
  const applyOnce = () => {
    if (applied.value) return
    applied.value = true
    try { applyTheme() } catch { /* the store remains responsible for its own state */ }
  }

  try {
    const element = document.documentElement
    const currentGeneration = ++generation
    const oldValues = sample(element)
    cancelCurrent()
    applyOnce()
    await nextTick()
    if (currentGeneration !== generation) return
    const targetValues = sample(element)
    if (!oldValues || !targetValues) return
    const media = window.matchMedia('(prefers-reduced-motion: reduce)')
    if (media.matches || document.hidden || oldValues['--canvas'] === targetValues['--canvas']) return
    if (!registerTokens() || typeof element.animate !== 'function') return

    let localAnimation: Animation | null = null
    const stop = () => {
      if (currentGeneration !== generation) return
      try { localAnimation?.cancel() } catch { /* animation support is optional */ }
      if (currentAnimation === localAnimation) currentAnimation = null
    }
    const onReducedMotion = () => { if (media.matches) stop() }
    const onVisibility = () => { if (document.hidden) stop() }
    let mediaListening = false
    let visibilityListening = false
    try {
      media.addEventListener?.('change', onReducedMotion)
      mediaListening = true
      document.addEventListener('visibilitychange', onVisibility)
      visibilityListening = true
      localAnimation = element.animate([oldValues, targetValues], {
        duration,
        easing,
        fill: 'none',
        id: 'wm-theme-transition',
      })
      currentAnimation = localAnimation
      await localAnimation.finished.catch(() => undefined)
    } catch {
      // The theme value was already applied; animation failure is non-fatal.
    } finally {
      if (mediaListening) media.removeEventListener?.('change', onReducedMotion)
      if (visibilityListening) document.removeEventListener('visibilitychange', onVisibility)
      if (currentGeneration === generation && currentAnimation === localAnimation) currentAnimation = null
    }
  } catch {
    applyOnce()
  }
}

const hot = (import.meta as ImportMeta & { hot?: { dispose: (callback: () => void) => void } }).hot
hot?.dispose(() => {
  generation++
  cancelCurrent()
})
