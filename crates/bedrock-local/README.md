# bedrock-local

在本机跑核心：数据存在数据目录里的 SQLite 文件，时间取系统时间，编号用 UUID v7。命令行（含 `serve`）和给 Flutter 的接口出口都用 `bedrock_local::open(数据目录)` 得到核心。

## 不管什么

- 数据目录在哪：由各端决定（命令行看 `--data` 或 `BEDROCK_DATA`，Flutter 应用用系统给它的数据目录），`default_data_dir()` 只是默认值。
- 存储的细节：在 `bedrock-store-sqlite`。

## 不变量

- 加一个端口时，在这里的 `LocalPorts` 里接上真的实现；各端不各自组装端口。
