<script setup lang="ts">
import { useSettingsStore } from '../stores/settings.ts'
import { useWorkspaceStore } from '../stores/workspace.ts'

const settingsStore = useSettingsStore()
const workspaceStore = useWorkspaceStore()
</script>

<template>
  <div class="p-1 bg-bg-dark flex border-t-[1px] border-black">
    <div class="">Left</div>
    <div class="grow"></div>
    <div class="">{{ workspaceStore.debug }}</div>
    <div class="grow"></div>
    <div class="">{{ workspaceStore.relativeCursorRow }} : {{ workspaceStore.relativeCursorColumn}}</div>
  </div>
</template>
