<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '../stores/settings.ts'
import { useWorkspaceStore } from '../stores/workspace.ts'
import { invoke } from '@tauri-apps/api'

const settingsStore = useSettingsStore()
const workspaceStore = useWorkspaceStore()

const keyCode = ref('')
const hiddenInput = ref<HTMLELement | null>(null)

function key_down(e: KeyboardEvent) {
  e.preventDefault()
  keyCode.value = e.key
  if (e.key == 'f') {
    invoke('open_file', { path: '/home/satwik/Documents/test.py' }).then((bufferId) => {
      workspaceStore.bufferId = bufferId
      workspaceStore.debug = bufferId
      invoke('get_visible_lines_wrap', { bufferId: bufferId }).then((visibleLines) => {
        workspaceStore.visibleLines = visibleLines
      })
    })
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
  <div class="flex">
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
