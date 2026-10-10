use sqlx::PgPool;

use crate::db;

/// Readiness dependency check.
///
/// This remains an application/infrastructure seam rather than allowing the
/// HTTP adapter to call persistence infrastructure directly.
pub(crate) async fn database_ready(pool: &PgPool) -> bool {
    db::healthcheck(pool).await.is_ok()
}
