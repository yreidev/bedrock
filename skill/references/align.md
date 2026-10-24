# 让项目跟上 bedrock 的新规矩

项目的 `AGENTS.md` 开头记着它建自（或上次跟到）bedrock 的哪个提交，下面叫它 `<旧>`。

1. 看 bedrock 从那以后改了什么：

   ```sh
   git -C "$BEDROCK" log --oneline <旧>..HEAD
   git -C "$BEDROCK" diff <旧>..HEAD -- docs/conventions
   git -C "$BEDROCK" diff <旧>..HEAD -- . ':!docs/conventions' ':!docs/PLAN.md' ':!skill' ':!README.md' ':!AGENTS.md' ':!CHANGELOG.md'
   ```

   第二条是规矩的变化，第三条是骨架的变化。骨架里的名字是 `bedrock`，对到项目里换成项目名。
2. 逐条判断项目要不要跟，列成清单给用户确认：跟哪些、哪些不跟和为什么。项目已经按自己的实际情况改过的地方，以项目为准。
3. 照清单改，`just ci` 通过。
4. 把 `AGENTS.md` 里的提交换成 bedrock 现在的提交；决定不跟的写进项目的 `docs/PLAN.md`，下次对比时不用再判断一遍。
