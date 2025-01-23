use std::{result, sync::Arc};
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::info;

pub mod db;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod services;

#[derive(Clone)]
pub struct AppState {
    pub broadcaster: broadcast::Sender<Vec<u8>>,
    pub gop_cache: Arc<RwLock<GopCache>>,
}

impl AppState {
    pub fn new(buffer_size: usize) -> Self {
        let (broadcaster, _) = broadcast::channel(buffer_size);
        Self {
            broadcaster,
            gop_cache: Arc::new(RwLock::new(GopCache::new())), // 例如，10 MB 缓存
        }
    }
}

// 关键帧缓存的定义
pub struct GopCache {
    metadata: Vec<u8>,
    sequence_header: Vec<u8>,
    key_frame: Vec<u8>,
    aac_sequence: Vec<u8>,
}

impl GopCache {
    pub fn new() -> Self {
        Self {
            metadata: Vec::new(),
            sequence_header: Vec::new(),
            key_frame: Vec::new(),
            aac_sequence: Vec::new(),
        }
    }

    pub fn push_metadata(&mut self, frame: Vec<u8>) {
        self.metadata = frame;
        info!("存储的metadatadsdsds {:?}", self.metadata);
    }

    pub fn push_key_frame(&mut self, frame: Vec<u8>) {
        self.key_frame = frame;
    }

    pub fn push_sequence_header(&mut self, frame: Vec<u8>) {
        self.sequence_header = frame
    }

    pub fn push_aac_sequence(&mut self, frame: Vec<u8>) {
        self.aac_sequence = frame
    }

    pub fn get_metadata(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.metadata);
        result
    }

    pub fn get_sequence(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.sequence_header);
        result
    }

    pub fn get_aac_sequence(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.aac_sequence);
        result
    }

    pub fn get_key(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.key_frame);
        result
    }
}
