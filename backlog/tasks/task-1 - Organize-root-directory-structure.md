---
id: TASK-1
title: ルート直下のディレクトリ構成を整理する
status: In Progress
assignee: []
created_date: '2026-10-07 22:30'
updated_date: '2026-10-10 14:37'
labels:
  - chore
dependencies: []
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ルート直下の構成を整理し、移動・改名で壊れる参照も合わせて直す。
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
### 据え置くもの

- infra/ と infrastructure/
  - この整理は別タスクで対応

### 実装の順序

1. wasm を削除する
   - 削除するもの: wasm/、frontend/src/wasm/pkg/、frontend/src/pages/wasm.tsx
   - frontend/.prettierignore と frontend/.eslintrc.js から src/wasm の行を消す
2. models を backend に取り込む
   - models/src/* を backend/src/models/ に移す
   - backend/src/lib.rs に mod models を追加する
   - backend 内の models:: の参照（8ファイル、132箇所）を crate::models:: に書き換える
   - backend/Cargo.toml から models の path 依存を消す
   - chrono と uuid は backend が上位互換の feature を持つので、依存の追加は不要
3. Cargo workspace を解消する
   - ルートの Cargo.toml、Cargo.lock、rust-toolchain を backend/ に移す
   - ルートの .gitignore から /target を消す
   - Cargo.lock は cargo で再生成し、差分を確認する
4. graphql-codegen を frontend/graphql-codegen/ に移す
   - codegen.ts の ../frontend/src を ../src に変える
5. docs/issues/ を backlog/tasks/ に移す
   - api-state-update.md と fix-dev-env.md を git mv で移す。ファイル名と中身は変えない
   - 空になった docs/issues/ を削除する
   - backlog CLI の管理形式（frontmatter、task-N の命名）ではないので、タスクとしては認識されない。管理外の markdown として置く
6. 参照パスを更新する
   - .github/workflows/backend.yml の paths から models/** を消す
   - /app/target を /app/backend/target に変える: backend/docker/docker-compose.yml、.vscode/launch.json、docs/backend/vscode-debugger.md
7. このタスクの AC に、結果を反映する

### 確認方法

各ステップの後と、最後にまとめて行う確認:

- backend/ で cargo make check、cargo make lint、cargo make format、cargo make test を実行する
- frontend/ で pnpm tsc、pnpm lint、pnpm build を実行する
- frontend/graphql-codegen/ で pnpm tsc を実行する
- grep で旧パス（models/、wasm、graphql-codegen、/app/target）の取りこぼしがないか確認する

### リスク

実装中に注意する点:

- models の macros.rs の macro_export の扱いが変わる可能性がある。cargo check で確かめる
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
手順1: wasm の削除（完了）

- 削除したもの
  - wasm/
  - frontend/src/wasm/
  - frontend/src/pages/wasm.tsx
- 設定の修正
  - frontend/.prettierignore と frontend/.eslintrc.js から src/wasm を削除した
  - ルート Cargo.toml の workspace members から wasm も外した（計画外。wasm/ を消すと cargo が壊れるため）
- 確認
  - pnpm tsc と pnpm lint が通った
- 持ち越し
  - Cargo.lock の wasm エントリは手順3で再生成する

手順2: models を backend に取り込む（完了）

- 移動
  - models/src/* を backend/src/models/ に移した（lib.rs は mod.rs に改名）
  - models/Cargo.toml を削除した
  - backend/src/models/page.rs の use crate::common を use super::common に直した
- 参照の書き換え
  - backend 内の models:: を crate::models:: に書き換えた
  - backend/Cargo.toml から models の path 依存を消した
  - lib.rs と main.rs の両方に mod models を追加した。main.rs と lib.rs が別クレートとして repositories を持つため
  - macro_use の define_id が他モジュールへ漏れないよう、mod models は各ファイルの末尾に置いた
- 計画外の変更
  - ルート Cargo.toml の workspace members から models を外した（手順3で workspace ごと解消する）
  - Cargo.lock から models と wasm のエントリを消した。ホストの cargo 1.98 が lock を version 4 に書き換えて Docker 内の cargo 1.75 が読めなくなるため、version = 3 に戻した
  - models 側の lint が backend の clippy 対象になり、2件が出たので直した
    - define_id と DateTimeUtc の cfg(test) な new() に allow(clippy::new_without_default) を付けた
    - page.rs の MoveTarget をテストモジュールより前に移した
- 確認
  - cargo make check / lint / format / test がすべて通った（test は 17 件成功）
  - test は test プロファイルの DB が未作成だったため、cargo make -p test init を先に実行した
<!-- SECTION:NOTES:END -->

## AC

対応不要確認済み、対応済みにチェックする

- [x] .devcontainer
- [x] .github
- [x] .vscode
- [x] backend
- [x] backlog
- [ ] docs
- [x] frontend
- [ ] graphql-codegen
- [x] infra
- [x] infrastructure
- [ ] models
- [ ] wasm
