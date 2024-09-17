<script setup lang="ts">
import EditorPanel from './components/EditorPanel.vue'
import StatusBar from './components/StatusBar.vue'
import CommandDispatcher from './components/CommandDispatcher.vue'
</script>

<template>
  <div class="flex flex-col bg-stone-800 text-stone-100 min-h-full">
    <EditorPanel />
    <StatusBar />
    <CommandDispatcher />
  </div>
</template>
