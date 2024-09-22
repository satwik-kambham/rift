import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useSettingsStore = defineStore('settings', () => {
  const fontSize = ref(26)
  const fontFamily = ref('Monaspace Neon, consolas, monospace')
  const lineHeight = ref(1.5)

  return { fontSize, fontFamily, lineHeight }
})
