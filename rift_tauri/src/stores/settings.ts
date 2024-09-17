import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useSettingsStore = defineStore('settings', () => {
  const fontSize = ref(18)
  const fontFamily = ref('consolas, monospace')
  const lineHeight = ref(1)

  return { fontSize, fontFamily, lineHeight }
})
