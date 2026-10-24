# bedrock-store-sqlite

核心存储端口的 SQLite 实现。一个数据库文件，`SqliteStore` 实现所有业务的存储端口；表结构和迁移在 `migrations/`。

## 不管什么

- 业务规则：在核心里。这里只做领域类型和表的互转、库的错误转成端口的错误。
- 数据库文件放在哪：由组装的地方决定（`bedrock-local`）。

## 不变量

- 读回来的行也走领域类型的构造函数，库里的脏数据报 `StoreError::Other`，不会混进核心。
- 时间存成 UTC 毫秒数（`INTEGER`），和 `Clock` 的精度一致。
- 迁移只加不改：已经发出去的迁移文件不能动，要改表就加一个新迁移。
- 测试跑核心的约定测试（`bedrock_core::note::contract`），保证和内存版行为一致。
