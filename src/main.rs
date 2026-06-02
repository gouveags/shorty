use std::{net::SocketAddr, sync::Arc};

use shorty::{
    app::state::AppState,
    application::{
        create_short_url::CreateShortUrlService, get_short_url_stats::GetShortUrlStatsService,
        resolve_short_url::ResolveShortUrlService,
    },
    config::Config,
    infrastructure::{
        code::base62_generator::Base62CodeGenerator,
        postgres::short_url_repository::PostgresShortUrlRepository,
        redis::url_cache::RedisUrlCache,
    },
    interfaces::http::routes::router,
    ports::{
        code_generator::CodeGenerator, short_url_repository::ShortUrlRepository,
        url_cache::UrlCache,
    },
    workers::access_count_flusher::AccessCountFlusher,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shorty=debug,tower_http=debug".into()),
        )
        .init();

    let config = Config::from_env()?;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let repository: Arc<dyn ShortUrlRepository> = Arc::new(PostgresShortUrlRepository::new(pool));
    let cache: Arc<dyn UrlCache> = Arc::new(RedisUrlCache::new(
        &config.redis_url,
        config.cache_ttl_seconds,
    )?);
    let code_generator: Arc<dyn CodeGenerator> =
        Arc::new(Base62CodeGenerator::new(config.code_length));

    let flusher = AccessCountFlusher::new(
        Arc::clone(&repository),
        Arc::clone(&cache),
        config.access_flush_interval,
    );
    tokio::spawn(flusher.run());

    let state = AppState {
        public_base_url: config.public_base_url,
        create_short_url: CreateShortUrlService::new(
            Arc::clone(&repository),
            Arc::clone(&cache),
            Arc::clone(&code_generator),
        ),
        resolve_short_url: ResolveShortUrlService::new(Arc::clone(&repository), Arc::clone(&cache)),
        get_short_url_stats: GetShortUrlStatsService::new(repository, cache),
    };

    let address: SocketAddr = config.bind_address.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;

    tracing::info!(%address, "shorty listening");

    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
