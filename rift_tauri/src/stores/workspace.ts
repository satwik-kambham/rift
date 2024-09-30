import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useWorkspaceStore = defineStore('workspace', () => {
  const debug = ref('Debug')
  const bufferId = ref(-1)
  const visibleLines = ref([])
  const gutterInfo = ref([])
  const startLine = ref(0)
  const relativeCursorRow = ref(0)
  const relativeCursorColumn = ref(0)
  const cursorRow = ref(0)
  const cursorColumn = ref(0)

  return {
    debug,
    bufferId,
    visibleLines,
    relativeCursorRow,
    relativeCursorColumn,
    cursorRow,
    cursorColumn,
    gutterInfo
  }
})
