import { readFileSync } from "node:fs";

const lockPath = process.argv[2];

if (!lockPath) {
    console.error(
        "usage: node check-package-lock.mjs <package-lock.json>",
    );
    process.exit(2);
}

const lock = JSON.parse(
    readFileSync(lockPath, "utf8"),
);

if (lock.lockfileVersion !== 3) {
    console.error(
        `unsupported package-lock version: ${lock.lockfileVersion}`,
    );
    process.exit(1);
}

const packages = lock.packages;

if (
    packages === null
    || typeof packages !== "object"
    || Array.isArray(packages)
) {
    console.error(
        "package-lock.json does not contain a packages object",
    );
    process.exit(1);
}

const failures = [];
let auditedPackages = 0;

for (const [location, dependency] of Object.entries(packages)) {
    if (location === "") {
        continue;
    }

    auditedPackages += 1;

    if (
        dependency === null
        || typeof dependency !== "object"
        || Array.isArray(dependency)
    ) {
        failures.push(
            `${location}: dependency entry is not an object`,
        );
        continue;
    }

    if (dependency.link === true) {
        failures.push(
            `${location}: linked dependency is not allowed`,
        );
        continue;
    }

    if (
        typeof dependency.resolved !== "string"
        || dependency.resolved.length === 0
    ) {
        failures.push(
            `${location}: missing resolved package source`,
        );
        continue;
    }

    let resolved;

    try {
        resolved = new URL(dependency.resolved);
    } catch {
        failures.push(
            `${location}: invalid resolved package URL`,
        );
        continue;
    }

    if (resolved.protocol !== "https:") {
        failures.push(
            `${location}: package source must use HTTPS`,
        );
    }

    if (resolved.hostname !== "registry.npmjs.org") {
        failures.push(
            `${location}: untrusted package source host `
            + `${resolved.hostname || "<none>"}`,
        );
    }

    if (
        typeof dependency.integrity !== "string"
        || dependency.integrity.length === 0
    ) {
        failures.push(
            `${location}: missing package integrity metadata`,
        );
    }
}

if (failures.length !== 0) {
    console.error(
        "frontend lockfile source policy failed:",
    );

    for (const failure of failures) {
        console.error(`  ${failure}`);
    }

    process.exit(1);
}

console.log(
    `frontend lockfile source policy ok: `
    + `${auditedPackages} packages`,
);
