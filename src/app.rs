use std::error::Error;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    router::{Router, RouterBuilderDiscoverExt},
    runtime::RouterBuilderRuntimeExt,
    session::RouterBuilderSessionExt,
};

use crate::{
    auth::AuthManager,
    config::{AuthConfig, DatabaseConfig},
    db,
};

pub(crate) async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let database_config = DatabaseConfig::load()?;

    let auth_config = AuthConfig::load()?;

    let database = db::connect(&database_config, "topcoat-poc").await?;

    db::healthcheck(&database).await?;

    let auth = AuthManager::from_config(&auth_config);

    // The selected auth driver owns the complete session-security profile.
    // There is no independent runtime switch that can weaken these settings.
    let session_config = auth.session_config();

    let router = Router::builder()
        .cookies()
        .sessions(session_config)
        .runtime()
        .discover()
        .app_context(database)
        .app_context(auth)
        .assets(AssetBundle::load().expect("Topcoat asset bundle was not found"))
        .build();

    topcoat::start(router).await?;

    Ok(())
}
