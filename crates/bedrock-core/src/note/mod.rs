//! 备忘：新建、列出、删除。

#[cfg(any(test, feature = "fake"))]
pub mod contract;
mod error;
#[cfg(any(test, feature = "fake"))]
mod fake;
mod model;
mod ops;
mod ports;

pub use error::{CreateNoteError, DeleteNoteError, ListNotesError};
#[cfg(any(test, feature = "fake"))]
pub use fake::MemoryNotes;
pub use model::{
    BODY_MAX, Note, NoteBody, NoteBodyError, NoteId, NoteTitle, NoteTitleError, TITLE_MAX,
};
pub use ops::{
    CreateNote, CreateNoteInput, DeleteNote, DeleteNoteInput, DeleteNoteOutput, ListNotes,
    ListNotesInput, NoteList, NoteView,
};
pub use ports::{NoteStore, StoreError};
