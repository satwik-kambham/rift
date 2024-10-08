# TODO

o - Started / - Partial x - Done

- [x] Line buffer from text
- [x] Line buffer to text for saving to file
- [x] Read file contents from path
- [x] Add tailwind
- [x] Persistent editor settings (font family, font size, tab width, line endings)
- [x] Add basic file handling methods
- [x] Create component which is always in focus and handles all keyboard inputs
- [ ] Create completion component which shows all completions
- [x] Add state management to rust
- [ ] Create commands for file handling methods
- [x] Add frontend state for current buffer id
- [x] Create editor panel component which is bound to the current buffer id
- [x] Add simple tree sitter based syntax highlighting
- [x] Add methods and commands to initialize the editor panel by calculating the number of visible lines based on a dummy element
- [x] Add cursor, selection and scroll position (line number and percentage distance from the top) structs
- [x] Display syntax highlighted text
- [/] Word wrap (soft wrap and hard wrap)
- [/] Display cursor, selection
- [/] Add backend methods to insert and delete selections with tests
- [o] Add backend methods for undo and redo
- [ ] Add edit grouping
- [ ] Add methods and commands for changes to the number of visible lines
- [-] Add backend methods to calculate the lines that need to be replaced after an edit or change
- [x] Add scrolling functionality
- [/] Add editing functinality to the frontend
- [o] Add async queue to ensure commands run in order
