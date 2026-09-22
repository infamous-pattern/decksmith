# Preview security review — 2026-09-22

## Scope and result

Reviewed the published `v0.1.0-preview.1` source and its downloaded Fedora x86_64
runtime assets, then tested hardening changes for `v0.1.0-preview.2`. This is an
engineering security review, not an independent penetration-test certification
or a guarantee that vulnerabilities do not exist.

The preview.1 archive identified source commit
`84ec46e3d06a6f83b97e39b47257133a922a3e46`, release `0.1.0-00e3161ac5d6`.
All four downloaded asset checksums and all 329 package entries matched the
manifest. The preview.2 release manifest records its own exact source commit.

## Findings addressed in preview.2

| Area | Finding and exposure | Correction and regression coverage |
| --- | --- | --- |
| Installer | A partial checksum list could pass the shell checker without covering executable downloads. This is an integrity-validation gap, not protection against a compromised publisher. | Require exactly the four expected files, reject duplicates, omissions, unexpected paths, malformed/oversized manifests and mismatches; run verification with isolated Python. HTTPS-only redirects. |
| SVG imports | A byte-pattern DTD check could be bypassed with UTF-16 XML. This bypassed the intended no-entity policy; arbitrary code execution was not demonstrated. | Parser-level DTD rejection before expansion; tests cover UTF-8 and both UTF-16 byte orders. Existing shape/attribute restrictions remain. |
| Website icons | Initial URL checks did not explicitly constrain every redirect. | Restrict every redirect to HTTP(S), reject embedded credentials, and retain byte and image-dimension limits. Tests reject file, FTP, data and credential-bearing redirects. |
| Optional Homebridge panel | An unauthenticated loopback page disclosed its command token; state was also readable without that token. Another local user could discover the port and access this interface. | The browser entry page requires the token, state requires an authentication header, managed mode does not print an access URL, and browser history/referrers are restricted. A real loopback HTTP test verifies rejected anonymous/wrong-origin/wrong-host requests and successful authenticated requests without touching Homebridge or a device. |

The main installer does **not** update a separately installed Homebridge companion.
Its panel fix must be included when that companion is rebuilt/reinstalled. The
main public runtime does not include the external OpenHomeB executable or vendor
wheel; those remain separately managed experimental components.

## Checks performed

- Cargo-deny 0.20.2 with refreshed advisory data: main locked dependency graph
  passed advisories, licenses, bans and source-policy checks.
- Retained optional companion Rust source: advisory scan passed. This does not
  establish reproducible provenance for every separately installed companion binary.
- pip-audit: pinned companion `websockets==15.0.1` had no reported vulnerabilities.
- Bandit 1.9.4 scanned Python runtime, editor and installer code. No high-severity
  findings were reported. The two medium findings concerned SVG/XML and URL
  handling and were manually reviewed and hardened. Generic warnings about
  subprocess use, exception handling and test constants were reviewed in context;
  scanner silence is not the acceptance criterion.
- Command launches use argument lists rather than shell interpolation. Reviewed
  bounded archive extraction, traversal/link rejection, manifest coverage, runtime
  file ownership/permissions, Secret Service password handling and local HTTP gates.
- Published daemon and CLI binaries: PIE, GNU RELRO, immediate binding, and a
  non-executable GNU_STACK segment verified with readelf.
- 142 editor tests and 15 installer/companion tests passed locally after the fixes.
  Tests include malformed/tampered archives, rollback, symlink protection,
  incomplete checksum manifests, encoded SVG DTDs and HTTP authorization.
  CI skips only the optional packaged-companion import test when its separate
  build fixture is absent. Dependency auditing is now a CI gate.
- A focused credential-pattern scan and internal-metadata review preceded public
  publication. Public history was initialized separately from private development
  history. This is not proof that all possible secret formats were detected.

## Remaining boundaries and follow-up

- Release checksums detect corruption; they do not authenticate a compromised
  GitHub account or release. Signed provenance/release attestation is still future
  work. Inspect downloaded code and use the official repository.
- GTK, Pillow, librsvg, Python, systemd and audio libraries come from Fedora and
  must be kept updated through Fedora. Cargo/pip audits do not certify OS packages.
- Decksmith and optional plugins run as the desktop user. Process supervision is
  not an OS sandbox, and same-user malware is outside this protection boundary.
- Homebridge HTTP connections are unencrypted. Use HTTPS when available, or a
  trusted LAN; passwords and tokens sent over HTTP are not protected in transit.
- Website artwork fetching may contact the selected site's icon hosts. Local/LAN
  URLs remain supported intentionally; the feature is not a network isolation boundary.
- No exhaustive fuzzing, malicious-USB analysis, hardware firmware audit, or
  independent penetration test was performed. More devices, distributions and
  shared-user environments need validation before V1.
