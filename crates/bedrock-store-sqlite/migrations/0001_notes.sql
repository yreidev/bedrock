CREATE TABLE notes (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    -- 没有正文时是空字符串
    body TEXT NOT NULL DEFAULT '',
    -- UTC，自 1970 年起的毫秒数
    created_at INTEGER NOT NULL
);

CREATE INDEX notes_by_created_at ON notes (created_at DESC, id DESC);
