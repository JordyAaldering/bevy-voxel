use std::net::SocketAddr;

use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::{Message, WebSocket}},
    response::{Html, Response},
    routing,
};
use bevy::{
    prelude::*,
    diagnostic::{DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
};
use serde::Serialize;
use tokio::sync::watch;

// Resource used by Bevy to publish the latest snapshot.
#[derive(Resource)]
pub struct TelemetryPublisher(pub watch::Sender<Telemetry>);

#[derive(Clone, Debug, Default, Serialize)]
pub struct Telemetry {
    fps: f64,
    frame_time_ms: f64,
    entity_count: f64,
}

pub async fn run_server(receiver: watch::Receiver<Telemetry>) {
    let app = Router::new()
        .route("/", routing::get(index))
        .route("/ws", routing::get(websocket))
        .with_state(receiver);

    let address = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .unwrap();

    info!("Telemetry dashboard: http://{address}");

    axum::serve(listener, app)
        .await
        .unwrap();
}

pub fn collect_telemetry(
    diagnostics: Res<DiagnosticsStore>,
    publisher: Res<TelemetryPublisher>,
) {
    let value = |path| {
        diagnostics.get(path)
            .and_then(|d| d.smoothed())
            .unwrap_or_default()
    };

    let snapshot = Telemetry {
        fps: value(&FrameTimeDiagnosticsPlugin::FPS),
        frame_time_ms: value(&FrameTimeDiagnosticsPlugin::FRAME_TIME),
        entity_count: value(&EntityCountDiagnosticsPlugin::ENTITY_COUNT),
    };

    publisher.0.send_replace(snapshot);
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../ws/telemetry.html"))
}

async fn websocket(upgrade: WebSocketUpgrade, receiver: State<watch::Receiver<Telemetry>>) -> Response {
    upgrade.on_upgrade(move |socket| stream_telemetry(socket, receiver.0))
}

async fn stream_telemetry(mut socket: WebSocket, mut receiver: watch::Receiver<Telemetry>) {
    loop {
        let snapshot = receiver.borrow_and_update().clone();

        let Ok(json) = serde_json::to_string(&snapshot) else {
            break;
        };

        if socket.send(Message::Text(json.into())).await.is_err() {
            break;
        }

        // Wait for a new snapshot rather than busy-polling
        if receiver.changed().await.is_err() {
            break;
        }
    }
}
