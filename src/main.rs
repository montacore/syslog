use axum::{
    Router,
    extract::State,
    response::Json,
    routing::get,
    response::Json,
    routing::get,
    
};
use std::sync::{Arc,RwLock};
use serde_json::Value as SerdeValue;

use std::sync::Arc;
   
struct AppState {
    events: Arc<RwLock<Vec<Event>>>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Event {
    pub message: String,
    pub level: String,
}
//TODO: GET /events
async fn get_events(State(state): State<AppState>) -> Json<Vec<Event>>{
    let events = state.events.read().expect("failed to acquire lock.");

    Json(*events);
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
    );
    let app = Router::new()
        .route("/", get(get_index))
        .route("/events", get(get_events))
        .with_state(shared_state);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    println!("Listening: https://{}", listen);
    axum::serve(listener, app).await.unwrap();
}
