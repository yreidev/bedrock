# 开发

## 环境

<!-- 需要装什么、什么版本。版本以 rust-toolchain.toml、.fvmrc、justfile 为准，这里只写怎么装。 -->

第一次克隆后运行一次：

```sh
just deps
```

## 常用命令

```sh
just dev serve    # 开发模式开服务
just check        # 只检查不改文件
just test         # 测试
just gen          # 改了操作的输入输出后重新生成类型
```

## 调试

<!-- 日志怎么开、数据在哪、怎么只跑某一个测试。 -->
