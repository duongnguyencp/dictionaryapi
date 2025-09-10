use actix_governor::{
    Governor, GovernorConfigBuilder, PeerIpKeyExtractor, governor::middleware::NoOpMiddleware,
};

pub fn init_rl() -> Governor<PeerIpKeyExtractor, NoOpMiddleware> {
    let governor_cof = GovernorConfigBuilder::default()
        .seconds_per_request(5)
        .burst_size(10)
        .finish()
        .unwrap();
    Governor::new(&governor_cof)
}
