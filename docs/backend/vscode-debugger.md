# BackendサーバをVSCode Debuggerでデバッグする方法

devcontainerを使うことで、Docker Containerの中のプロセスに対してdebuggerでデバッグすることができる。

## 設定

- [.devcontainer/devcontainer.json](https://github.com/suimenkathemove/notion-clone/blob/main/.devcontainer/devcontainer.json)
- [.vscode/launch.json](https://github.com/suimenkathemove/notion-clone/blob/main/.vscode/launch.json)

## 手順

1. Backend APIを起動する
2. Command Paletteで`Dev Containers: Reopen in Container`を実行する
3. Debuggerで`Attach 'backend'`を選択する
4. `/app/target/debug/backend`のprocessを選択する

## 参考

- <https://daveceddia.com/debug-electron-native-rust-with-vscode/>

## ヒープデータの値が表示されない問題

ヒープデータの値が表示されず、変数のアドレスが表示されてしまう場合の対処法。

### 原因

launch.jsonがない場合に生成できる設定の、

```json
"cargo": {
  "args": ["build", "--bin=backend", "--package=backend"],
  "filter": {
    "name": "backend",
    "kind": "bin"
  }
},
```

が必要のよう。

1. monorepoを採用している場合、`"cwd": "${workspaceFolder}"`がうまく機能しないようなので、devcontainer.jsonに`"workspaceFolder": "/app/backend"`を設定する
2. コンテナは起動しておくが、サーバは落としておく
3. `Dev Containers: Reopen in Container`を実行する
4. `Cargo.toml has been detected in this workspace.Would you like to generate launch configurations for its targets?`の、生成を実行する
5. デバッグの実行

```sh
# serverのprocessをkillする
kill -9 $(pidof backend)
```
