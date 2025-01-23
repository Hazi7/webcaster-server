use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket},
        State,
    },
};
use futures::{Stream, StreamExt};
use tracing::info;

use crate::AppState;

use super::flv_service::FlvService;

pub struct Livestream {
    flv_service: FlvService,
}

impl Livestream {
    pub async fn handle_push_client(mut socket: WebSocket, state: Arc<AppState>) {
        while let Some(Ok(msg)) = socket.next().await {
            match msg {
                Message::Binary(data) => {
                    let mut cache = state.gop_cache.write().await;

                    if FlvService::is_metadata(&data) {
                        cache.push_metadata(data.clone());
                    }

                    if FlvService::is_sequence_header(&data) {
                        cache.push_sequence_header(data.clone());
                    }

                    if FlvService::is_aac_sequence(&data) {
                        cache.push_aac_sequence(data.clone());
                    }

                    // 是关键帧
                    if FlvService::is_key_frame(&data) {
                        cache.push_key_frame(data.clone());
                    }

                    if state.broadcaster.receiver_count() > 0 {
                        // 将数据发送到广播通道

                        let _ = state.broadcaster.send(data.clone());
                    }
                }
                Message::Close(_) => {
                    break;
                }
                Message::Text(text) => {
                    println!("Received text message: {}", text);
                }
                _ => {}
            }
        }
    }
    pub async fn create_flv_stream(
        state: State<Arc<AppState>>,
        mut base_timestamp: Option<u32>,
        mut first_timestamp: Option<u32>,
    ) -> impl Stream<Item = Result<Bytes, std::io::Error>> {
        let gop = state.gop_cache.read().await;
        let metadata = gop.get_metadata();
        let aac_sequence = gop.get_aac_sequence();
        let sequence_header = gop.get_sequence();
        let mut key_frame = gop.get_key();

        if key_frame.len() > 0 {
            key_frame[6] = (0 & 0xff) as u8;
            key_frame[5] = ((0 >> 8) & 0xff) as u8;
            key_frame[4] = ((0 >> 16) & 0xff) as u8;
        } else {
            info!("无缓存关键帧");
        }

        let broadcaster = state.broadcaster.clone();

        async_stream::stream! {

            info!("发送流");

            // 发送 FLV header
            yield Ok(FlvService::create_flv_header());

            yield Ok(Bytes::from(metadata));

            yield Ok(Bytes::from(aac_sequence));

            yield Ok(Bytes::from(sequence_header));

            yield Ok(Bytes::from(key_frame));

            let mut receiver = broadcaster.subscribe();
            while let Ok(data) = receiver.recv().await {

                if FlvService::is_sequence_header(&data) {
                    continue;
                }

                let process_data = FlvService::process_video_timestamp(data.clone(), &mut base_timestamp, &mut first_timestamp);
                // 发送 FLV 数据
                yield Ok(Bytes::from(process_data.to_vec()));
            }
        }
    }
}
