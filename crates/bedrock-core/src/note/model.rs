use jiff::Timestamp;

/// 标题最多多少个字。
pub const TITLE_MAX: usize = 200;
/// 正文最多多少个字。
pub const BODY_MAX: usize = 10_000;

/// 一条备忘的编号。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoteId(String);

impl NoteId {
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 备忘的标题：去掉首尾空白后 1 到 [`TITLE_MAX`] 个字。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteTitle(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NoteTitleError {
    #[error("标题不能为空")]
    Empty,
    #[error("标题最多 {TITLE_MAX} 个字")]
    TooLong,
}

impl NoteTitle {
    pub fn new(raw: &str) -> Result<Self, NoteTitleError> {
        let title = raw.trim();
        if title.is_empty() {
            return Err(NoteTitleError::Empty);
        }
        if title.chars().count() > TITLE_MAX {
            return Err(NoteTitleError::TooLong);
        }
        Ok(Self(title.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 备忘的正文：去掉首尾空白后最多 [`BODY_MAX`] 个字，空的就是没有正文。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NoteBody(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NoteBodyError {
    #[error("正文最多 {BODY_MAX} 个字")]
    TooLong,
}

impl NoteBody {
    pub fn new(raw: &str) -> Result<Self, NoteBodyError> {
        let body = raw.trim();
        if body.chars().count() > BODY_MAX {
            return Err(NoteBodyError::TooLong);
        }
        Ok(Self(body.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// 一条备忘。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    id: NoteId,
    title: NoteTitle,
    body: NoteBody,
    created_at: Timestamp,
}

impl Note {
    /// 新建时用，适配器从存储里读回来时也用：各字段都已经是校验过的类型。
    pub fn new(id: NoteId, title: NoteTitle, body: NoteBody, created_at: Timestamp) -> Self {
        Self {
            id,
            title,
            body,
            created_at,
        }
    }

    pub fn id(&self) -> &NoteId {
        &self.id
    }

    pub fn title(&self) -> &NoteTitle {
        &self.title
    }

    pub fn body(&self) -> &NoteBody {
        &self.body
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_the_title() {
        assert_eq!(NoteTitle::new("  买牛奶 \n").unwrap().as_str(), "买牛奶");
    }

    #[test]
    fn rejects_empty_title() {
        assert_eq!(NoteTitle::new(" \t "), Err(NoteTitleError::Empty));
    }

    #[test]
    fn counts_title_length_in_characters() {
        assert!(NoteTitle::new(&"字".repeat(TITLE_MAX)).is_ok());
        assert_eq!(
            NoteTitle::new(&"字".repeat(TITLE_MAX + 1)),
            Err(NoteTitleError::TooLong)
        );
    }

    #[test]
    fn blank_body_is_empty() {
        assert!(NoteBody::new("  \n").unwrap().is_empty());
        assert_eq!(
            NoteBody::new(&"字".repeat(BODY_MAX + 1)),
            Err(NoteBodyError::TooLong)
        );
    }
}
