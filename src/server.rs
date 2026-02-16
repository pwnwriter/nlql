// http server mode - run nlql as an api

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::core::QueryResult;
use crate::{Claude, Db, Error, Safety};

struct AppState {
    db: Db,
    schema: String,
}

#[derive(Deserialize)]
struct QueryRequest {
    prompt: String,
    #[serde(default)]
    dry_run: bool,
    #[serde(default)]
    run_dangerous: bool,
}

#[derive(Serialize)]
struct QueryResponse {
    sql: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<QueryResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub struct Server;

impl Server {
    pub async fn run(db_url: &str, host: &str, port: u16) -> Result<(), Error> {
        let db = Db::connect(db_url).await?;
        let schema = db.schema().await?;

        let state = Arc::new(AppState { db, schema });

        let app = Router::new()
            .route("/health", get(health))
            .route("/query", post(query))
            .route("/schema", get(get_schema))
            .layer(CorsLayer::permissive())
            .with_state(state);

        let addr = format!("{host}:{port}");
        println!("server running at http://{addr}");

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| Error::Server(e.to_string()))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| Error::Server(e.to_string()))?;

        Ok(())
    }
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn get_schema(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "schema": state.schema }))
}

async fn query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QueryRequest>,
) -> (StatusCode, Json<QueryResponse>) {
    // get claude ready
    let claude = match Claude::new(None) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(QueryResponse {
                    sql: String::new(),
                    result: None,
                    warning: None,
                    error: Some(e.to_string()),
                }),
            );
        }
    };

    // generate the sql
    let sql = match claude.generate_sql(&req.prompt, &state.schema).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(QueryResponse {
                    sql: String::new(),
                    result: None,
                    warning: None,
                    error: Some(e.to_string()),
                }),
            );
        }
    };

    // check if it's safe
    let safety = Safety::check(&sql);
    if safety.is_dangerous && !req.run_dangerous {
        return (
            StatusCode::BAD_REQUEST,
            Json(QueryResponse {
                sql,
                result: None,
                warning: None,
                error: Some(format!("blocked: {}", safety.reason)),
            }),
        );
    }

    // just return sql if dry run
    if req.dry_run {
        return (
            StatusCode::OK,
            Json(QueryResponse {
                sql,
                result: None,
                warning: safety.warning,
                error: None,
            }),
        );
    }

    // run it
    match state.db.execute(&sql).await {
        Ok(result) => (
            StatusCode::OK,
            Json(QueryResponse {
                sql,
                result: Some(result),
                warning: safety.warning,
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(QueryResponse {
                sql,
                result: None,
                warning: safety.warning,
                error: Some(e.to_string()),
            }),
        ),
    }
}
