# Port Public Docs

This directory contains the public MDX documentation site for Port.

## Local Workflow

Use the repo-supported `just` recipes from the repository root:

```bash
just docs-install
just docs-dev
just docs-build
```

The dev server binds to `0.0.0.0` by default. Override the port when `3000` is
already in use:

```bash
PORT=3010 just docs-dev
```

These commands use the repository's Nix-backed Node toolchain so the docs
workflow stays reproducible in this workspace.

## Deployment Inputs

The site reads these optional environment variables at build time:

- `DOCS_SITE_URL`
- `DOCS_BASE_URL`

If they are not set, the site defaults to `https://port.spoke.sh` and `/`.
