# ポートフォリオ向け最小コストデプロイ構成

低トラフィックのポートフォリオを実質$0/月で運用することを目的とした構成をここに置く。

## アーキテクチャ

AWSとコスト面で比較した際に非常に安価であることから、Cloudflareを採用した。
ただしCloudflare Workersで動くRustは`wasm32-unknown-unknown`ターゲットへのコンパイルが必須で、tokioのようなマルチスレッドasyncランタイムは使えない。
axum自体は設定次第でWorkers上でも動くが、`sqlx`はtokioのTCPソケットに依存しており、ソケットAPIを持たない`wasm32-unknown-unknown`では動かない。そのため現行のRust/Axum/sqlxバックエンドはそのままではWorkers上に載らない。
そのためフロントエンドのホスティングをCloudflare Workersに置き、あわせてDNS・CDN・レートリミットもCloudflareに寄せる。
バックエンドはAWS Lambdaに、DBはNeonに置くハイブリッド構成にした。

| レイヤー                    | 採用サービス                                   | 備考                                                                                                                                                                 |
| --------------------------- | ---------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Frontend (Next.js)          | Cloudflare Workers（`@opennextjs/cloudflare`） |                                                                                                                                                                      |
| Backend (Rust Axum GraphQL) | AWS Lambda（`cargo-lambda` + `lambda_http`）   | 100万req/月永年無料。コードは書き換え不要（GraphQL Subscription等は未使用前提）                                                                                      |
| PostgreSQL                  | Neon（Serverless Postgres）                    | Lambdaの多コネクション問題に対応。sqlxがそのまま使える                                                                                                               |
| DNS / TLS / CDN             | Cloudflare（ゾーン管理）                       | 無料でHTTPS・独自ドメイン                                                                                                                                            |
| レートリミット              | Cloudflare Rate Limiting                       | フロント/API手前で標準搭載、追加実装不要。ただしLambdaのオリジン（Function URL / API Gateway）をCloudflare経由以外から直接叩けないよう制限しないと迂回可能な点に注意 |

不採用にした選択肢:

- DB を Aurora Serverless v2 にする案 → VPC・NATゲートウェイなど周辺コストが個人開発には過大
- Backend をRust/AxumのままCloudflare Workersに載せ、DBはNeonを継続する案
  - NeonのServerless DriverはJS/TS専用でRust/wasm実装がない
  - 生ソケット + `tokio-postgres`で繋ぐ道はあるが、Workersの`connect()`APIをtokioのAsyncストリームへ橋渡しする自作実装が必要で、かつ`sqlx`のコンパイル時クエリ検証も失う実験的経路のため見送り
