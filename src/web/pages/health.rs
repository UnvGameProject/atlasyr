use sqlx::PgPool;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{error::service_unavailable, route},
};

use crate::application::health::database_ready;

/// Process/router liveness.
///
/// This intentionally has no database dependency. A temporary database outage
/// must make the application unready without falsely declaring the process dead.
#[route(GET "/health/live")]
async fn live() -> Result<&'static str> {
    Ok("live")
}

/// Application readiness.
///
/// PostgreSQL is currently the application's required runtime dependency. A
/// failed readiness check returns 503 without exposing backend error detail.
#[route(GET "/health/ready")]
async fn ready(cx: &Cx) -> Result<&'static str> {
    let pool = app_context::<PgPool>(cx);

    if !database_ready(pool).await {
        return Err(service_unavailable(1).into());
    }

    Ok("ready")
}
