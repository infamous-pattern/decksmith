# Repository synchronization

GitHub is the public home: https://github.com/infamous-pattern/decksmith.
Gitea retains the original development history. GitHub starts with a clean public
history to keep historical internal addresses and local filesystem paths private.
The current source tree is identical on both hosts; commit IDs differ by design.

From the maintainer checkout, run:

```sh
python3 scripts/publish-source.py
python3 scripts/publish-source.py --publish
```

The first command prepares and validates the export without pushing. The second
pushes the canonical development commit to `origin` (Gitea), then commits the same
tracked tree to the separate public history on `github` and verifies both remote
trees. It never copies development history to GitHub or uses a force push.
Only the configured `github` remote receives the clean export. Both remotes must
already be configured with normal credential helpers; no credentials are stored
in source. Build public release assets from the resulting GitHub commit.

Direct GitHub contributions must be reviewed and brought into the canonical
checkout before publication. The exporter refuses to overwrite a public tree
that differs from the last recorded export. A partial push failure is reported;
rerunning can finish the public update. New maintainer clones must configure the
remotes and retain the last exported tree marker in local Git configuration.
Release tags live in their respective histories and therefore have different
commit IDs, but should identify the same source tree.
