use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::error;
use tracing::info;
use tracing::info_span;
use tracing::Instrument;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;
use wtransport::endpoint::IncomingSession;
use wtransport::Endpoint;
use wtransport::Identity;
use wtransport::RecvStream;
use wtransport::ServerConfig;

use crate::services::flv_service::FlvService;
use crate::AppState;

pub async fn init_transport(state: Arc<AppState>) -> Result<()> {
    init_logging();

    let identity = Identity::self_signed_builder()
        .subject_alt_names(&["localhost", "127.0.0.1", "::1"])
        .from_now_utc()
        .validity_days(14)
        .build()
        .unwrap();

    let cert_digest = identity.certificate_chain().as_slice()[0].hash();

    println!("{:?}", cert_digest);

    let config = ServerConfig::builder()
        .with_bind_default(4433)
        .with_identity(identity)
        .keep_alive_interval(Some(Duration::from_secs(3)))
        .build();

    let server = Endpoint::server(config)?;

    info!("Server ready!");

    for id in 0.. {
        let incoming_session = server.accept().await;
        tokio::spawn(
            handle_connection(incoming_session, state.clone())
                .instrument(info_span!("Connection", id)),
        );
    }

    Ok(())
}

async fn handle_connection(incoming_session: IncomingSession, state: Arc<AppState>) {
    let result = handle_connection_impl(incoming_session, state.clone()).await;
    error!("{:?}", result);
}

async fn handle_connection_impl(
    incoming_session: IncomingSession,
    state: Arc<AppState>,
) -> Result<()> {
    let mut buffer = vec![0; 231072].into_boxed_slice();

    info!("Waiting for session request...");

    let session_request = incoming_session.await?;

    info!(
        "New session: Authority: '{}', Path: '{}'",
        session_request.authority(),
        session_request.path()
    );

    let connection = session_request.accept().await?;

    info!("Waiting for data from client...");

    loop {
        tokio::select! {
            stream = connection.accept_bi() => {
                let mut stream = stream?;
                info!("Accepted BI stream");

                let bytes_read = match stream.1.read(&mut buffer).await? {
                    Some(bytes_read) => bytes_read,
                    None => continue,
                };

                let str_data = std::str::from_utf8(&buffer[..bytes_read])?;

                info!("Received (bi) '{str_data}' from client");

                stream.0.write_all(b"ACK").await?;
            }
            stream = connection.accept_uni() => {
                let mut stream = stream?;
                let state_clone = Arc::clone(&state);

                // 写这里会造成拉流那边lock阻塞，不知道为啥
                // let mut cache = state.gop_cache.write().await;

                tokio::spawn(async move {
                    process_media_stream(&mut stream, state_clone).await;
                });

            }
            dgram = connection.receive_datagram() => {
                let dgram = dgram?;
                let str_data = std::str::from_utf8(&dgram)?;

                info!("Received (dgram) '{str_data}' from client");

                connection.send_datagram(b"ACK")?;
            }
        }
    }
}

fn init_logging() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_env_filter(env_filter)
        .init();
}

async fn process_media_stream(stream: &mut RecvStream, state: Arc<AppState>) -> Result<()> {
    let mut buffer_dd = vec![0; 1024 * 1024].into_boxed_slice(); // 64KB buffer

    while let Some(bytes_read) = stream.read(&mut buffer_dd).await? {
        let media_data = &buffer_dd[0..bytes_read].to_vec();

        // info!("buffer_size {:?}", bytes_read);

        let data_size;

        if bytes_read > 13 {
            data_size = (media_data[1] as u32) << 16
                | (media_data[2] as u32) << 8
                | (media_data[3] as u32)
                | 0;

            if data_size > 1000000 {
                info!("data {:?}", media_data);
                info!("data_size {:?}", data_size);
            }
        }

        // 使用scope来确保锁的及时释放
        {
            let mut cache = state.gop_cache.write().await;

            if FlvService::is_metadata(&media_data) {
                info!("是metadata {:?}", &media_data);
                cache.push_metadata(media_data.clone());
            }
            if FlvService::is_sequence_header(&media_data) {
                cache.push_sequence_header(media_data.clone());
            }
            if FlvService::is_key_frame(&media_data) {
                info!("是keyframe");
                cache.push_key_frame(media_data.clone());
            }
        } // 锁在这里释放

        // 广播数据给所有接收者
        if state.broadcaster.receiver_count() > 0 {
            let _ = state.broadcaster.send(media_data.clone());
        }
    }

    Ok(())
}
