<script setup lang="ts">
import { useSettingsStore } from '../stores/settings.ts'
import { useWorkspaceStore } from '../stores/workspace.ts'

const settingsStore = useSettingsStore();
const workspaceStore = useWorkspaceStore();
</script>

<template>
  <div class="p-1 bg-stone-700 flex">
    <div class="">Left</div>
    <div class="grow"></div>
    <div class="">{{ workspaceStore.debug }}</div>
    <div class="grow"></div>
    <div class="">Right</div>
  </div>
</template>
