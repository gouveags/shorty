use crate::application::{
    create_short_url::CreateShortUrlService, get_short_url_stats::GetShortUrlStatsService,
    resolve_short_url::ResolveShortUrlService,
};

#[derive(Clone)]
pub struct AppState {
    pub public_base_url: String,
    pub create_short_url: CreateShortUrlService,
    pub resolve_short_url: ResolveShortUrlService,
    pub get_short_url_stats: GetShortUrlStatsService,
}
