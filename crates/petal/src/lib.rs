use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("note not found: {0}")]
    NoteNotFound(Uuid),
    #[error("attachment not found: {0}")]
    AttachmentNotFound(String),
    #[error("invalid filename: {0}")]
    InvalidFilename(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, StoreError>;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Block {
    Text {
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        content: String,
    },
    Attachment {
        file: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: Uuid,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub blocks: Vec<Block>,
}

// ---------------------------------------------------------------------------
// NoteStore
// ---------------------------------------------------------------------------

pub struct NoteStore {
    root: PathBuf,
}

impl NoteStore {
    // -- Construction -------------------------------------------------------

    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join("notes"))?;
        Ok(Self { root })
    }

    // -- Notes --------------------------------------------------------------

    pub fn create_note(&self) -> Result<Note> {
        let now = Utc::now();
        let note = Note {
            id: Uuid::new_v4(),
            title: None,
            created_at: now,
            updated_at: now,
            blocks: Vec::new(),
        };
        fs::create_dir_all(self.note_dir(note.id))?;
        self.persist_note(&note)?;
        Ok(note)
    }

    pub fn get_note(&self, note_id: Uuid) -> Result<Note> {
        let path = self.note_json_path(note_id);
        let data = fs::read_to_string(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StoreError::NoteNotFound(note_id)
            } else {
                StoreError::Io(e)
            }
        })?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save_note(&self, mut note: Note) -> Result<Note> {
        if !self.note_dir(note.id).exists() {
            return Err(StoreError::NoteNotFound(note.id));
        }
        note.updated_at = Utc::now();
        self.persist_note(&note)?;
        Ok(note)
    }

    pub fn delete_note(&self, note_id: Uuid) -> Result<()> {
        let dir = self.note_dir(note_id);
        if !dir.exists() {
            return Err(StoreError::NoteNotFound(note_id));
        }
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let mut notes = Vec::new();
        for entry in fs::read_dir(self.notes_dir())? {
            let entry = entry?;
            let name = entry.file_name();
            if Uuid::parse_str(name.to_string_lossy().as_ref()).is_ok() {
                let id = Uuid::parse_str(name.to_string_lossy().as_ref()).unwrap();
                notes.push(self.get_note(id)?);
            }
        }
        notes.sort_by_key(|n| n.created_at);
        Ok(notes)
    }

    // -- Attachments --------------------------------------------------------

    pub fn write_attachment(&self, note_id: Uuid, filename: &str, data: &[u8]) -> Result<Block> {
        Self::validate_filename(filename)?;
        let dir = self.note_dir(note_id);
        if !dir.exists() {
            return Err(StoreError::NoteNotFound(note_id));
        }
        fs::write(dir.join(filename), data)?;
        Ok(Block::Attachment {
            file: filename.to_string(),
        })
    }

    pub fn read_attachment(&self, note_id: Uuid, filename: &str) -> Result<Vec<u8>> {
        Self::validate_filename(filename)?;
        let path = self.note_dir(note_id).join(filename);
        fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StoreError::AttachmentNotFound(filename.to_string())
            } else {
                StoreError::Io(e)
            }
        })
    }

    pub fn delete_attachment(&self, note_id: Uuid, filename: &str) -> Result<()> {
        Self::validate_filename(filename)?;
        let path = self.note_dir(note_id).join(filename);
        fs::remove_file(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StoreError::AttachmentNotFound(filename.to_string())
            } else {
                StoreError::Io(e)
            }
        })
    }

    // -- Private helpers ----------------------------------------------------

    fn notes_dir(&self) -> PathBuf {
        self.root.join("notes")
    }

    fn note_dir(&self, id: Uuid) -> PathBuf {
        self.notes_dir().join(id.to_string())
    }

    fn note_json_path(&self, id: Uuid) -> PathBuf {
        self.note_dir(id).join("note.json")
    }

    fn validate_filename(filename: &str) -> Result<()> {
        if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
            return Err(StoreError::InvalidFilename(filename.to_string()));
        }
        Ok(())
    }

    fn persist_note(&self, note: &Note) -> Result<()> {
        let json = serde_json::to_string_pretty(note)?;
        fs::write(self.note_json_path(note.id), json)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, NoteStore) {
        let dir = TempDir::new().unwrap();
        let store = NoteStore::open(dir.path()).unwrap();
        (dir, store)
    }

    #[test]
    fn open_idempotent() {
        let dir = TempDir::new().unwrap();
        let _s1 = NoteStore::open(dir.path()).unwrap();
        let _s2 = NoteStore::open(dir.path()).unwrap();
        assert!(dir.path().join("notes").is_dir());
    }

    #[test]
    fn create_and_get_roundtrip() {
        let (_dir, store) = make_store();
        let note = store.create_note().unwrap();
        assert!(note.title.is_none());
        assert!(note.blocks.is_empty());

        let fetched = store.get_note(note.id).unwrap();
        assert_eq!(note, fetched);
    }

    #[test]
    fn save_updates_timestamp() {
        let (_dir, store) = make_store();
        let note = store.create_note().unwrap();
        let original_updated = note.updated_at;

        let saved = store.save_note(note).unwrap();
        assert!(saved.updated_at >= original_updated);
    }

    #[test]
    fn save_nonexistent_note_errors() {
        let (_dir, store) = make_store();
        let note = Note {
            id: Uuid::new_v4(),
            title: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            blocks: Vec::new(),
        };
        let err = store.save_note(note).unwrap_err();
        assert!(matches!(err, StoreError::NoteNotFound(_)));
    }

    #[test]
    fn delete_and_get_errors() {
        let (_dir, store) = make_store();
        let note = store.create_note().unwrap();
        store.delete_note(note.id).unwrap();

        let err = store.get_note(note.id).unwrap_err();
        assert!(matches!(err, StoreError::NoteNotFound(_)));
    }

    #[test]
    fn delete_nonexistent_errors() {
        let (_dir, store) = make_store();
        let err = store.delete_note(Uuid::new_v4()).unwrap_err();
        assert!(matches!(err, StoreError::NoteNotFound(_)));
    }

    #[test]
    fn list_sorted_and_empty() {
        let (_dir, store) = make_store();
        assert!(store.list_notes().unwrap().is_empty());

        let n1 = store.create_note().unwrap();
        let n2 = store.create_note().unwrap();
        let n3 = store.create_note().unwrap();

        let list = store.list_notes().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].id, n1.id);
        assert_eq!(list[1].id, n2.id);
        assert_eq!(list[2].id, n3.id);
    }

    #[test]
    fn attachment_write_read_delete() {
        let (_dir, store) = make_store();
        let note = store.create_note().unwrap();
        let data = b"hello world";

        let block = store.write_attachment(note.id, "test.txt", data).unwrap();
        assert_eq!(
            block,
            Block::Attachment {
                file: "test.txt".to_string()
            }
        );

        let read_back = store.read_attachment(note.id, "test.txt").unwrap();
        assert_eq!(read_back, data);

        store.delete_attachment(note.id, "test.txt").unwrap();

        let err = store.read_attachment(note.id, "test.txt").unwrap_err();
        assert!(matches!(err, StoreError::AttachmentNotFound(_)));
    }

    #[test]
    fn invalid_filename_rejected() {
        let (_dir, store) = make_store();
        let note = store.create_note().unwrap();

        for bad in &["../etc/passwd", "foo/bar", "a\\b", "..hidden"] {
            let err = store.write_attachment(note.id, bad, b"x").unwrap_err();
            assert!(
                matches!(err, StoreError::InvalidFilename(_)),
                "expected InvalidFilename for {bad:?}, got {err:?}"
            );
        }
    }

    #[test]
    fn block_json_roundtrip() {
        let text = Block::Text {
            label: Some("transcript".to_string()),
            content: "hello".to_string(),
        };
        let json = serde_json::to_value(&text).unwrap();
        assert_eq!(json["kind"], "text");
        assert_eq!(json["label"], "transcript");
        assert_eq!(json["content"], "hello");

        let text_no_label = Block::Text {
            label: None,
            content: "bare".to_string(),
        };
        let json2 = serde_json::to_value(&text_no_label).unwrap();
        assert!(json2.get("label").is_none());

        let attachment = Block::Attachment {
            file: "recording.mp3".to_string(),
        };
        let json3 = serde_json::to_value(&attachment).unwrap();
        assert_eq!(json3["kind"], "attachment");
        assert_eq!(json3["file"], "recording.mp3");

        // Deserialize back
        let rt: Block = serde_json::from_value(json).unwrap();
        assert_eq!(rt, text);
        let rt3: Block = serde_json::from_value(json3).unwrap();
        assert_eq!(rt3, attachment);
    }
}
