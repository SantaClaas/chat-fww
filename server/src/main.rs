use std::sync::Arc;

use crate::actor::{delivery_service, websocket};
use axum::http::StatusCode;
use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    http::{HeaderValue, Method},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod actor;

#[derive(Clone, Default)]
struct AppState {
    delivery_service: delivery_service::Handle,
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
    // SPA setup
    // Not used during development where vite hosts the frontend and we use CORS
    let serve_client = ServeDir::new("./client")
        // If the route is a client side navigation route, this will serve the app and let the app router take over the
        // path handling after the app is loaded
        .not_found_service(ServeFile::new("./client/index.html"));

    let mut app = Router::new()
        .route("/messages/:name", get(websocket_handler))
        .route("/users", get(get_users));

    // Not looking nice, but functional to have CORS only in development
    #[cfg(debug_assertions)]
    {
        app = app.layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        );
    }

    let app = app
        .fallback_service(serve_client)
        .with_state(Default::default());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn get_users(State(state): State<AppState>) -> Result<Json<Arc<[Arc<str>]>>, StatusCode> {
    let result = state.delivery_service.get_users().await;
    match result {
        Ok(users) => Ok(Json(users)),
        Err(error) => {
            tracing::error!("Error getting users: {:?}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn websocket_handler(
    Path(name): Path<String>,
    websocket: WebSocketUpgrade,
    state: State<AppState>,
) -> impl IntoResponse {
    websocket.on_upgrade(move |socket| create_actor(socket, state, name.into()))
}

async fn create_actor(stream: WebSocket, State(state): State<AppState>, name: Arc<str>) {
    let result = state.delivery_service.get_or_insert(name).await;

    let user = match result {
        Ok(user) => user,
        Err(error) => {
            tracing::error!(
                "Error getting/creating user for websocket connection: {:?}",
                error
            );
            return;
        }
    };

    let socket = websocket::Handle::new(stream, user.clone());
    let result = user.add_socket(socket).await;
    let Err(error) = result else {
        return;
    };
    tracing::error!("Error adding socket: {}", error);
}
