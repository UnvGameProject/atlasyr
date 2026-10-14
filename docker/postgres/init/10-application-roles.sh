#!/usr/bin/env bash
set -Eeuo pipefail

migrator_password="$(cat /run/secrets/postgres_migrator_password)"
app_password="$(cat /run/secrets/postgres_app_password)"

psql \
    --username "$POSTGRES_USER" \
    --dbname "$POSTGRES_DB" \
    --set=ON_ERROR_STOP=1 \
    --set=migrator_password="$migrator_password" \
    --set=app_password="$app_password" <<'SQL'

CREATE ROLE atlasyr_migrator
    LOGIN
    PASSWORD :'migrator_password'
    NOSUPERUSER
    NOCREATEDB
    NOCREATEROLE
    NOREPLICATION;

CREATE ROLE atlasyr_app
    LOGIN
    PASSWORD :'app_password'
    NOSUPERUSER
    NOCREATEDB
    NOCREATEROLE
    NOREPLICATION;

ALTER DATABASE atlasyr OWNER TO atlasyr_migrator;
ALTER SCHEMA public OWNER TO atlasyr_migrator;

GRANT CONNECT ON DATABASE atlasyr TO atlasyr_app;
GRANT USAGE ON SCHEMA public TO atlasyr_app;

ALTER DEFAULT PRIVILEGES
    FOR ROLE atlasyr_migrator
    IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO atlasyr_app;

ALTER DEFAULT PRIVILEGES
    FOR ROLE atlasyr_migrator
    IN SCHEMA public
    GRANT USAGE, SELECT ON SEQUENCES TO atlasyr_app;

SQL
