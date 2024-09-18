import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useWorkspaceStore = defineStore('workspace', () => {
  const debug = ref('Debug')
  const bufferId = ref(-1)
  const visibleLines = ref([])
  const startLine = ref(0)

  return { debug, bufferId, visibleLines, startLine }
})
