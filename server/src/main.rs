mod endpoints;

use axum::{extract::Extension, routing::get, Router};
use std::{net::SocketAddr, sync::mpsc, sync::Arc};
use structopt::StructOpt;
use tokio::sync::broadcast;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, StructOpt)]
#[structopt(about = "axum chat example server running options")]
struct Opts {
    #[structopt(
        short = "a",
        long,
        default_value = "0.0.0.0:3000",
        help = "listen address"
    )]
    listen_address: String,
    #[structopt(
        short = "c",
        long,
        default_value = "test",
        help = "dragonfly PUBSUB channel name for chat"
    )]
    chat_channel_name: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let options: Opts = Opts::from_args();

    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379/0".to_owned());
    let redis_pool =
        dragonfly::new_pool(redis_url, 4).expect("redis connection pool must be initialized.");
    let server_id = domain::models::ServerId::new();
    tracing::debug!("server_id: {:?}", &server_id);
    let channel_name = options.chat_channel_name.clone();
    let (broadcaster, _) = broadcast::channel(100);
    let (publisher, receiver) = mpsc::sync_channel(100);

    // start subscriber service async
    let service = domain::services::chat_room::ChatRoomSubscriberService::new(
        redis_pool.clone(),
        server_id.clone(),
        channel_name.clone(),
        broadcaster.clone(),
    );
    std::thread::spawn(move || service.start());

    // start publisher service async
    let service = domain::services::chat_room::ChatRoomPublisherService::new(
        redis_pool.clone(),
        server_id.clone(),
        channel_name.clone(),
        broadcaster.clone(),
        receiver,
    );
    std::thread::spawn(move || service.start());

    let app_state = Arc::new(endpoints::websocket::AppState::new(
        redis_pool,
        broadcaster,
        publisher,
    ));
    let static_html_routes = Router::new().route("/", get(endpoints::index::handler));
    let websocket_routes = Router::new()
        .route("/websocket", get(endpoints::websocket::handler))
        .layer(Extension(app_state));
    let app = Router::new()
        .merge(static_html_routes)
        .merge(websocket_routes);

    let addr: SocketAddr = options.listen_address.as_str().parse().unwrap();
    tracing::debug!("listening on {}", &options.listen_address);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
