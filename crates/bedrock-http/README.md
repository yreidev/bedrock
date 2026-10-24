# bedrock-http

把核心的操作面开成 HTTP：`POST /api/op/<名字>` 和 `GET /api/health`。命令行的 `bedrock serve` 用它，自己不读配置、不监听，监听好的 `TcpListener` 和核心由调用方传进来。

## 不管什么

- 监听哪个地址、什么时候退出：由调用方决定（`serve` 的第三个参数是退出信号）。
- 认证：还没有。只监听本机时靠主机名检查挡住浏览器，监听别的地址前要先加认证。

## 不变量

- 只收 `Content-Type: application/json`，空的请求体当作 `{}`。
- 只监听本机地址时，只认本机的主机名，挡住 DNS 重绑定。
- 任何路径都回 JSON，带安全头；错误的状态码只按 `ErrorKind` 映射，不看具体的 `code`。
