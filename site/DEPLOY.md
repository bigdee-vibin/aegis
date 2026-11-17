# Deploying the site to GitHub Pages

Nothing here has been run. This is a draft for the repo owner to review.

## Why a workflow

Settings > Pages > "Deploy from a branch" only lets you pick `/` (root) or `/docs` as the folder. `/site` is not selectable. Two options:

1. **GitHub Actions (recommended):** Settings > Pages > Build and deployment > Source: **GitHub Actions**, then add the workflow below.
2. Move or copy the contents of `site/` into `/docs` on the branch you publish, then choose "Deploy from a branch" with folder `/docs`.

## Draft workflow

Save as `.github/workflows/pages.yml` on the branch Pages should deploy from. It is shown here only; it is deliberately not placed in `.github/` by this change. It uses only first-party GitHub actions, pinned by tag; consider pinning by commit SHA after review.

```yaml
name: Deploy site to Pages

on:
  push:
    branches: [main]
    paths: ["site/**"]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: false

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/configure-pages@v5
      - uses: actions/upload-pages-artifact@v3
        with:
          path: site
      - id: deployment
        uses: actions/deploy-pages@v4
```

Adjust `branches` to wherever `site/` is merged (currently it exists only on the `demo-site` branch). Pages environment protection rules may restrict which branches can deploy.

## Before publishing

- Review the status panel in `site/index.html` against the current repo state.
- Confirm you are happy to publish the design board and links to architecture documents (the repo is already public).
- The result URL will be https://bigdee-vibin.github.io/aegis/.
