# ポートフォリオ向け最小コストデプロイ構成

低トラフィックのポートフォリオを実質$0/月で運用することを目的とした構成をここに置く。

## アーキテクチャ

AWSとコスト面で比較した際に非常に安価であることから、Cloudflareを採用した。
ただしCloudflare Workersで動くRustは`wasm32-unknown-unknown`ターゲットへのコンパイルが必須で、tokioのようなマルチスレッドasyncランタイムは使えない。
axum自体は設定次第でWorkers上でも動くが、`sqlx`はtokioのTCPソケットに依存しており、ソケットAPIを持たない`wasm32-unknown-unknown`では動かない。そのため現行のRust/Axum/sqlxバックエンドはそのままではWorkers上に載らない。
そのためフロントエンドのホスティングをCloudflare Workersに置き、あわせてDNS・CDN・レートリミットもCloudflareに寄せる。
バックエンドはAWS Lambdaに、DBはNeonに置くハイブリッド構成にした。

| レイヤー                    | 採用サービス                                       | 備考                                                                                        |
| --------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Frontend (Next.js)          | Cloudflare Workers（静的書き出し + Static Assets） | SSRを使っていないため`output: "export"`で書き出す。同じWorkerが`/graphql`をLambdaへ中継する |
| Backend (Rust Axum GraphQL) | AWS Lambda（`cargo-lambda` + `lambda_http`）       | 100万req/月永年無料。コードは書き換え不要（GraphQL Subscription等は未使用前提）             |
| PostgreSQL                  | Neon（Serverless Postgres）                        | Lambdaの多コネクション問題に対応。sqlxがそのまま使える                                      |
| DNS / TLS / CDN             | Cloudflare（ゾーン管理）                           | 無料でHTTPS・独自ドメイン                                                                   |
| レートリミット              | Workers Rate Limiting binding                      | ゾーン版のWAFルールは独自ドメインが要るため、Worker内で`/graphql`のみ制限する               |

APIは必ずWorkerを経由させる。
ブラウザからLambdaのFunction URLを直接叩く形だと、Cloudflareが経路に入らずレートリミットが効かない上、共有シークレットをJSに埋めることになり秘密にできない。
Workerを挟むことでAPIが同一オリジンになり、CORSも不要になる。

不採用にした選択肢:

- フロントエンドを`@opennextjs/cloudflare`で載せる案
  - 現行版のpeerDependencyは`next: >=15.5.21`で、Next.js 14のサポートは2026 Q1に終了済み
  - SSR・API Routesを使っていないため、Workers上でNodeランタイムを動かす必要がそもそも無い
  - 将来SSRやISRが必要になった時点で、Next.jsのメジャーアップグレードとあわせて再検討する
- DB を Aurora Serverless v2 にする案 → VPC・NATゲートウェイなど周辺コストが個人開発には過大
- DB を Supabase にする案（2026-07時点の無料枠で比較）
  - 無料枠では7日間アクセスがないとプロジェクトが一時停止し、手動で復帰させる必要がある。
- Backend をRust/AxumのままCloudflare Workersに載せ、DBはNeonを継続する案
  - NeonのServerless DriverはJS/TS専用でRust/wasm実装がない
  - 生ソケット + `tokio-postgres`で繋ぐ道はあるが、Workersの`connect()`APIをtokioのAsyncストリームへ橋渡しする自作実装が必要で、かつ`sqlx`のコンパイル時クエリ検証も失う実験的経路のため見送り

## Terraform state

`neon` / `aws` / `cloudflare` の3つのstateは、
S3 backend（バケット `kato-yoshiharu-notion-clone-tfstate`、キーは `<ディレクトリ名>/terraform.tfstate`）に置く。
ロックはS3のネイティブロック（`use_lockfile = true`）で行うため、DynamoDBのテーブルは使わない。

`neon.connection_uri` と `aws.origin_shared_secret` はstateに平文で入るため、
バケットは非公開・暗号化必須・バージョニング有効とし、非TLSのアクセスはバケットポリシーで拒否している。

そのバケットとGitHub Actions用のIAMロール自体は、stateを置く先が無い状態で作る必要がある。
この鶏卵問題を避けるため `infra/minimal/bootstrap` に切り出し、**ここだけはローカルstateの手動applyのまま**とする。

```sh
cd infra/minimal/bootstrap
AWS_PROFILE=notion-clone terraform init
AWS_PROFILE=notion-clone terraform apply
terraform output github_actions_role_arn  # GitHub Secretsに登録する
```

以降の `neon` / `aws` / `cloudflare` をローカルから実行するときは、
backendの認証にプロファイルを使うため `AWS_PROFILE=notion-clone` を付ける。

```sh
AWS_PROFILE=notion-clone terraform init
```

ローカルstateからS3へ移行済みの環境で、手元に古い `terraform.tfstate` が残っている場合は次のように移す。

```sh
AWS_PROFILE=notion-clone terraform init -migrate-state
```

## デプロイ手順

Neon → AWS → Cloudflare の順に依存している。初回は上から順に実行する。
いずれも先に `bootstrap` のapplyが済んでいる前提。

### 1. Neon

```sh
cd infra/minimal/neon
AWS_PROFILE=notion-clone terraform init  # stateはS3 backend
terraform apply
```

作られたDBは空なので、スキーマを流し込む。

```sh
psql "$(terraform output -raw connection_uri)" \
  -v ON_ERROR_STOP=1 -f ../../../backend/migrations/20230305132144_notion.sql
```

### 2. AWS (バックエンド)

Lambdaのバイナリを先にビルドする。クロスコンパイルのためDocker外のホストで実行する。

初回のみ以下が必要。`cargo-lambda`の最新版はrustc 1.85以上を要求するが、
このリポジトリは`rust-toolchain`で1.75に固定しているため、対応するバージョンを指定する。

```sh
cargo install cargo-lambda@1.5.0 --locked
brew install zig                         # クロスコンパイルのリンカに使う
rustup target add aarch64-unknown-linux-gnu
```

```sh
cd backend
cargo make lambda-build
```

`target/lambda/backend/bootstrap` が `ELF 64-bit, ARM aarch64` になっていれば成功。
x86バイナリでもterraform applyは通り、実行時に初めて失敗するので確認しておく。

```sh
file target/lambda/backend/bootstrap
```

共有シークレットを生成し、`terraform.tfvars` に書く。同じ値をあとでWorkerにも設定する。
CIから実行する場合は、GitHub Secretsの `ORIGIN_SHARED_SECRET` を `TF_VAR_origin_shared_secret` として渡す。

```sh
openssl rand -hex 32
```

```sh
cd infra/minimal/aws
cp terraform.tfvars.example terraform.tfvars  # origin_shared_secret を埋める
AWS_PROFILE=notion-clone terraform init  # stateはS3 backend
terraform apply
terraform output function_url
```

### 3. Cloudflare (フロントエンド)

```sh
cd frontend
pnpm install
cp .dev.vars.example .dev.vars
export CLOUDFLARE_API_TOKEN=$(cd ../infra/minimal/cloudflare && terraform output -raw main_token_value)
```

ビルドしてデプロイする。`NEXT_PUBLIC_*` はビルド時に埋め込まれる。

```sh
NEXT_PUBLIC_GRAPHQL_URL=/graphql pnpm build
pnpm deploy:cf
```

`BACKEND_ORIGIN`と`ORIGIN_SHARED_SECRET`はsecretとして設定する。
公開リポジトリのため、Cloudflareを迂回する入口になる`BACKEND_ORIGIN`も`vars`には置かない。
デプロイ済みWorkerに同名の`vars`が残っていると`already in use`になるので、デプロイを先に行う。

```sh
npx wrangler secret put BACKEND_ORIGIN        # 手順2の function_url
npx wrangler secret put ORIGIN_SHARED_SECRET  # 手順2と同じ値
```

`wrangler.jsonc`のバインディングを変えたときは`pnpm types`で型を再生成する。

デプロイ先URLは`pnpm deploy:cf`の出力に表示される。

## 資格情報の扱い

いずれもリポジトリには置かず、パスワードマネージャに保管する。

| 資格情報                           | 取得元                                         | ローテーション                                           |
| ---------------------------------- | ---------------------------------------------- | -------------------------------------------------------- |
| Cloudflareブートストラップトークン | ダッシュボードで手動作成                       | ダッシュボードで再作成                                   |
| `notion-clone-ci`トークン          | `terraform output -raw main_token_value`       | `terraform apply -replace=cloudflare_account_token.main` |
| Neon APIキー                       | Neonのダッシュボード                           | ダッシュボードで再作成。stateには入らない                |
| `origin_shared_secret`             | GitHub Secrets / ローカルの `terraform.tfvars` | 値を変えてapplyし、`wrangler secret put`も更新する       |

`origin_shared_secret`はLambdaとWorkerの両方に同じ値が必要なため、片方だけ更新すると全リクエストが403になる。

## ローカルでの確認

`pnpm run preview` で、本番と同じWorker経由の構成を手元で再現できる。

```sh
cd frontend
NEXT_PUBLIC_GRAPHQL_URL=/graphql pnpm build
npx wrangler dev --var BACKEND_ORIGIN:http://localhost:8080/ --var ORIGIN_SHARED_SECRET:dummy
```

通常の開発は `pnpm dev` のままでよい。この場合はWorkerを経由せず、`http://localhost:8080` を直接叩く。
