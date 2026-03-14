# `petal` Design Document

## Overview

`petal` is a Rust library crate providing a file-system backed store for
a note-taking application. Notes can contain mixed content: plain text, audio
recordings, images, and arbitrary file attachments. The format is intentionally
open and inspectable — no proprietary binary containers, no external database
required.

---

## Goals

- **Openness.** The on-disk format must be readable and writable by any tool
  that understands JSON and basic file I/O. No knowledge of a proprietary
  format should ever be required.
- **Self-contained portability.** A note is a plain directory. Copy it,
  zip it, email it — it moves cleanly with no orphaned data.
- **Simplicity of implementation.** The library should be implementable from
  scratch by a single developer working from this document alone.
- **Correct by default.** Timestamps are always set automatically. File paths
  are validated to prevent traversal. Mutations require an explicit `save_note`
  call, making persistence intentional.

## Non-Goals

- **Concurrent access.** The store does not support multiple processes or
  threads reading and writing simultaneously. No file locking is implemented.
- **Encryption or access control.** Security at rest is delegated to the OS or
  the caller.
- **Full-text search.** Querying note content is not provided; callers read
  notes and filter in-process, or build their own index on top.
- **Async I/O.** All operations are synchronous. An async wrapper can be added
  by the caller (e.g. `tokio::task::spawn_blocking`).

---

## On-Disk Format

### Directory Layout

```
<root>/
  notes/
    <note-uuid>/
      note.json
      recording.mp3
      photo.png
```

Every identifier is a UUID v4, used as both the in-memory identifier and the
directory name. Attachment files sit directly inside the note's directory,
referenced by filename.

### `note.json`

```json
{
  "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "title": null,
  "created_at": "2024-03-01T09:15:00Z",
  "updated_at": "2024-03-01T09:45:00Z",
  "blocks": [
    {
      "kind": "attachment",
      "file": "recording.mp3"
    },
    {
      "kind": "text",
      "label": "transcript",
      "content": "Recorded near the eastern bank just after sunrise..."
    },
    {
      "kind": "text",
      "label": "structured",
      "content": "## Observations\n\nRecorded near the eastern bank..."
    },
    {
      "kind": "text",
      "label": "actions",
      "content": "- Return at dusk to compare ambient sound levels."
    }
  ]
}
```

`title` is `null` when the note is first created and can be set later — for
example, auto-generated from the transcript by an LLM once it is available.
```

### Block Types

| `kind`       | Fields                                    | Notes                                          |
|--------------|-------------------------------------------|------------------------------------------------|
| `text`       | `label: Option<String>`, `content: String` | `label` is an application-defined tag (e.g. `"transcript"`, `"structured"`, `"actions"`). Omit for untagged text. MIME type is inferred from the file extension at read time — not stored. |
| `attachment` | `file: String`                            | Filename relative to the note's directory. MIME type is inferred from the file extension at read time — not stored. |

Blocks are stored as an ordered array. Position is implicit — reordering blocks
means reordering the array and calling `save_note`.

---

## API

The entire public surface is exposed through a single type, `NoteStore`.

### Store

```rust
fn NoteStore::open(root: impl AsRef<Path>) -> Result<NoteStore>
```
Opens an existing store or creates a new one at `root`. Creates the `notes/`
directory if it does not exist. Safe to call repeatedly on an existing store.

---

### Notes

```rust
fn create_note(&self) -> Result<Note>
```
Creates a new note with a `None` title and an empty block list and persists it
to disk. Assigns a new UUID v4 and sets both `created_at` and `updated_at` to
the current time.

---

```rust
fn get_note(&self, note_id: Uuid) -> Result<Note>
```
Loads and deserializes a note from disk by ID. Returns `StoreError::NoteNotFound`
if no note directory or `note.json` exists at that ID.

---

```rust
fn save_note(&self, note: Note) -> Result<Note>
```
Persists a note to disk, overwriting the existing `note.json`. Automatically
updates `updated_at` to the current time. Returns `StoreError::NoteNotFound`
if the note does not already exist on disk — use `create_note` for new notes.
Use this to set or update the title at any point after creation by mutating
`note.title` before calling `save_note`.

---

```rust
fn delete_note(&self, note_id: Uuid) -> Result<()>
```
Permanently removes the note's directory and all attachment files within it.
Returns `StoreError::NoteNotFound` if the note does not exist.

---

```rust
fn list_notes(&self) -> Result<Vec<Note>>
```
Returns all notes in the store, sorted by `created_at` ascending. Walks the
`notes/` directory and deserializes each `note.json`.

---

### Attachments

```rust
fn write_attachment(&self, note_id: Uuid, filename: &str, data: &[u8]) -> Result<Block>
```
Writes raw bytes to `filename` inside the note's directory and returns a
`Block::Attachment` referencing it. The caller is responsible for pushing the
returned block onto `note.blocks` and calling `save_note`. Returns
`StoreError::InvalidFilename` if `filename` contains `/`, `\`, or `..`.

---

```rust
fn read_attachment(&self, note_id: Uuid, filename: &str) -> Result<Vec<u8>>
```
Reads and returns the raw bytes of an attachment file. Returns
`StoreError::AttachmentNotFound` if the file does not exist in the note's
directory.

---

```rust
fn delete_attachment(&self, note_id: Uuid, filename: &str) -> Result<()>
```
Removes an attachment file from disk. The caller is responsible for also
removing the corresponding block from `note.blocks` and calling `save_note`.
Returns `StoreError::AttachmentNotFound` if the file does not exist.

---

## Data Models

```rust
pub struct Note {
    pub id: Uuid,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub blocks: Vec<Block>,
}

pub enum Block {
    Text       { label: Option<String>, content: String },
    Attachment { file: String },
}
```

MIME type is not stored. Callers infer it from the file extension at read time
using a crate such as `mime_guess`.

---

## Error Handling

All fallible operations return `Result<T, StoreError>`. The error type is
defined with `thiserror` and covers every failure mode explicitly:

| Variant                      | Cause                                          |
|------------------------------|------------------------------------------------|
| `NoteNotFound(Uuid)`         | No note directory or `note.json` at that ID    |
| `AttachmentNotFound(String)` | File does not exist in the note's directory    |
| `InvalidFilename(String)`    | Filename contains `/`, `\`, or `..`            |
| `Io(std::io::Error)`         | Any OS-level I/O failure                       |
| `Json(serde_json::Error)`    | Malformed JSON when reading a stored file      |

---

## Security Considerations

**Path traversal.** Attachment filenames provided by callers are validated
before use. Any filename containing `/`, `\`, or `..` is rejected with
`StoreError::InvalidFilename`. This prevents a malicious or buggy caller from
writing files outside the note's directory.

**No sandboxing.** The store does not sandbox itself. A caller with a valid
`NoteStore` handle can read and write anywhere the process has permission.
Access control is the caller's responsibility.

---

## Dependencies

| Crate          | Purpose                              |
|----------------|--------------------------------------|
| `serde`        | Derive `Serialize` / `Deserialize`   |
| `serde_json`   | JSON encoding and decoding           |
| `uuid`         | UUID v4 generation and serde support |
| `chrono`       | `DateTime<Utc>` with serde support   |
| `thiserror`    | Structured error type derivation     |

---

## Future Considerations

These are explicitly out of scope for now but worth keeping in mind as the
format is designed to accommodate them cleanly.

- **Streaming attachment I/O.** `read_attachment` and `write_attachment`
  currently load the entire file into memory. A `Read`/`Write` handle API
  would be more appropriate for large audio files.
- **Async variant.** A thin `AsyncNoteStore` wrapper using
  `tokio::task::spawn_blocking` around the synchronous core.
- **Note ordering.** Currently implicit via `created_at`. Could be made
  explicit by adding an `order` field to notes without breaking existing stores.
- **Note templates.** A `templates/` directory at the store root following the
  same `note.json` format, instantiated by a `create_note_from_template` call.
