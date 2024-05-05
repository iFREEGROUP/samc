use axum::extract::FromRef;
use segment::Segment;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub(crate) config:Config,
    pub(crate) model: Segment,
}

impl FromRef<AppState> for Config {
    fn from_ref(input: &AppState) -> Self {
        input.config.clone()
    }
}

impl FromRef<AppState> for Segment {
    fn from_ref(input: &AppState) -> Self {
        input.model.clone()
    }
}