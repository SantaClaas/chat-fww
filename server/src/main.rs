use std::{collections::HashMap, sync::Arc};

use actor::user;
use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    http::{HeaderValue, Method},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod actor;

#[derive(Clone, Default)]
struct AppState {
    users: Arc<Mutex<HashMap<Arc<str>, user::Handle>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Setting up");

    let app = Router::new()
        .route("/messages/:name", get(websocket_handler))
        .route("/users", get(get_users))
        .layer(
            CorsLayer::new()
                // .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                // .allow_origin("localhost:1421".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        )
        .with_state(AppState {
            users: Default::default(),
        });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn get_users(State(state): State<AppState>) -> Json<Vec<String>> {
    let users = state.users.lock().await;
    let users = users
        .keys()
        .map(|user| user.to_string())
        .collect::<Vec<_>>();

    dbg!(&users);

    Json(users)
}

async fn websocket_handler(
    Path(name): Path<String>,
    websocket: WebSocketUpgrade,
    state: State<AppState>,
) -> impl IntoResponse {
    websocket.on_upgrade(move |socket| create_actor(socket, state, name.into()))
}

// 2/3e, duck2duck encryption, melt
async fn create_actor(stream: WebSocket, State(state): State<AppState>, name: Arc<str>) {
    let mut users = state.users.lock().await;
    let entry = users.entry(name.clone());
    // Get or create user actor
    let user = entry.or_insert_with(|| user::Handle::new(name));

    // Create websocket actor
    let socket = actor::websocket::Handle::new(stream, user.clone());
    let result = user.add_socket(socket).await;

    if let Err(error) = result {
        tracing::error!("Error adding socket: {}", error);
    }
}
