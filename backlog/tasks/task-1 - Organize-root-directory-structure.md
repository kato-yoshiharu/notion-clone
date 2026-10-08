---
id: TASK-1
title: ルート直下のディレクトリ構成を整理する
status: To Do
assignee: []
created_date: '2026-10-07 22:30'
updated_date: '2026-10-08 08:04'
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
