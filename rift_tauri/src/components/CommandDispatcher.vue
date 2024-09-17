<script setup lang="ts">
import { ref, onMounted } from 'vue'

const keyCode = ref('')
const hiddenInput = ref<HTMLELement | null>(null)

function key_down(e: KeyboardEvent) {
  e.preventDefault()
  keyCode.value = e.key
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
