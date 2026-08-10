/// Every repository in the Workinabox org. The monitor shows all of them.
pub const ORG_REPOS: [&str; 10] = [
    ".github",
    "backend",
    "frontend",
    "app",
    "website",
    "sw-dev-team",
    "iac",
    "dev",
    "docs",
    "assets",
];

/// The repos that take part in a synchronized `dev release` — tagged together
/// with one version. Deliberately a subset of [`ORG_REPOS`]: `sw-dev-team`
/// versions independently (its own semver + Python wheel scheme), and `docs` /
/// `iac` do not cut synchronized release tags. Revisit if that policy changes.
pub const RELEASE_REPOS: [&str; 7] = [
    ".github", "dev", "backend", "frontend", "website", "app", "assets",
];
