# Repository and Release Strategy

**Project:** Decksmith   
**Decision:** Private Gitea incubation; GitHub becomes canonical at public developer preview.

## Naming note before public launch

**Decksmith** is the standardized internal/project brand and uses the tagline **Open Control, Forged for Linux.** Before creating the canonical public GitHub organization/repository, release packages, DNS name, or reverse-DNS application ID, perform a final trademark/name/namespace review. Unrelated software projects already use “DeckSmith” publicly, so the public namespace must be chosen deliberately even though the product direction is now standardized internally.

## Recommendation

Use the local Gitea instance as the initial private development origin while the architecture, naming, licensing, and first hardware milestone are still changing rapidly. Build and test on the Fedora desktop regardless of remote host.

When the project is ready for outside users/contributors, make GitHub the single canonical public repository and configure Gitea as a pull mirror/backup.

Do **not** run two equal writable canonical repositories.

## Why

Git repository mirroring can synchronize commits, branches, and tags, but collaboration metadata such as issues, pull requests, discussions, release workflow, stars/watchers, and community activity is platform-specific. A dual-primary model creates split-brain governance.

GitHub is the stronger canonical public home because the project is intended to be open source and will benefit from:

- discoverability
- contributor familiarity
- Issues and Pull Requests
- Discussions
- Actions
- Releases
- security policy/dependency tooling
- project boards
- package/release visibility

Gitea remains valuable for:

- local control
- private incubation
- backup/mirroring
- local Actions runner
- continued access if external hosting is unavailable

## Stages

### Stage A — Private incubation

Canonical writable remote: **local Gitea**

Use for:

- initial Rust workspace
- architecture changes
- hardware developer preview
- private issue tracking if desired
- Gitea Actions runner

Avoid creating a large amount of issue/discussion history that will need migration later.

### Stage B — Public developer preview

Canonical writable remote: **GitHub**

Actions:

1. Create public GitHub repository with final project name.
2. Push full Git history and tags.
3. Add README, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT, license, issue templates, and PR template.
4. Enable GitHub Actions for quality gates/releases.
5. Configure local Gitea as a pull mirror of GitHub.
6. Mark any old Gitea issue tracker read-only/archival if it was used.

### Stage C — Stable releases

GitHub remains canonical for collaboration.

Gitea remains:

- backup mirror
- internal/local CI target if useful
- disaster-recovery copy

## Build performance

There is no meaningful build-performance advantage to Gitea versus GitHub after cloning. Rust, GTK, tests, and RPM packaging execute on the local Fedora desktop or CI runner. The remote Git host is primarily collaboration and source-control infrastructure.

## CI portability

Put real build logic in the repository:

```text
just check
just test
just test-integration
just test-hardware
just package-rpm
```

Gitea Actions and GitHub Actions should call those commands rather than duplicating build logic in workflow YAML.

## Branching

Recommended simple model:

- `main` is always buildable.
- short-lived feature branches.
- pull request required once GitHub is public.
- release tags: `v0.x.y`, then `v1.x.y`.
- no long-lived `develop` branch unless the project grows enough to justify it.

## Protection after public launch

On GitHub:

- require passing CI before merge
- require at least one approval once there is more than one maintainer
- disallow force-push to `main`
- require linear history or squash merge policy
- enable vulnerability reporting/private security advisories

## Release channels

- Developer Preview: GitHub pre-release artifacts
- Alpha/Beta: GitHub Releases + test COPR repository
- Stable: GitHub Releases + stable COPR, later Fedora package submission

## Mirroring

Gitea supports pull and push repository mirrors. For this project, once GitHub is canonical, prefer **GitHub -> Gitea pull mirror** so GitHub remains the obvious source of truth.

Reference: https://docs.gitea.com/1.25/usage/repository/repo-mirror/
