<script setup lang="ts">
import { ref, onMounted } from 'vue'

const keyCode = ref('sdfsdf')
const hiddenInput = ref<HTMLELement | null>(null)

function key_down(e: KeyboardEvent) {
  e.preventDefault()
  keyCode.value = e.key
}

function focusInput() {
  hiddenInput.value?.focus()
}

onMounted(() => {
  focusInput();
})
</script>

<template>
  <div class="underline">Dispatcher: {{ keyCode }}</div>
  <input ref="hiddenInput" class="bg-slate-100" tabindex="-1" type="text" @keydown="key_down" @blur="focusInput" />
</template>
