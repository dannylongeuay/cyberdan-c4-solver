use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use c4_solver::bitboard::{Bitboard, MoveError, WIDTH};
use c4_solver::player::Difficulty;
use c4_solver::solver;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tower_http::cors::CorsLayer;

const MAX_TIMEOUT_SECS: f64 = 10.0;
const DEFAULT_TIMEOUT_SECS: f64 = 5.0;

#[derive(Deserialize)]
struct SolveRequest {
    moves: Vec<usize>,
    #[serde(default = "default_difficulty")]
    difficulty: Difficulty,
    #[serde(default = "default_timeout")]
    timeout: f64,
}

fn default_difficulty() -> Difficulty {
    Difficulty::Normal
}

fn default_timeout() -> f64 {
    DEFAULT_TIMEOUT_SECS
}

#[derive(Serialize)]
struct SolveResponse {
    column: usize,
    status: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn error_response(status: StatusCode, msg: impl Into<String>) -> impl IntoResponse {
    (status, Json(ErrorResponse { error: msg.into() }))
}

async fn health() -> &'static str {
    "ok"
}

async fn solve(Json(req): Json<SolveRequest>) -> Result<Json<SolveResponse>, impl IntoResponse> {
    let mut board = Bitboard::new();

    for (i, &col) in req.moves.iter().enumerate() {
        if col >= WIDTH {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                format!("move {i}: column {col} is out of range (0-6)"),
            ));
        }
        match board.play(col) {
            Ok(()) => {}
            Err(MoveError::ColumnFull) => {
                return Err(error_response(
                    StatusCode::BAD_REQUEST,
                    format!("move {i}: column {col} is full"),
                ));
            }
            Err(MoveError::OutOfRange) => {
                return Err(error_response(
                    StatusCode::BAD_REQUEST,
                    format!("move {i}: column {col} is out of range (0-6)"),
                ));
            }
        }

        if board.is_winning() {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                format!("game already over after move {i}"),
            ));
        }
        if board.is_draw() {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "game already ended in a draw".to_string(),
            ));
        }
    }

    let depth = req.difficulty.depth();
    let timeout_secs = req.timeout.clamp(0.1, MAX_TIMEOUT_SECS);
    let timeout = Duration::from_secs_f64(timeout_secs);

    let col = tokio::task::spawn_blocking(move || solver::best_move(&board, depth, timeout))
        .await
        .expect("solver task panicked");

    let mut after = board;
    after.play(col).expect("solver returned invalid column");

    let status = if after.is_winning() {
        "win"
    } else if after.is_draw() {
        "draw"
    } else {
        "ongoing"
    };

    Ok(Json(SolveResponse {
        column: col,
        status,
    }))
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");

    let app = Router::new()
        .route("/health", get(health))
        .route("/solve", post(solve))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");

    eprintln!("c4-api listening on {addr}");
    axum::serve(listener, app).await.expect("server error");
}
