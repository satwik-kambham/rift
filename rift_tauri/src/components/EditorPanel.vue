<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '../stores/settings.ts'
import { useWorkspaceStore } from '../stores/workspace.ts'
import { invoke } from '@tauri-apps/api'

const settingsStore = useSettingsStore()
const workspaceStore = useWorkspaceStore()

const panel = ref<HTMLELement | null>(null)
const hiddenLine = ref<HTMLELement | null>(null)
const gutterWidth = ref(40)
const characterWidth = ref(40)
const lineHeight = ref(40)

onMounted(() => {
  const capacity = calculateCapacity()
  calculateCharacterDimensions()
  invoke('panel_resized', capacity)
})

function calculateCharacterDimensions() {
  lineHeight.value = hiddenLine.value?.offsetHeight
  characterWidth.value = hiddenLine.value?.offsetWidth
}

function calculateCapacity() {
  const height = panel.value?.offsetHeight
  const width = panel.value?.offsetWidth
  const lineHeight = hiddenLine.value?.offsetHeight
  const characterWidth = hiddenLine.value?.offsetWidth
  const visibleLines = parseInt(height / lineHeight)
  const charactersPerLine = parseInt((width - gutterWidth.value) / characterWidth)
  return {
    visibleLines: visibleLines,
    charactersPerLine: charactersPerLine
  }
}
</script>

<template>
  <div
    ref="panel"
    class="flex-grow bg-bg antialiased w-full text-text cursor-text select-none relative"
    :style="{
      'font-size': settingsStore.fontSize + 'px',
      'font-family': settingsStore.fontFamily,
      'line-height': settingsStore.lineHeight
    }"
  >
    <div v-for="line in workspaceStore.visibleLines">
      <span class="inline-block whitespace-pre">
        <span class="whitespace-pre">{{ line }}</span>
      </span>
    </div>
    <div class="absolute pointer-events-none z-10 bg-primary opacity-30" :style="{
      top: workspaceStore.cursorRow * lineHeight + 'px',
      left: workspaceStore.cursorColumn * characterWidth + 'px',
      width: characterWidth + 'px',
      height: lineHeight + 'px',
    }"></div>
    <div ref="hiddenLine" class="absolute invisible whitespace-pre inline-block">X</div>
  </div>
</template>
