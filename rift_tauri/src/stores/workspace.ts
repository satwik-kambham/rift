import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useWorkspaceStore = defineStore('workspace', () => {
  const debug = ref('Debug')
  const bufferId = ref(-1)
  const visibleLines = ref([])

  return { debug, bufferId, visibleLines }
})
