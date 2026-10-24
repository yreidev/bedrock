use crate::Internal;
use crate::error::op_error;

use super::{NoteBodyError, NoteTitleError};

#[derive(Debug, thiserror::Error)]
pub enum CreateNoteError {
    #[error(transparent)]
    Title(#[from] NoteTitleError),
    #[error(transparent)]
    Body(#[from] NoteBodyError),
    #[error(transparent)]
    Internal(#[from] Internal),
}

op_error! {
    CreateNoteError {
        Title(NoteTitleError::Empty) => "empty_title", Invalid;
        Title(NoteTitleError::TooLong) => "title_too_long", Invalid;
        Body(NoteBodyError::TooLong) => "body_too_long", Invalid;
        Internal(_) => "internal", Internal;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ListNotesError {
    #[error(transparent)]
    Internal(#[from] Internal),
}

op_error! {
    ListNotesError {
        Internal(_) => "internal", Internal;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteNoteError {
    #[error("这条备忘不存在，可能已经删掉了")]
    NotFound,
    #[error(transparent)]
    Internal(#[from] Internal),
}

op_error! {
    DeleteNoteError {
        NotFound => "not_found", NotFound;
        Internal(_) => "internal", Internal;
    }
}
