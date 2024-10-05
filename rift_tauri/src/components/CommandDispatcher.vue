<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '../stores/settings.ts'
import { useWorkspaceStore } from '../stores/workspace.ts'
import { invoke } from '@tauri-apps/api'

const settingsStore = useSettingsStore()
const workspaceStore = useWorkspaceStore()

const keyCode = ref('')
const hiddenInput = ref<HTMLELement | null>(null)

function getVisibleLines() {
  invoke('get_visible_lines', { bufferId: workspaceStore.bufferId }).then((result) => {
    workspaceStore.visibleLines = result[0]
    console.log(result[0])
    workspaceStore.relativeCursorRow = result[1].row
    workspaceStore.relativeCursorColumn = result[1].column
    workspaceStore.cursorRow = result[2].row
    workspaceStore.cursorColumn = result[2].column
    workspaceStore.gutterInfo = result[3]
  })
}

function key_down(e: KeyboardEvent) {
  e.preventDefault()
  keyCode.value = e.key
  if (e.key == 'F1') {
    invoke('open_file', { path: '/home/satwik/Documents/test.rs' }).then((bufferId) => {
      workspaceStore.bufferId = bufferId
      getVisibleLines()
    })
  } else if (e.key == 'ArrowRight') {
    invoke('move_cursor_right', { bufferId: workspaceStore.bufferId }).then(getVisibleLines())
  } else if (e.key == 'ArrowLeft') {
    invoke('move_cursor_left', { bufferId: workspaceStore.bufferId }).then(getVisibleLines())
  } else if (e.key == 'ArrowDown') {
    invoke('move_cursor_down', { bufferId: workspaceStore.bufferId }).then(getVisibleLines())
  } else if (e.key == 'ArrowUp') {
    invoke('move_cursor_up', { bufferId: workspaceStore.bufferId }).then(getVisibleLines())
  } else if (e.key == 'Backspace') {
    invoke('remove_text', { bufferId: workspaceStore.bufferId }).then(getVisibleLines())
  } else if (e.key == 'Enter') {
    invoke('insert_text', { bufferId: workspaceStore.bufferId, text: '\n' }).then(getVisibleLines())
  } else {
    invoke('insert_text', { bufferId: workspaceStore.bufferId, text: e.key }).then(getVisibleLines())
  }
}

function focusInput() {
  hiddenInput.value?.focus()
}

onMounted(() => {
  focusInput()
})
</script>

<template>
  <div class="flex bg-bg-dark px-1">
    <div class="">Dispatcher: {{ keyCode }}</div>
    <input
      ref="hiddenInput"
      class="opacity-0 w-0 h-0"
      tabindex="-1"
      type="text"
      @keydown="key_down"
      @blur="focusInput"
    />
  </div>
</template>
