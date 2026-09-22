# Quality gate and CI readiness

The shared gate is scripts/check.sh; just check delegates to it. It runs formatting,
Clippy, all unit/integration tests and cargo-deny checks for advisories, license
allowlist, duplicate versions, wildcard requirements and unapproved sources.
RustSec data is refreshed on online runs; offline success is not current verification.

On this workstation cargo-deny 0.20.2 is installed in the task-local work/tools/bin
folder. Set DECKSMITH_CARGO_DENY to that executable if it is not on PATH. No global
shell or desktop settings were changed. just is optional and is not installed here.

The Gitea runner list showed zero available runners on 2026-09-10. The workflow
.gitea/workflows/quality.yaml is therefore manual-only and has NOT been executed
remotely. It expects a repository-scoped runner with label decksmith-linux, Git,
Bash, Node 20 for checkout, Rust stable/rustfmt/Clippy and cargo-deny 0.20.2.
It needs network access to Gitea, crates.io, GitHub checkout action and RustSec.
The full gate now also requires a C compiler, pkg-config and libudev development
headers (Fedora systemd-devel). It requires no attached hardware. Checkout credentials are not persisted.

Before automatic CI: choose an isolated runner host, authorize registration,
provision those prerequisites, run the manual workflow and verify success. Then add
push/pull_request triggers. No runner registration token has been created, and no
background runner or container service has been installed on the desktop.

The checkout action is pinned to the commit resolved from official actions/checkout
v4 on 2026-09-10. Build logic is host-independent and reusable for future GitHub CI.

References:
- https://docs.gitea.com/usage/actions/quickstart/
- https://embarkstudios.github.io/cargo-deny/
