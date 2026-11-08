import test from 'node:test'
import assert from 'node:assert/strict'

const tokens = [
  '--canvas', '--surface', '--surface-raised', '--surface-soft', '--text',
  '--text-muted', '--text-soft', '--primary', '--primary-soft', '--primary-wash',
  '--positive', '--positive-wash', '--negative', '--negative-wash', '--border',
  '--shadow-color',
]

function makeEnvironment({ reduced = false, hidden = false, registerError = null } = {}) {
  const rootValues = Object.fromEntries(tokens.map((token) => [token, `light-${token}`]))
  const inlineValues = new Map()
  const inlineWrites = { set: 0, remove: 0 }
  const style = {
    setProperty(name, value) {
      inlineWrites.set += 1
      inlineValues.set(name, value)
    },
    getPropertyValue(name) {
      return inlineValues.get(name) ?? ''
    },
    removeProperty(name) {
      inlineWrites.remove += 1
      inlineValues.delete(name)
    },
  }
  const listeners = new Map()
  const trackedTarget = () => {
    const target = new EventTarget()
    target.addEventListener = (type, listener, options) => {
      if (!listeners.has(type)) listeners.set(type, new Set())
      listeners.get(type).add(listener)
      EventTarget.prototype.addEventListener.call(target, type, listener, options)
    }
    target.removeEventListener = (type, listener, options) => {
      listeners.get(type)?.delete(listener)
      EventTarget.prototype.removeEventListener.call(target, type, listener, options)
    }
    return target
  }
  const root = trackedTarget()
  root.style = style
  const documentObject = trackedTarget()
  documentObject.documentElement = root
  documentObject.hidden = hidden
  documentObject.createElement = () => ({})
  const media = trackedTarget()
  media.matches = reduced
  const animations = []
  const unrelatedAnimation = { cancelCount: 0, cancel() { this.cancelCount += 1 } }
  let activeAnimation = null
  let sampleError = false

  root.animate = (keyframes) => {
    let resolveFinished
    let rejectFinished
    const finished = new Promise((resolve, reject) => {
      resolveFinished = resolve
      rejectFinished = reject
    })
    const animation = {
      keyframes,
      currentValues: { ...keyframes[0] },
      finished,
      cancelCount: 0,
      cancel() {
        this.cancelCount += 1
        if (activeAnimation === this) activeAnimation = null
        rejectFinished(new Error('cancelled'))
      },
      finish() {
        if (activeAnimation === this) activeAnimation = null
        resolveFinished()
      },
    }
    animations.push(animation)
    activeAnimation = animation
    return animation
  }
  documentObject.getAnimations = () => [unrelatedAnimation, ...animations]

  const previous = new Map([
    ['document', globalThis.document],
    ['window', globalThis.window],
    ['CSS', globalThis.CSS],
    ['getComputedStyle', globalThis.getComputedStyle],
  ])
  globalThis.document = documentObject
  globalThis.window = { matchMedia: () => media }
  globalThis.CSS = {
    registerProperty() {
      if (registerError) throw registerError
    },
  }
  globalThis.getComputedStyle = () => {
    if (sampleError) throw new Error('sample failed')
    return {
      getPropertyValue(name) {
        return activeAnimation?.currentValues[name] ?? rootValues[name] ?? `base-${name}`
      },
    }
  }
  inlineValues.set('--user-inline', 'keep')

  return {
    rootValues,
    inlineValues,
    inlineWrites,
    root,
    documentObject,
    media,
    animations,
    unrelatedAnimation,
    listeners,
    setTheme(kind) {
      this.requestedTheme = kind
      queueMicrotask(() => {
        for (const token of tokens) rootValues[token] = `${kind}-${token}`
      })
    },
    setSampleError(value) { sampleError = value },
    restore() {
      for (const animation of animations) animation.finish()
      for (const [name, value] of previous) {
        if (value === undefined) delete globalThis[name]
        else globalThis[name] = value
      }
    },
  }
}

let moduleCounter = 0
async function loadTransition() {
  const url = new URL('../src/ui/themeTransition.ts', import.meta.url)
  url.search = `case=${++moduleCounter}`
  return import(url.href)
}

async function flush() {
  await Promise.resolve()
  await Promise.resolve()
}

async function withEnvironment(options, callback) {
  const environment = makeEnvironment(options)
  try {
    return await callback(environment, await loadTransition())
  } finally {
    environment.restore()
  }
}

test('applies exactly once, completes, and never writes inline theme tokens', async () => {
  await withEnvironment({}, async (environment, transition) => {
    let calls = 0
    const pending = transition.transitionTheme(() => {
      calls += 1
      environment.setTheme('dark')
    })
    await flush()
    assert.equal(calls, 1)
    assert.equal(environment.animations.length, 1)
    environment.animations[0].finish()
    await pending
    assert.deepEqual(environment.inlineWrites, { set: 0, remove: 0 })
    assert.equal(environment.inlineValues.get('--user-inline'), 'keep')
    assert.equal(environment.rootValues['--canvas'], 'dark---canvas')
  })
})

test('rapid reverse starts at the current displayed color and preserves the latest animation', async () => {
  await withEnvironment({}, async (environment, transition) => {
    const first = transition.transitionTheme(() => environment.setTheme('dark'))
    await flush()
    const firstAnimation = environment.animations[0]
    firstAnimation.currentValues['--canvas'] = 'dark-mid---canvas'
    const second = transition.transitionTheme(() => environment.setTheme('light'))
    await flush()
    assert.equal(firstAnimation.cancelCount, 1)
    assert.equal(environment.animations.length, 2)
    const latest = environment.animations[1]
    assert.equal(latest.keyframes[0]['--canvas'], 'dark-mid---canvas')
    assert.equal(latest.cancelCount, 0)
    latest.finish()
    await Promise.all([first, second])
  })
})

test('same-tick requests let the final theme win without an obsolete animation', async () => {
  await withEnvironment({}, async (environment, transition) => {
    const first = transition.transitionTheme(() => environment.setTheme('dark'))
    const second = transition.transitionTheme(() => environment.setTheme('light'))
    await flush()
    for (const animation of environment.animations) animation.finish()
    await Promise.all([first, second])
    assert.equal(environment.animations.length, 0)
    assert.equal(environment.rootValues['--canvas'], 'light---canvas')
  })
})

test('reduced motion, hidden page, and missing animate fall back directly', async () => {
  await withEnvironment({ reduced: true }, async (environment, transition) => {
    await transition.transitionTheme(() => environment.setTheme('dark'))
    assert.equal(environment.animations.length, 0)
  })
  await withEnvironment({ hidden: true }, async (environment, transition) => {
    await transition.transitionTheme(() => environment.setTheme('dark'))
    assert.equal(environment.animations.length, 0)
  })
  await withEnvironment({}, async (environment, transition) => {
    environment.root.animate = undefined
    await transition.transitionTheme(() => environment.setTheme('dark'))
    assert.equal(environment.animations.length, 0)
  })
})

test('reduced-motion and visibility changes cancel the animation and remove listeners', async () => {
  await withEnvironment({}, async (environment, transition) => {
    const pending = transition.transitionTheme(() => environment.setTheme('dark'))
    await flush()
    const animation = environment.animations[0]
    environment.media.matches = true
    environment.media.dispatchEvent(new Event('change'))
    assert.equal(animation.cancelCount, 1)
    await pending
    assert.equal(environment.listeners.get('change')?.size ?? 0, 0)
    assert.equal(environment.listeners.get('visibilitychange')?.size ?? 0, 0)
  })
  await withEnvironment({}, async (environment, transition) => {
    const pending = transition.transitionTheme(() => environment.setTheme('dark'))
    await flush()
    const animation = environment.animations[0]
    environment.documentObject.hidden = true
    environment.documentObject.dispatchEvent(new Event('visibilitychange'))
    assert.equal(animation.cancelCount, 1)
    await pending
    assert.equal(environment.listeners.get('change')?.size ?? 0, 0)
    assert.equal(environment.listeners.get('visibilitychange')?.size ?? 0, 0)
  })
})

test('registration, sampling, and animate errors still apply once without rejecting', async () => {
  await withEnvironment({ registerError: new Error('registration failed') }, async (environment, transition) => {
    let calls = 0
    await assert.doesNotReject(transition.transitionTheme(() => {
      calls += 1
      environment.setTheme('dark')
    }))
    assert.equal(calls, 1)
  })
  await withEnvironment({}, async (environment, transition) => {
    environment.setSampleError(true)
    let calls = 0
    await assert.doesNotReject(transition.transitionTheme(() => {
      calls += 1
      environment.setTheme('dark')
    }))
    assert.equal(calls, 1)
  })
  await withEnvironment({}, async (environment, transition) => {
    environment.root.animate = () => { throw new Error('animate failed') }
    let calls = 0
    await assert.doesNotReject(transition.transitionTheme(() => {
      calls += 1
      environment.setTheme('dark')
    }))
    assert.equal(calls, 1)
  })
})

test('missing CSS API and InvalidModificationError remain non-fatal', async () => {
  await withEnvironment({}, async (environment, transition) => {
    globalThis.CSS = {}
    await transition.transitionTheme(() => environment.setTheme('dark'))
    assert.equal(environment.animations.length, 0)
  })
  await withEnvironment({ registerError: new DOMException('already registered', 'InvalidModificationError') }, async (environment, transition) => {
    const pending = transition.transitionTheme(() => environment.setTheme('dark'))
    await flush()
    assert.equal(environment.animations.length, 1)
    environment.animations[0].finish()
    await pending
  })
})

test('only the helper-owned animation is cancelled', async () => {
  await withEnvironment({}, async (environment, transition) => {
    const first = transition.transitionTheme(() => environment.setTheme('dark'))
    await flush()
    const themeAnimation = environment.animations[0]
    const second = transition.transitionTheme(() => environment.setTheme('light'))
    await flush()
    assert.equal(themeAnimation.cancelCount, 1)
    assert.equal(environment.unrelatedAnimation.cancelCount, 0)
    for (const animation of environment.animations) animation.finish()
    await Promise.all([first, second])
  })
})
