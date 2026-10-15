#!/usr/bin/env bash
set -Eeuo pipefail

# Non-release compatibility experiment.
# Never mutate the qualified Atlasyr dependency or lockfile.

repo="$(git rev-parse --show-toplevel)"

[ "$(pwd -P)" = "$repo" ] || {
    echo 'FAIL: Run from the Atlasyr repository root'
    exit 1
}

command -v git >/dev/null
command -v cargo >/dev/null
command -v rustup >/dev/null
command -v node >/dev/null
command -v npm >/dev/null
command -v python3 >/dev/null

if ! rustup run stable rustc --version >/dev/null 2>&1; then
    echo 'FAIL: Rust stable toolchain is not installed'
    echo 'Install it separately before running this canary'
    exit 1
fi

scratch="$(mktemp -d "${TMPDIR:-/tmp}/atlasyr-topcoat-canary.XXXXXXXX")"
stage='initialization'
upstream_sha='not-resolved'

finish() {
    result=$?

    printf '\n============================================================\n'
    printf ' TOPCOAT MAIN CANARY RESULT\n'
    printf '============================================================\n'
    printf ' Upstream commit: %s\n' "$upstream_sha"
    printf ' Last stage: %s\n' "$stage"
    printf ' Exit status: %s\n' "$result"

    if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
        {
            printf '## Atlasyr / Topcoat `main` canary\n\n'
            printf -- '- Upstream commit: `%s`\n' "$upstream_sha"
            printf -- '- Last stage: `%s`\n' "$stage"
            printf -- '- Exit status: `%s`\n' "$result"
            printf -- '- Atlasyr stable dependency: unchanged\n'
            printf -- '- Release qualification: not performed\n'
        } >> "$GITHUB_STEP_SUMMARY"
    fi

    rm -rf -- "$scratch"

    if [ "$result" -ne 0 ]; then
        echo 'FAIL: Upstream compatibility experiment did not pass'
    else
        echo 'PASS: Upstream compatibility experiment completed'
    fi
}

trap finish EXIT

upstream="$scratch/topcoat"
application="$scratch/atlasyr"
tools="$scratch/tools"

mkdir -p "$application" "$tools"

export RUSTUP_TOOLCHAIN=stable
export SQLX_OFFLINE=true
export CARGO_TARGET_DIR="$scratch/cargo-target"
export CARGO_INCREMENTAL=0

printf '\n=== 1. RESOLVE TOPCOAT MAIN ===\n'

stage='upstream checkout'

git clone \
    --quiet \
    --depth 1 \
    --filter=blob:none \
    --single-branch \
    --branch main \
    https://github.com/tokio-rs/topcoat.git \
    "$upstream"

upstream_sha="$(git -C "$upstream" rev-parse HEAD)"

[[ "$upstream_sha" =~ ^[0-9a-f]{40}$ ]] || {
    echo 'FAIL: Unexpected upstream commit'
    exit 1
}

printf 'Topcoat commit: %s\n' "$upstream_sha"

python3 - "$upstream/Cargo.toml" <<'PY'
import sys
import tomllib
from pathlib import Path

with Path(sys.argv[1]).open("rb") as file:
    data = tomllib.load(file)

workspace = data["workspace"]["package"]

print("Topcoat workspace version:", workspace["version"])
print("Topcoat declared MSRV:", workspace["rust-version"])
PY

rustc --version
cargo --version
node --version
npm --version

printf '\n=== 2. CREATE AN ISOLATED ATLASYR SOURCE COPY ===\n'

stage='isolated Atlasyr source preparation'

git archive --format=tar HEAD |
    tar -xf - -C "$application"

python3 - "$application/Cargo.toml" "$upstream_sha" <<'PY'
import sys
import tomllib
from pathlib import Path

manifest_path = Path(sys.argv[1])
revision = sys.argv[2]
content = manifest_path.read_text()

with manifest_path.open("rb") as file:
    manifest = tomllib.load(file)

topcoat = manifest["dependencies"]["topcoat"]

if topcoat.get("version") != "=0.10.0":
    raise SystemExit("FAIL: Unexpected qualified Topcoat pin")

if sorted(topcoat.get("features", [])) != ["htmx", "tower"]:
    raise SystemExit("FAIL: Unexpected Topcoat feature selection")

original = (
    'topcoat = { version = "=0.10.0", '
    'features = ["tower", "htmx"] }'
)

replacement = (
    'topcoat = { git = "https://github.com/tokio-rs/topcoat", '
    f'rev = "{revision}", features = ["tower", "htmx"] }}'
)

if content.count(original) != 1:
    raise SystemExit("FAIL: Atlasyr manifest structure changed")

manifest_path.write_text(content.replace(original, replacement, 1))

print("PASS: Canary-only Git dependency configured")
print("PASS: Qualified repository files remain untouched")
PY

printf '\n=== 3. BUILD FRONTEND PREREQUISITES ===\n'

stage='frontend prerequisites'

(
    cd "$application"

    npm ci \
        --ignore-scripts \
        --no-audit \
        --no-fund

    mkdir -p generated
    npm run build:assets
)

printf '\n=== 4. CHECK ATLASYR AGAINST TOPCOAT MAIN ===\n'

stage='application API compilation'

(
    cd "$application"

    cargo check --all-targets
)

printf '\n=== 5. COMPILE AND RUN FOCUSED PROXY TESTS ===\n'

stage='focused regression tests'

(
    cd "$application"

    cargo test --lib config::proxy::tests::
)

printf '\n=== 6. BUILD ATLASYR CANARY BINARY ===\n'

stage='application binary build'

(
    cd "$application"

    cargo build --bin atlasyr
)

printf '\n=== 7. BUILD MATCHING UPSTREAM CLI ===\n'

stage='Topcoat CLI build'

cargo install \
    --path "$upstream/crates/topcoat-cli" \
    --locked \
    --root "$tools"

[ -x "$tools/bin/topcoat" ] || {
    echo 'FAIL: Matching Topcoat CLI was not produced'
    exit 1
}

printf '\n=== 8. EXERCISE MATCHED CLI / ASSET BUNDLER ===\n'

stage='matched CLI asset bundle'

(
    cd "$application"

    "$tools/bin/topcoat" asset bundle --bin atlasyr
)

printf '\n=== 9. VERIFY STABLE ATLASYR REPOSITORY ===\n'

stage='repository integrity'

[ -z "$(git status --porcelain)" ] || {
    echo 'FAIL: Canary modified the real repository'
    git status --short
    exit 1
}

printf 'PASS: Original repository remains clean\n'
printf 'PASS: Topcoat library and CLI tested from one upstream commit\n'
printf 'PASS: No stable dependency, lockfile, or release changes\n'
