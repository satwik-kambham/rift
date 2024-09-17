<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '../stores/settings.ts'
import { useWorkspaceStore } from '../stores/workspace.ts'

const settingsStore = useSettingsStore();
const workspaceStore = useWorkspaceStore();

const panel = ref<HTMLELement | null>(null)

onMounted(() => {
  const dimensions = calculateCapacity();
  workspaceStore.debug = "Width: " + dimensions.width + ", Height: " + dimensions.height;
})

function calculateCapacity() {
  const width = panel.value?.offsetWidth;
  const height = panel.value?.offsetHeight;
  return { width: width, height: height }
}
</script>

<template>
  <div ref="panel" class="flex-grow bg-stone-800">Editor</div>
</template>
