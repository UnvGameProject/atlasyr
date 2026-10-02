-- Durable application audit trail for successful Contacts mutations.
--
-- The table deliberately stores only stable identifiers and typed action
-- metadata. Contact names, email addresses, request payloads, query strings,
-- IP addresses, cookies, session/token material, and arbitrary metadata do
-- not belong in the durable audit record.
--
-- actor_user_id intentionally does not have a foreign key to public.users:
-- durable audit history must preserve the actor identifier even if a future
-- user-retention policy removes the corresponding account.
--
-- resource_id intentionally does not reference public.contacts because a
-- delete audit event must outlive the deleted contact row.

CREATE TABLE public.audit_events (
    id UUID PRIMARY KEY,
    actor_user_id UUID NOT NULL,
    action TEXT NOT NULL
        CHECK (action IN ('create', 'update', 'delete')),
    resource_type TEXT NOT NULL
        CHECK (resource_type = 'contact'),
    resource_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX audit_events_actor_created_at_index
    ON public.audit_events (actor_user_id, created_at DESC);

CREATE INDEX audit_events_resource_created_at_index
    ON public.audit_events (
        resource_type,
        resource_id,
        created_at DESC
    );

-- Runtime-role privilege narrowing is intentionally performed after schema
-- migration by src/db/migrate.rs. Schema migrations remain role-agnostic so
-- SQLx isolated test databases and other migration-only environments do not
-- require deployment-specific PostgreSQL roles to exist.
