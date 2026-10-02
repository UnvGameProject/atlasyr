use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    application::authorization::{AuthorizationActor, load_authorization_actor},
    auth::password::hash_password,
    db::models::{contact::Contact, user::User},
    domain::{
        contact::{ContactError, CreateContactInput},
        user::CreateUserInput,
    },
};

use super::{
    CreateContactCommand, CreateContactOutcome, DeleteContactCommand, DeleteContactOutcome,
    UpdateContactCommand, UpdateContactOutcome, create_contact, delete_contact, update_contact,
};

async fn operator_actor(pool: &PgPool, email: &str) -> (AuthorizationActor, Uuid) {
    let input = CreateUserInput::from_untrusted(email)
        .validate()
        .expect("audit test operator email should be valid");

    let password_hash = hash_password("contacts-audit-test-password")
        .expect("audit test password hashing should succeed");

    let user = User::create(pool, &input, password_hash)
        .await
        .expect("audit test operator should persist");

    let actor = load_authorization_actor(pool, user.id)
        .await
        .expect("audit test authorization lookup should succeed")
        .expect("audit test authorization actor should exist");

    (actor, user.id)
}

async fn persist_test_contact(pool: &PgPool, name: &str, email: &str) -> Contact {
    let input = CreateContactInput::from_untrusted(name, email)
        .validate()
        .expect("test contact should validate");

    Contact::create(pool, &input)
        .await
        .expect("test contact should persist")
}

async fn audit_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM public.audit_events
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("audit count should succeed")
}

#[sqlx::test]
async fn successful_contact_mutations_are_durably_attributed(pool: PgPool) {
    let (actor, actor_user_id) = operator_actor(&pool, "audit-success-operator@example.com").await;

    let create = create_contact(
        &pool,
        actor
            .contacts_create()
            .expect("operator should receive create grant"),
        CreateContactCommand::new("Ada Lovelace".to_owned(), "ada@example.com".to_owned()),
    )
    .await
    .expect("contact create should succeed");

    assert_eq!(create, CreateContactOutcome::Created,);

    let contacts = Contact::find_all(&pool)
        .await
        .expect("created contact should be readable");

    assert_eq!(contacts.len(), 1);

    let contact_id = contacts[0].id;

    let update = update_contact(
        &pool,
        actor
            .contacts_update()
            .expect("operator should receive update grant"),
        UpdateContactCommand::new(
            contact_id.to_string(),
            "Ada Byron".to_owned(),
            "ada.byron@example.com".to_owned(),
        ),
    )
    .await
    .expect("contact update should succeed");

    assert_eq!(
        update,
        UpdateContactOutcome::Updated {
            email: "ada.byron@example.com".to_owned(),
        },
    );

    let delete = delete_contact(
        &pool,
        actor
            .contacts_delete()
            .expect("operator should receive delete grant"),
        DeleteContactCommand::new(contact_id.to_string()),
    )
    .await
    .expect("contact delete should succeed");

    assert_eq!(delete, DeleteContactOutcome::Deleted,);

    assert!(
        Contact::find_by_id(&pool, contact_id,)
            .await
            .expect("deleted contact lookup should succeed",)
            .is_none(),
        "contact should be deleted while audit history remains",
    );

    let rows: Vec<(Uuid, String, String, Uuid)> = sqlx::query_as(
        r#"
        SELECT
            actor_user_id,
            action,
            resource_type,
            resource_id
        FROM public.audit_events
        ORDER BY action
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("audit rows should be readable");

    assert_eq!(rows.len(), 3);

    let expected_actions = ["create", "delete", "update"];

    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row.0, actor_user_id);
        assert_eq!(row.1, expected_actions[index],);
        assert_eq!(row.2, "contact");
        assert_eq!(row.3, contact_id);
    }
}

#[sqlx::test]
async fn expected_unsuccessful_outcomes_do_not_create_audit_rows(pool: PgPool) {
    let (actor, _) = operator_actor(&pool, "audit-outcome-operator@example.com").await;

    let invalid = create_contact(
        &pool,
        actor
            .contacts_create()
            .expect("operator should receive create grant"),
        CreateContactCommand::new("".to_owned(), "valid@example.com".to_owned()),
    )
    .await
    .expect("validation failure should not be backend failure");

    assert!(matches!(invalid, CreateContactOutcome::Validation(_),));

    let source = persist_test_contact(&pool, "Existing Contact", "existing@example.com").await;

    let duplicate = create_contact(
        &pool,
        actor
            .contacts_create()
            .expect("operator should receive create grant"),
        CreateContactCommand::new("Duplicate".to_owned(), source.email.clone()),
    )
    .await
    .expect("duplicate should remain typed outcome");

    assert_eq!(duplicate, CreateContactOutcome::EmailAlreadyExists,);

    let missing_id = Uuid::new_v4();

    let update = update_contact(
        &pool,
        actor
            .contacts_update()
            .expect("operator should receive update grant"),
        UpdateContactCommand::new(
            missing_id.to_string(),
            "Missing".to_owned(),
            "missing@example.com".to_owned(),
        ),
    )
    .await
    .expect("missing update should remain typed outcome");

    assert_eq!(update, UpdateContactOutcome::NotFound,);

    let delete = delete_contact(
        &pool,
        actor
            .contacts_delete()
            .expect("operator should receive delete grant"),
        DeleteContactCommand::new(missing_id.to_string()),
    )
    .await
    .expect("missing delete should remain typed outcome");

    assert_eq!(delete, DeleteContactOutcome::NotFound,);

    assert_eq!(
        audit_count(&pool).await,
        0,
        "only successful mutations belong in durable audit history",
    );
}

#[sqlx::test]
async fn audit_failure_rolls_back_contact_create(pool: PgPool) {
    let (actor, _) = operator_actor(&pool, "audit-create-rollback@example.com").await;

    sqlx::query("DROP TABLE public.audit_events")
        .execute(&pool)
        .await
        .expect("audit table should be removable in isolated test DB");

    let result = create_contact(
        &pool,
        actor
            .contacts_create()
            .expect("operator should receive create grant"),
        CreateContactCommand::new(
            "Rollback Create".to_owned(),
            "rollback-create@example.com".to_owned(),
        ),
    )
    .await;

    assert!(
        matches!(result, Err(ContactError::Storage(_)),),
        "audit failure must surface as storage failure",
    );

    let contacts = Contact::find_all(&pool)
        .await
        .expect("contact lookup should succeed");

    assert!(
        contacts.is_empty(),
        "contact create must roll back when audit append fails",
    );
}

#[sqlx::test]
async fn audit_failure_rolls_back_contact_update(pool: PgPool) {
    let (actor, _) = operator_actor(&pool, "audit-update-rollback@example.com").await;

    let contact = persist_test_contact(&pool, "Original Name", "original@example.com").await;

    sqlx::query("DROP TABLE public.audit_events")
        .execute(&pool)
        .await
        .expect("audit table should be removable in isolated test DB");

    let result = update_contact(
        &pool,
        actor
            .contacts_update()
            .expect("operator should receive update grant"),
        UpdateContactCommand::new(
            contact.id.to_string(),
            "Changed Name".to_owned(),
            "changed@example.com".to_owned(),
        ),
    )
    .await;

    assert!(
        matches!(result, Err(ContactError::Storage(_)),),
        "audit failure must surface as storage failure",
    );

    let persisted = Contact::find_by_id(&pool, contact.id)
        .await
        .expect("contact lookup should succeed")
        .expect("contact must survive rollback");

    assert_eq!(persisted.name, "Original Name",);

    assert_eq!(persisted.email, "original@example.com",);
}

#[sqlx::test]
async fn audit_failure_rolls_back_contact_delete(pool: PgPool) {
    let (actor, _) = operator_actor(&pool, "audit-delete-rollback@example.com").await;

    let contact =
        persist_test_contact(&pool, "Delete Rollback", "delete-rollback@example.com").await;

    sqlx::query("DROP TABLE public.audit_events")
        .execute(&pool)
        .await
        .expect("audit table should be removable in isolated test DB");

    let result = delete_contact(
        &pool,
        actor
            .contacts_delete()
            .expect("operator should receive delete grant"),
        DeleteContactCommand::new(contact.id.to_string()),
    )
    .await;

    assert!(
        matches!(result, Err(ContactError::Storage(_)),),
        "audit failure must surface as storage failure",
    );

    assert!(
        Contact::find_by_id(&pool, contact.id,)
            .await
            .expect("contact lookup should succeed")
            .is_some(),
        "contact delete must roll back when audit append fails",
    );
}
