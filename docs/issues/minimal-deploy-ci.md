# infra/minimal のデプロイの GitHub Actions 化

## 概要

`infra/minimal` の構成は現在すべて手元での手動デプロイになっている。
`infra/minimal/README.md` の「デプロイ手順」は Neon → AWS → Cloudflare の 3 ステップで、
Lambda のクロスコンパイル、`terraform apply`、`pnpm build` + `wrangler deploy`、`wrangler secret put` を
それぞれ人間が順番に実行する前提で書かれている。

これを GitHub Actions から実行できるようにする。ただし現状は CI から回すうえで以下の問題がある。

### 1. Terraform の state がローカルにしかない

3 つの state（`neon` / `aws` / `cloudflare`）はいずれも `backend "local"` で、
`infra/.gitignore` により `*.tfstate` はコミットされていない。
CI の実行環境は毎回まっさらなので、このままでは state を持たない apply になり、既存リソースの再作成を試みる。

さらに `aws/neon.tf` は Neon の state を相対パスで参照している。

```hcl
data "terraform_remote_state" "neon" {
  backend = "local"
  config = { path = "${path.module}/../neon/terraform.tfstate" }
}
```

リモート backend に移す場合、この参照も同じ backend を指すよう書き換えが必要になる。

### 2. 変数がローカルの tfvars 前提

シークレットが入るため `*.tfvars` は gitignore 対象。
CI では別経路で値を渡す必要がある。

シークレットは `origin_shared_secret` のみで、これは GitHub Secrets に置いて `TF_VAR_origin_shared_secret` として渡す。
ただし同じ値が Lambda の環境変数（Terraform 管理）と Worker の secret（`wrangler secret put`）の両方に要るため、
2 つの経路へ同一の値を配る必要がある。

`neon.org_id` / `cloudflare.account_id` はシークレットではないので `variables.tf` の default に直書きし、
CI で渡す変数を `origin_shared_secret` だけに絞る。

### 3. AWS の認証がローカルプロファイル前提

`aws/versions.tf` の provider は `profile = var.aws_profile`（既定 `notion-clone`）を指定している。
CI には `~/.aws/credentials` が無いため、そのままでは認証できない。
長期のアクセスキーを Secrets に置くのは避けたいので、GitHub OIDC + IAM ロールの AssumeRole にしたい。

### 4. Lambda のビルドがホスト依存

`cargo make lambda-build` は `cargo-lambda` と zig、`aarch64-unknown-linux-gnu` ターゲットを要求する。
また `rust-toolchain` が 1.75 に固定されているため、`cargo-lambda` は 1.5.0 を使う必要がある（README 記載）。
CI ランナー上でこれを再現し、成果物が ARM の ELF であることを検証する必要がある。

### 5. Worker の secret 設定が手動

`BACKEND_ORIGIN` / `ORIGIN_SHARED_SECRET` は `wrangler secret put` で対話的に入力する手順になっている。
`BACKEND_ORIGIN` は AWS 側の `terraform output function_url` に依存するため、AWS のデプロイジョブから Cloudflare のデプロイジョブへ値を受け渡す経路が要る。

### 6. マイグレーションがアプリ起動時に走っている

README は `psql -f backend/migrations/...` を手で流す手順になっているが、
実際には `backend/src/main.rs` の `build_app` でも起動のたびに適用される。

```rust
// TODO: マイグレーション実行はCIのデプロイ前ステップに移す
sqlx::migrate!("./migrations").run(&pool).await
```

Lambda ではこれがコールドスタートのたびに実行される。以下の点で望ましくない。

- コールドスタートに適用処理が乗る。
  単独ならバージョン照合のクエリだけで短いが、
  同時実行数だけ実行環境が立つため、複数のコールドスタートが同時に適用を試みる。
- アプリの実行ロールに DDL 権限が必要になる
- 適用の失敗が「アプリが起動しない」という形で現れ、切り分けづらい

## MEMO

## 資格情報

| 名前                   | 置き場所               | 用途                              |
| ---------------------- | ---------------------- | --------------------------------- |
| AWS デプロイロール ARN | GitHub Secrets（OIDC） | Lambda / state バケットへの apply |
| `NEON_API_KEY`         | GitHub Secrets         | Neon provider の認証              |
| `CLOUDFLARE_API_TOKEN` | GitHub Secrets         | `wrangler deploy`                 |
| `ORIGIN_SHARED_SECRET` | GitHub Secrets         | Lambda と Worker の双方に投入     |

`neon.org_id` / `cloudflare.account_id` などの ID 類は認証に使う秘密ではないため、この表には含めない。
CI で渡す変数を減らす目的で `variables.tf` の `default` に直書きする（タスク 2）。

公開リポジトリのため、`BACKEND_ORIGIN`（Cloudflare を迂回できる入口）はログにも出さない。
`terraform output` を Actions の出力に載せる際は必ずマスクする。

## タスク

### 1. state のリモート化

AWS を既に使っているため S3 backend（S3 のネイティブロック `use_lockfile = true`、DynamoDB は使わない）を第一候補とする。

ただし state を置く S3 バケット自体は誰が作るのかという鶏卵問題があるため、
バケットとロールだけを扱う `infra/minimal/bootstrap` を切り出し、そこだけは手動 apply のままとする案を検討する。

Neon の state を AWS に置くことになるが、認証情報は state に入らない（`NEON_API_KEY` は環境変数）ため許容できる。
一方 `neon.connection_uri` と `aws.origin_shared_secret` は state に平文で入るので、
バケットは非公開・暗号化必須・バージョニング有効とする。

- [x] state 用 S3 バケットと GitHub OIDC 用 IAM ロールを作る（`infra/minimal/bootstrap`、手動 apply）
- [x] `neon` / `aws` / `cloudflare` の backend を S3 に変更する（`terraform init -migrate-state` の実行は手元で行う）
- [x] `aws/neon.tf` の `terraform_remote_state` を S3 backend 参照へ書き換える
- [x] `infra/.gitignore` と README の state に関する記述を更新する

### 2. 変数の受け渡し

シークレットは `origin_shared_secret` のみ。
これだけを GitHub Secrets から渡し、残りの ID 類はコードに持たせる。

- [ ] `neon.org_id` / `cloudflare.account_id` を `variables.tf` の `default` へ移す
- [ ] `origin_shared_secret` を `TF_VAR_origin_shared_secret` として GitHub Secrets から渡す
- [ ] `terraform.tfvars.example` を、ローカル専用である旨がわかる記述に更新する

### 3. AWS 認証の CI 対応

- [ ] `var.aws_profile` を空文字許容にし、CI では profile を使わない形にする
- [ ] `aws-actions/configure-aws-credentials` で OIDC AssumeRole する

### 4. Lambda ビルドジョブ

- [ ] `cargo-lambda@1.5.0` / zig / `aarch64-unknown-linux-gnu` をセットアップする
- [ ] `cargo make lambda-build` を実行し、`file` で ARM ELF であることを検証する
- [ ] 成果物を artifact として apply ジョブへ渡す

### 5. Terraform ワークフロー

デプロイ対象ごとに依存の向きが違うため、1 ファイルにまとめず順序を明示する。
`paths` で `infra/minimal/**` / `backend/**` / `frontend/**` を出し分ける。
Neon はほぼ変化せず、かつ `prevent_destroy` が付いた本番 DB のため、apply は手動実行に限定する。

- [ ] PR で `terraform plan`、`main` の push で `terraform apply` を実行する
- [ ] Neon は `workflow_dispatch` 限定にする
- [ ] `fmt -check` / `validate` を lint として追加する

### 6. Cloudflare デプロイジョブ

- [ ] `NEXT_PUBLIC_GRAPHQL_URL=/graphql pnpm build` の後に `wrangler deploy` する
- [ ] AWS ジョブの `function_url` を受け取り、`wrangler secret put` を非対話（stdin）で流す
- [ ] secret の値がログに出ないことを確認する

### 7. マイグレーション

CI のデプロイジョブで毎回実行する。
`sqlx migrate` は `_sqlx_migrations` テーブルで適用済みバージョンを管理するため、
毎回流しても実際に DDL が走るのは新しいファイルが増えたときだけで、条件分岐は不要。

実行順は「ビルド → マイグレーション適用 → `terraform apply`（Lambda 更新）」とする。

- [ ] CI に `sqlx-cli` をセットアップし、`sqlx migrate run` を Lambda 更新の前に実行する
- [ ] 接続先は Neon の `connection_uri` を使い、値がログに出ないようマスクする
- [ ] `backend/src/main.rs` の `sqlx::migrate!` 呼び出しと TODO コメントを削除する
- [ ] ローカル開発（`cargo make` の `migrate-run`）は現状のままとし、影響がないことを確認する
- [ ] README のスキーマ適用手順（`psql -f`）を CI 前提の記述へ差し替える

### 8. ドキュメント

- [ ] `infra/minimal/README.md` のデプロイ手順を、手動運用から CI 前提の記述へ更新する
- [ ] 手動で行う必要が残る操作（bootstrap、資格情報のローテーション）を明記する

## times
