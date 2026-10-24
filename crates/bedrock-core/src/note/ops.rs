use jiff::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Clock, Core, Ctx, Ids, Internal, Kind, Operation, Ports};

use super::{
    CreateNoteError, DeleteNoteError, ListNotesError, Note, NoteBody, NoteId, NoteStore, NoteTitle,
    StoreError,
};

/// 一条备忘，给调用方看的样子。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NoteView {
    pub id: String,
    pub title: String,
    /// 没有正文时不出现
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub created_at: Timestamp,
}

impl From<&Note> for NoteView {
    fn from(note: &Note) -> Self {
        Self {
            id: note.id().as_str().to_owned(),
            title: note.title().as_str().to_owned(),
            body: (!note.body().is_empty()).then(|| note.body().as_str().to_owned()),
            created_at: note.created_at(),
        }
    }
}

/// 新建一条备忘。
pub struct CreateNote;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateNoteInput {
    /// 去掉首尾空白后 1 到 200 个字
    pub title: String,
    /// 最多 10000 个字，可以不写
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

impl Operation for CreateNote {
    const NAME: &'static str = "note.create";
    const KIND: Kind = Kind::Action;
    type Input = CreateNoteInput;
    type Output = NoteView;
    type Error = CreateNoteError;

    async fn run<P: Ports>(
        core: &Core<P>,
        _ctx: &Ctx,
        input: CreateNoteInput,
    ) -> Result<NoteView, CreateNoteError> {
        let title = NoteTitle::new(&input.title)?;
        let body = NoteBody::new(input.body.as_deref().unwrap_or_default())?;
        let ports = core.ports();
        let note = Note::new(
            NoteId::new(ports.ids().new_id()),
            title,
            body,
            ports.clock().now(),
        );
        ports.notes().insert(&note).await.map_err(Internal::new)?;
        Ok(NoteView::from(&note))
    }
}

/// 列出所有备忘，新建的在前。
pub struct ListNotes;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ListNotesInput {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NoteList {
    pub notes: Vec<NoteView>,
}

impl Operation for ListNotes {
    const NAME: &'static str = "note.list";
    const KIND: Kind = Kind::Query;
    type Input = ListNotesInput;
    type Output = NoteList;
    type Error = ListNotesError;

    async fn run<P: Ports>(
        core: &Core<P>,
        _ctx: &Ctx,
        _input: ListNotesInput,
    ) -> Result<NoteList, ListNotesError> {
        let notes = core.ports().notes().list().await.map_err(Internal::new)?;
        Ok(NoteList {
            notes: notes.iter().map(NoteView::from).collect(),
        })
    }
}

/// 删掉一条备忘。
pub struct DeleteNote;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteNoteInput {
    pub id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DeleteNoteOutput {}

impl Operation for DeleteNote {
    const NAME: &'static str = "note.delete";
    const KIND: Kind = Kind::Action;
    type Input = DeleteNoteInput;
    type Output = DeleteNoteOutput;
    type Error = DeleteNoteError;

    async fn run<P: Ports>(
        core: &Core<P>,
        _ctx: &Ctx,
        input: DeleteNoteInput,
    ) -> Result<DeleteNoteOutput, DeleteNoteError> {
        match core.ports().notes().delete(&NoteId::new(input.id)).await {
            Ok(()) => Ok(DeleteNoteOutput {}),
            Err(StoreError::NotFound) => Err(DeleteNoteError::NotFound),
            Err(err) => Err(Internal::new(err).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::{Core, Ctx, ErrorBody, ErrorKind, fake};

    async fn call(
        core: &Core<fake::FakePorts>,
        name: &str,
        input: Value,
    ) -> Result<Value, ErrorBody> {
        core.call(&Ctx::local(), name, input).await
    }

    #[tokio::test]
    async fn creates_and_lists_notes_newest_first() {
        let core = fake::core();
        let first = call(&core, "note.create", json!({ "title": " 买牛奶 " }))
            .await
            .unwrap();
        assert_eq!(
            first,
            json!({ "id": "id-1", "title": "买牛奶", "created_at": "2026-01-01T00:00:01Z" })
        );
        call(
            &core,
            "note.create",
            json!({ "title": "交房租", "body": "月底前" }),
        )
        .await
        .unwrap();

        let list = call(&core, "note.list", json!({})).await.unwrap();
        assert_eq!(
            list,
            json!({ "notes": [
                { "id": "id-2", "title": "交房租", "body": "月底前", "created_at": "2026-01-01T00:00:02Z" },
                { "id": "id-1", "title": "买牛奶", "created_at": "2026-01-01T00:00:01Z" },
            ] })
        );
    }

    #[tokio::test]
    async fn rejects_invalid_titles_with_their_own_codes() {
        let core = fake::core();
        let empty = call(&core, "note.create", json!({ "title": "  " }))
            .await
            .unwrap_err();
        assert_eq!(
            empty,
            ErrorBody {
                code: "empty_title",
                message: "标题不能为空".to_owned(),
                kind: ErrorKind::Invalid,
            }
        );
        let long = call(&core, "note.create", json!({ "title": "长".repeat(201) }))
            .await
            .unwrap_err();
        assert_eq!(long.code, "title_too_long");
        assert_eq!(long.message, "标题最多 200 个字");
    }

    #[tokio::test]
    async fn deletes_a_note_once() {
        let core = fake::core();
        call(&core, "note.create", json!({ "title": "买牛奶" }))
            .await
            .unwrap();

        assert_eq!(
            call(&core, "note.delete", json!({ "id": "id-1" })).await,
            Ok(json!({}))
        );
        let again = call(&core, "note.delete", json!({ "id": "id-1" }))
            .await
            .unwrap_err();
        assert_eq!((again.code, again.kind), ("not_found", ErrorKind::NotFound));
        assert_eq!(
            call(&core, "note.list", json!({})).await,
            Ok(json!({ "notes": [] }))
        );
    }

    #[tokio::test]
    async fn reports_malformed_input_and_unknown_names() {
        let core = fake::core();
        let missing = call(&core, "note.create", json!({})).await.unwrap_err();
        assert_eq!(missing.code, "invalid_input");
        assert!(missing.message.contains("title"), "{}", missing.message);

        let unknown = call(&core, "note.rename", json!({})).await.unwrap_err();
        assert_eq!(
            (unknown.code, unknown.kind),
            ("unknown_operation", ErrorKind::NotFound)
        );
    }
}
