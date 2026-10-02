use sqlx::PgPool;
use uuid::Uuid;

use super::super::{ContactAuditAction, record_contact_event};

#[sqlx::test]
async fn audit_schema_contains_only_minimal_fields(pool: PgPool) {
    let columns: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT column_name
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'audit_events'
        ORDER BY ordinal_position
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("audit schema inspection should succeed");

    assert_eq!(
        columns,
        vec![
            "id",
            "actor_user_id",
            "action",
            "resource_type",
            "resource_id",
            "created_at",
        ],
    );

    let foreign_key_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM information_schema.table_constraints
        WHERE table_schema = 'public'
          AND table_name = 'audit_events'
          AND constraint_type = 'FOREIGN KEY'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("audit foreign-key inspection should succeed");

    assert_eq!(
        foreign_key_count, 0,
        "durable actor/resource identifiers must outlive source rows",
    );
}

#[sqlx::test]
async fn audit_action_and_resource_type_are_database_constrained(pool: PgPool) {
    let actor_user_id = Uuid::new_v4();
    let resource_id = Uuid::new_v4();

    record_contact_event(
        &pool,
        actor_user_id,
        ContactAuditAction::Create,
        resource_id,
    )
    .await
    .expect("typed audit event should persist");

    let invalid_action = sqlx::query(
        r#"
        INSERT INTO public.audit_events (
            id,
            actor_user_id,
            action,
            resource_type,
            resource_id
        )
        VALUES ($1, $2, 'read', 'contact', $3)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(actor_user_id)
    .bind(resource_id)
    .execute(&pool)
    .await;

    assert!(
        invalid_action.is_err(),
        "database must reject unknown audit actions",
    );

    let invalid_resource_type = sqlx::query(
        r#"
        INSERT INTO public.audit_events (
            id,
            actor_user_id,
            action,
            resource_type,
            resource_id
        )
        VALUES ($1, $2, 'create', 'user', $3)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(actor_user_id)
    .bind(resource_id)
    .execute(&pool)
    .await;

    assert!(
        invalid_resource_type.is_err(),
        "database must reject non-Contacts resource types",
    );
}
