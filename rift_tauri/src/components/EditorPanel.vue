<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '../stores/settings.ts'
import { useWorkspaceStore } from '../stores/workspace.ts'
import { invoke } from '@tauri-apps/api'

const settingsStore = useSettingsStore()
const workspaceStore = useWorkspaceStore()

const panel = ref<HTMLELement | null>(null)
const hiddenLine = ref<HTMLELement | null>(null)
const gutterWidth = ref(80)
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
    class="flex flex-grow bg-bg antialiased w-full text-text cursor-text select-none relative"
    :style="{
      'font-size': settingsStore.fontSize + 'px',
      'font-family': settingsStore.fontFamily,
      'line-height': settingsStore.lineHeight
    }"
  >
    <div
      class="bg-bg-dark text-text-light"
      :style="{
        width: gutterWidth + 'px'
      }"
    >
      <div v-for="line in workspaceStore.gutterInfo">
        <span class="inline-block whitespace-pre">
          <span v-if="line.wrapped" class="whitespace-pre px-4">.</span>
          <span v-else class="whitespace-pre px-4">{{ line.start.row }}</span>
        </span>
      </div>
    </div>
    <div class="bg-bg flex-grow">
      <div v-for="line in workspaceStore.visibleLines">
        <span class="inline-block whitespace-pre">
          <span
            v-for="token in line"
            :class="{
              'whitespace-pre text-highlight-Red': token[1] != null,
              'whitespace-pre': token[1] == null
            }"
            >{{ token[0] }}</span
          >
        </span>
      </div>
      <div
        class="absolute pointer-events-none z-10 bg-primary opacity-30"
        :style="{
          top: workspaceStore.relativeCursorRow * lineHeight + 'px',
          left: workspaceStore.relativeCursorColumn * characterWidth + gutterWidth + 'px',
          width: characterWidth + 'px',
          height: lineHeight + 'px'
        }"
      ></div>
      <div ref="hiddenLine" class="absolute invisible whitespace-pre inline-block">X</div>
    </div>
  </div>
</template>
