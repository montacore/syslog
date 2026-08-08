use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use time::UtcDateTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ValueEnum, Debug, Clone)]
#[serde(rename_all = "lowercase")]
#[clap(rename_all = "lowercase")]
pub enum EventLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Critical,
}

impl fmt::Display for EventLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_names()
            .fmt(f)
    }
}

#[derive(Serialize, Deserialize, ValueEnum, Debug, Clone)]
pub struct Event {
    // Fields generated automatically by the server 
    // UUID (v7) for the event 
    pub id: Uuid,

    // Timestamp for the event
    pub created: UtcDeteTime,

    // Fields taken from the client
    pub hostname: String,

    // Name of source ( like "nginx" or "nagios")
    pub source: String,

    // Severity Level
    pub level: EventLevel,

    // any string message
    pub message: String,

    // any arbitraryJSON data
    pub data: JsonValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmitEventPayload {
    pub source: String,
    pub hostname: String, 
    pub level: EventLevel,
    pub message: String,
    pub data: Option<JsonValue>,
}
