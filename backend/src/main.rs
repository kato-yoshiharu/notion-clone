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

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "backend=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = create_pool().await;

    // TODO: マイグレーション実行はCIのデプロイ前ステップに移す
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run database migrations");

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

    let frontend_origin =
        std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    frontend_origin
                        .parse::<HeaderValue>()
                        .expect("invalid FRONTEND_ORIGIN"),
                )
                .allow_methods(Any)
                .allow_headers(vec![CONTENT_TYPE]),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
