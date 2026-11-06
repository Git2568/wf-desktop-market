<script setup lang="ts">
import { HugeiconsIcon } from '@hugeicons/vue'

defineProps<{ icon: any; title: string; description: string; compact?: boolean }>()
</script>

<template>
  <div class="empty-state" :class="{ compact }">
    <div class="empty-icon"><HugeiconsIcon :icon="icon" :size="compact ? 25 : 30" color="currentColor" :stroke-width="1.45" /></div>
    <h2>{{ title }}</h2>
    <p>{{ description }}</p>
  </div>
</template>
