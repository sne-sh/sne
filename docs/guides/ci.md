# Using sne in CI

In a CI job there is no shell profile, so run the setup line yourself before you call `node`.

## GitHub Actions

```yaml
steps:
  - uses: actions/checkout@v4
  - name: Install sne and the project's Node.js version
    shell: bash
    run: |
      curl -fsSL https://raw.githubusercontent.com/sne-sh/sne/main/install.sh | SNE_NO_PROFILE=1 sh
      export PATH="$HOME/.local/bin:$PATH"
      sne install
      eval "$(sne env --shell bash)"
      sne use
      dirname "$(command -v node)" >> "$GITHUB_PATH"
  - run: node -v
```

`sne install` with no version reads `.nvmrc`, `.node-version` or `package.json`. The last line of the
step makes `node` available to the steps that follow.

## One command with a specific version

`sne exec` needs no shell setup, which makes it handy in scripts:

```sh
sne install 20
sne exec 20 -- npm test
```

## Pin the sne version

Set `SNE_VERSION` so a new sne release cannot change your builds:

```sh
curl -fsSL https://raw.githubusercontent.com/sne-sh/sne/main/install.sh | SNE_VERSION=0.1.0-beta.4 sh
```

## Use a mirror

If your CI cannot reach nodejs.org, set [`SNE_NODE_MIRROR`](../reference/configuration.md) to a mirror
that has the same layout.
