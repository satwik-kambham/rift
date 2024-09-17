<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '../stores/settings.ts'
import { useWorkspaceStore } from '../stores/workspace.ts'
import { invoke } from '@tauri-apps/api'

const settingsStore = useSettingsStore()
const workspaceStore = useWorkspaceStore()

const panel = ref<HTMLELement | null>(null)
const hiddenLine = ref<HTMLELement | null>(null)

onMounted(() => {
  const visibleLines = calculateCapacity()
  invoke('panel_resized', { visibleLines: visibleLines })
})

function calculateCapacity() {
  const height = panel.value?.offsetHeight
  const lineHeight = hiddenLine.value?.offsetHeight
  const visibleLines = parseInt(height / lineHeight)
  return visibleLines
}
</script>

<template>
  <div
    ref="panel"
    class="flex-grow bg-stone-800 m-2 antialiased"
    :style="{
      'font-size': settingsStore.fontSize + 'px',
      'font-family': settingsStore.fontFamily,
      'line-height': settingsStore.lineHeight + 'rem'
    }"
  >
    <span v-for="line in workspaceStore.visibleLines" class="whitespace-pre inline-block">
      <span class="whitespace-pre">{{ line }}</span>
    </span>
    <div ref="hiddenLine" class="absolute invisible whitespace-pre inline-block">X</div>
  </div>
</template>
