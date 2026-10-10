---
id: TASK-1
title: ルート直下のディレクトリ構成を整理する
status: Done
assignee: []
created_date: '2026-10-07 22:30'
updated_date: '2026-10-10 15:43'
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

手順3: Cargo workspace の解消（完了）

- 移動と削除
  - Cargo.lock と rust-toolchain を backend/ に git mv した
  - ルートの Cargo.toml を削除した。[workspace] だけのファイルで、backend は単独パッケージになるため移す先がない
  - ルートの .gitignore を削除した。中身は /target だけで、backend/.gitignore に同じ行がある
- 手順6の一部を先に実施
  - ルートの target が不要になり、Docker の volume を /app/backend/target に変えないと検証できないため
  - backend/docker/docker-compose.yml、.vscode/launch.json、docs/backend/vscode-debugger.md の /app/target を /app/backend/target に変えた
- Cargo.lock
  - Docker 内の cargo で cargo update --workspace を実行したが、差分は出なかった。手順2で整理済みで、version = 3 のまま
- 確認
  - cargo make check / lint / format / test がすべて通った（test は 17 件成功）
- 後始末
  - 不要になったルートの target/ を削除した（git 管理外のビルド成果物）
  - volume を作り直すため docker compose down を実行した。DB のデータ volume は残っている

手順4: graphql-codegen の移動（完了）

- 移動
  - graphql-codegen/ を frontend/graphql-codegen/ に移した（bin/codegen.ts と lib/scalar.ts だけを残した）
- 自前の設定を frontend に統合
  - 削除したもの: package.json、pnpm-lock.yaml、tsconfig.json、.eslintrc.js、.prettierrc.js、.gitignore、README.md（空）
  - frontend/package.json の devDependencies に次のパッケージを追加した
    - @graphql-codegen/cli（5.0.0 固定）、typescript、typescript-operations、typescript-react-apollo（いずれも ^4 系）
    - @graphql-codegen/add（^5）。元は client-preset の推移的依存として入っていたもので、ないと add プラグインが見つからず失敗する
    - client-preset と ts-node は、使われていないので入れていない
  - frontend/package.json の scripts に codegen スクリプトを追加した（graphql-codegen --config graphql-codegen/bin/codegen.ts）
  - frontend/tsconfig.json の include に graphql-codegen/**/*.ts を追加し、frontend の lint 対象にした
  - bin/codegen.ts のパスは、frontend 直下を基準にした（src/**/*.graphql.ts、src/graphql/generated/index.ts）
- 確認
  - pnpm tsc / pnpm lint / pnpm build が通った
  - backend を cargo make dev で起動して pnpm codegen を実行し、成功した
  - 生成物（src/graphql/generated/index.ts）は git 管理下で、再生成しても差分は出なかった

手順5: docs/issues/ の移動（完了）

- 移動
  - docs/issues/api-state-update.md と fix-dev-env.md を backlog/tasks/ に git mv した。ファイル名と中身は変えていない
  - 空になった docs/issues/ を削除した
- 確認
  - backlog task list は TASK-1 だけを表示する。移した2ファイルはタスクとして認識されない（想定どおり）
  - backlog doctor に問題はなかった
  - 旧パス docs/issues への参照は、他のファイルになかった

手順6: 参照パスの更新（完了）

- .github/workflows/backend.yml の paths（push と pull_request の2か所）から models/** を消した
- infra/minimal/aws/lambda.tf の bootstrap_path を、ルートの target から backend/target に変えた
  - 計画外。target が backend/ に移ったため、terraform が Lambda のバイナリを見つけられなくなる
  - infra/minimal/README.md の file target/lambda/backend/bootstrap は、cd backend の後に実行する前提なので変更不要
- /app/target の置き換えは、手順3で実施済み
- 旧パス（models/、wasm、graphql-codegen、/app/target、docs/issues）を grep し、取りこぼしがないことを確認した
- 最終確認
  - backend: cargo make check / lint / format / test がすべて通った（test は 17 件成功）
  - frontend: pnpm tsc / lint / build が通った

手順7: AC への反映（完了）

- 未チェックだった4項目（docs、graphql-codegen、models、wasm）を [x] にした。12項目すべてがチェック済み
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
ルート直下の構成を整理し、移動・改名で壊れる参照を直した。

変更内容:

- wasm/ とフロントエンドの wasm 関連ファイルを削除した
- models クレートを backend/src/models/ に取り込み、Cargo workspace を解消した
  - Cargo.lock と rust-toolchain は backend/ に移し、ルートの Cargo.toml と .gitignore は削除した
- graphql-codegen を frontend/graphql-codegen/ に移し、自前の設定ファイルを frontend の設定に統合した
  - codegen の依存を frontend/package.json に追加し、codegen スクリプトを足した
- docs/issues/ の2ファイルを backlog/tasks/ に移した
- 参照パスを更新した
  - /app/target を /app/backend/target に変えた（docker-compose.yml、.vscode/launch.json、docs/backend/vscode-debugger.md）
  - .github/workflows/backend.yml の paths から models/** を消した
  - infra/minimal/aws/lambda.tf の bootstrap_path を backend/target に変えた

検証結果:

- backend: cargo make check / lint / format / test が通った（test は 17 件成功）
- frontend: pnpm tsc / lint / build が通った
- backend を起動して pnpm codegen を実行し、成功した。生成物に差分は出なかった
- 旧パスを grep し、取りこぼしがないことを確認した

リスクとフォローアップ:

- frontend/pnpm-lock.yaml が約1,950行増えた（codegen の依存を追加したため）
- Cargo.lock は version 3 のまま。ホストの cargo（1.98）で操作すると version 4 に書き換えられ、Docker 内の cargo（1.75）が読めなくなる
- infra/ と infrastructure/ の整理は据え置き。別タスクで対応する
<!-- SECTION:FINAL_SUMMARY:END -->

## AC

対応不要確認済み、対応済みにチェックする

- [x] .devcontainer
- [x] .github
- [x] .vscode
- [x] backend
- [x] backlog
- [x] docs
- [x] frontend
- [x] graphql-codegen
- [x] infra
- [x] infrastructure
- [x] models
- [x] wasm
