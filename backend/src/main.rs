mod apis;
mod repositories;
mod use_cases;

use crate::{
    apis::{
        handlers::{graphql_handler::graphql_handler, graphql_playground::graphql_playground},
        MutationRoot, QueryRoot,
    },
    repositories::{
        interfaces::page::IPageRepository,
        postgres::{create_pool, page::PageRepository},
    },
    use_cases::page::PageUseCase,
};
use async_graphql::{EmptySubscription, Schema};
use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Extension, Router,
};
use std::{net::SocketAddr, sync::Arc};
use subtle::ConstantTimeEq;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Lambda実行環境ではこの変数がランタイムから必ず注入される。
/// これを見てローカル起動とLambda起動を切り替える。
fn is_running_on_lambda() -> bool {
    std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok()
}

/// Cloudflareが付与する共有シークレットのヘッダ名。
const ORIGIN_SECRET_HEADER: &str = "x-origin-secret";

/// Function URLは認証なしで公開されるため、Cloudflareを迂回した直接アクセスをここで弾く。
/// これがないとCloudflare側のレートリミットを回避されてしまう。
async fn verify_origin_secret<B>(
    State(expected): State<Arc<String>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let presented = request
        .headers()
        .get(ORIGIN_SECRET_HEADER)
        .and_then(|value| value.to_str().ok());

    // 比較時間からシークレットを推測されないよう、定数時間で突き合わせる。
    let matched = presented.is_some_and(|value| value.as_bytes().ct_eq(expected.as_bytes()).into());

    if matched {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// ローカルは`axum::body::Body`、Lambdaは`lambda_http::Body`とボディ型が異なるため、
/// ルータをボディ型に対して汎用にしておき、呼び出し側で具体化する。
async fn build_app<B>() -> Router<(), B>
where
    B: axum::body::HttpBody<Data = axum::body::Bytes> + Send + Sync + Unpin + 'static,
    B::Error: Into<axum::BoxError>,
{
    let pool = create_pool().await;
    let pool = Arc::new(pool);

    let page_repository: Arc<dyn IPageRepository> =
        Arc::new(PageRepository::new(Arc::clone(&pool)));

    let page_use_case = PageUseCase::new(Arc::clone(&page_repository));

    let schema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(page_use_case)
    .finish();

    let allowed_origin =
        std::env::var("CORS_ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());

    let router = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema));

    // ローカル開発では未設定のため検証を挟まない。
    let router = match std::env::var("ORIGIN_SHARED_SECRET") {
        Ok(secret) if !secret.is_empty() => router.layer(middleware::from_fn_with_state(
            Arc::new(secret),
            verify_origin_secret,
        )),
        _ => {
            if is_running_on_lambda() {
                tracing::warn!(
                    "ORIGIN_SHARED_SECRET is not set; the Function URL is publicly reachable"
                );
            }
            router
        }
    };

    // CORSは必ず最も外側に置く。
    // 内側にすると403応答にCORSヘッダが付かず、ブラウザ側で原因の分からないエラーになる。
    // プリフライトも検証前に処理される。
    router.layer(
        CorsLayer::new()
            .allow_origin(allowed_origin.parse::<HeaderValue>().unwrap())
            .allow_methods(Any)
            .allow_headers(vec![CONTENT_TYPE]),
    )
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    let subscriber = tracing_subscriber::registry().with(tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "backend=trace".into()),
    ));
    if is_running_on_lambda() {
        // CloudWatch Logsではエスケープシーケンスが読めず、時刻もLambda側が付与する。
        subscriber
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .without_time(),
            )
            .init();
    } else {
        subscriber.with(tracing_subscriber::fmt::layer()).init();
    }

    if is_running_on_lambda() {
        lambda_http::run(build_app::<lambda_http::Body>().await).await?;
    } else {
        let app = build_app::<axum::body::Body>().await;
        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        tracing::debug!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
    }

    Ok(())
}
