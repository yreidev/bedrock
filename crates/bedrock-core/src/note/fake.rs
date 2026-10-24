use std::sync::{Mutex, PoisonError};

use super::{Note, NoteId, NoteStore, StoreError};

/// 内存里的 [`NoteStore`]，守和真存储一样的约定。
#[derive(Debug, Default)]
pub struct MemoryNotes {
    notes: Mutex<Vec<Note>>,
}

impl MemoryNotes {
    fn notes(&self) -> std::sync::MutexGuard<'_, Vec<Note>> {
        // 锁里的操作都不会 panic 到一半，数据总是完整的
        self.notes.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl NoteStore for MemoryNotes {
    async fn insert(&self, note: &Note) -> Result<(), StoreError> {
        let mut notes = self.notes();
        if notes.iter().any(|n| n.id() == note.id()) {
            return Err(StoreError::Conflict);
        }
        notes.push(note.clone());
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Note>, StoreError> {
        let mut notes = self.notes().clone();
        notes.sort_by(|a, b| {
            (b.created_at(), b.id().as_str()).cmp(&(a.created_at(), a.id().as_str()))
        });
        Ok(notes)
    }

    async fn delete(&self, id: &NoteId) -> Result<(), StoreError> {
        let mut notes = self.notes();
        let before = notes.len();
        notes.retain(|n| n.id() != id);
        if notes.len() == before {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_store_keeps_the_contract() {
        super::super::contract::check_store(&MemoryNotes::default()).await;
    }
}
