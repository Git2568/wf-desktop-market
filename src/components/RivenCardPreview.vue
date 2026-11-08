<script setup lang="ts">
import legendaryBackground from '../assets/riven/LegendaryBackground.png'
import frameTop from '../assets/riven/OmegaFrameTop.png'
import frameBottom from '../assets/riven/OmegaFrameBottom.png'
import sideLight from '../assets/riven/OmegaSideLight.png'
import cornerLights from '../assets/riven/OmegaCornerLights.png'
import topBacker from '../assets/riven/RivenTopRightBacker.png'
import lowerTab from '../assets/riven/RivenLowerTab.png'

defineProps<{
  title: string
  subtitle?: string
  positives: string[]
  negatives: string[]
}>()
</script>

<template>
  <div class="riven-card-preview" aria-label="紫卡预览">
    <div class="riven-card-stage">
      <img class="riven-layer riven-background" :src="legendaryBackground" alt="" aria-hidden="true" />
      <img class="riven-layer riven-top" :src="frameTop" alt="" aria-hidden="true" />
      <img class="riven-layer riven-bottom" :src="frameBottom" alt="" aria-hidden="true" />
      <img class="riven-layer riven-side riven-side-right" :src="sideLight" alt="" aria-hidden="true" />
      <img class="riven-layer riven-side riven-side-left" :src="sideLight" alt="" aria-hidden="true" />
      <img class="riven-layer riven-backer" :src="topBacker" alt="" aria-hidden="true" />
      <img class="riven-layer riven-tab" :src="lowerTab" alt="" aria-hidden="true" />
      <img class="riven-layer riven-corner riven-corner-right" :src="cornerLights" alt="" aria-hidden="true" />
      <img class="riven-layer riven-corner riven-corner-left" :src="cornerLights" alt="" aria-hidden="true" />
      <div class="riven-card-copy">
        <strong>{{ title }}</strong>
        <span v-if="subtitle" class="riven-card-subtitle">{{ subtitle }}</span>
        <div class="riven-card-stats">
          <span v-for="(stat, index) in positives" :key="`positive-${index}`" class="positive">{{ stat }}</span>
          <span v-for="(stat, index) in negatives" :key="`negative-${index}`" class="negative">{{ stat }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.riven-card-preview{display:flex;justify-content:center;width:100%;aspect-ratio:292 / 380;max-width:250px;margin:0 auto;overflow:hidden}
.riven-card-stage{position:relative;width:100%;height:100%;overflow:hidden}
.riven-layer{position:absolute;display:block;max-width:none;pointer-events:none;user-select:none}
.riven-background{left:6.1644%;top:-17.3684%;width:87.6712%;height:134.7368%}
.riven-top{left:3.9384%;top:1.4947%;width:96.2329%;height:30.7895%}
.riven-bottom{left:.6849%;top:70.2105%;width:100%;height:31.5789%}
.riven-side{top:10.9263%;width:5.4795%;height:67.3684%}
.riven-side-right{left:87.6986%}
.riven-side-left{left:6.1644%;transform:scaleX(-1)}
.riven-backer{left:76.3014%;top:9.5789%;width:15.411%;height:6.8421%}
.riven-tab{left:14.0548%;top:81.5789%;width:71.9178%;height:6.8421%}
.riven-corner{top:78.7368%;width:21.9178%;height:16.8421%}
.riven-corner-right{left:73.6849%;}
.riven-corner-left{left:4.6849%;transform:scaleX(-1)}
.riven-card-copy{position:absolute;inset:0 12% 0;display:flex;flex-direction:column;align-items:center;text-align:center;color:#fff;text-shadow:0 1px 2px #12071b;z-index:2;padding-top:25%;font-size:clamp(10px,1.8vw,13px);line-height:1.25}
.riven-card-copy strong{max-width:100%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:1.1em}
.riven-card-subtitle{margin-top:2px;font-size:.8em;opacity:.85}
.riven-card-stats{display:grid;gap:4px;margin-top:22%;max-width:100%;font-size:.88em}
.riven-card-stats .positive{color:#8ef09b}.riven-card-stats .negative{color:#ff9b9b}
</style>
