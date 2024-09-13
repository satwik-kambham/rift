import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useCounterStore = defineStore('workspace', () => {
  const count = ref(0)

  return { count }
})
