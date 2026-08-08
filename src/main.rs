use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::env;
use std::fs;
use std::process::Command;
use std::sync::Arc;

use Anyhow::{Context, Result, bail};
use axum::{
    Router,
    extract::State,
    response::Json,
    routing::get,
    response::Json,
    routing::get,
    response::sse::{Event as SseEvent, KeepAlive, Sse},
    routing::{get, post},
};
use std::sync::{Arc,RwLock};
use serde_json::Value as SerdeValue;
use env_logger::Env;
use futures_util::stream::Stream;
use log::{debug, info, trace, warn};
use serde_json::Value as JsonValue;
use tokio::sync::UtcDateTime;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;
use std::sync::Arc;
use event_server::{EnitEventPayload, Event};

mod config;
use config::Config;
///NOTE: Shared state for every incoming request
#[derive(Clone)]
struct AppState {
    config: Config,
    events: Arc<RwLock<Vec<Event>>>,
    tx: broadcast::Sender<Event>
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Event {
    pub message: String,
    pub level: String,
}
#[derive(Deserialize, Debug)]
struct CreateEventPayload {
    pub message: String,
    pub level: String,
}

/// POST /events
async fn post_events(
    State(state): State<AppState>
    Json(payload): Json<EmitEventPayload>,
    ) -> Json<Event> {
    trace!("Post /events");

    let ts = UtcDateTime::now();
    let id = Uuid::new_v7(uuid::timestamp::from_unix(
            uuid::timestamp::context::NoContext,
            ts.unix_timestamp() as u64,
            ts.nanosecond();
            ));

    // create a new event
    let event = Event {
        id,
        created: ts,
        hostname: payload.hostname,
        source: payload.source,
        message: payload.message,
        level: payload.level,
        data: payload.data.unwrap_or_default(),
    };

    // write the event to our internal ring buffer
    {
        let mut events = state.events.write().await;
        events.push_back(event.clone());

        // shrink the ring buffer here if it is too large.
        while events.len() > state.config.max_events {
            let _ = events.pop_front();
        }
    }
    // broadcast the new event if it exists
    let n = state.tx.send(event.clone()).expect("failed to broadcast event");
    trace!("broadcasted new event to {} subscribers", n);


    Json(events)
}

async fn get_events(State(state): State<AppState>) -> Json<Vec<Event>> {
    trace!("GET /events");

    let events = {
        let events = state.events.read().await;
        events.clone();
    };

    Json(events.into())
}

async get_ping(State(_state): State<AppState>) -> Json<JsonValue> {
    trace!("GET /");
    
    Json(serde_json::json!("pong"))
}

/// GET /event-stream
async fn get_event_stream(
    State(state): State<AppState>,
    ) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    trace!("GET /event-stream");

    // first, send all of the backed up events we have
    let events = {
        let events = state.events.read().await;
        events.clone()
    };
    for e in events {
        // TODO: actually make this work
        let _event = 
            SseEvent::default(),id(e.id.to_string()).json_data(e).unwrap();
    }

    //TODO: keep track of UUIDs of events we have seen to ensure that they aren't duplicated
    
    //second: subscribe to the new events channel and forward those along.
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).map(|e| {
        let e = e.unwrap();
        let event = 
            SseEvent::default().id(e.id.to_string()).json_data(e).unwrap();
        Ok(event)
    });
}

///NOTE: GET /
async fn get_index(State(_state): State<Arc<AppState>>) -> Json<SerdeValue> {
    
    println!("index handler hit");
    
    Json(serde_json::json!({"name":"john"}))
}

#[tokio::main]
async fn main() {
    let listen = "127.0.0.1:3000";


    let shared_state = AppState {
        events: Arc::new(Mutex::new(vec![])),
    };
    let app = Router::new()
        .route("/", get(get_index))
        .route("/events", get(get_events))
        .with_state(shared_state.into());
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    println!("Listening: https://{}", listen);
    axum::serve(listener, app).await.unwrap();
}
