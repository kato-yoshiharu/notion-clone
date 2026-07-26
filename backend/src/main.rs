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
    http::{header::CONTENT_TYPE, HeaderValue},
    routing::get,
    Extension, Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Lambda実行環境ではこの変数がランタイムから必ず注入される。
/// これを見てローカル起動とLambda起動を切り替える。
fn is_running_on_lambda() -> bool {
    std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok()
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

    Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema))
        .layer(
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
