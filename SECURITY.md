# Security policy

sne downloads and installs programs, and its installers change your `PATH` and shell profile, so
security reports are taken seriously.

## Supported versions

Only the latest release receives security fixes. Update with `sne update` before reporting.

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Report it privately through GitHub:

1. Go to the [Security tab](https://github.com/sne-sh/sne/security) of the repository.
2. Click **Report a vulnerability**.
3. Describe the problem, how to reproduce it, and the version from `sne --version`.

You will get a reply within a few days. Please give us reasonable time to release a fix before you
publish details. We credit reporters in the release notes unless you ask us not to.

## What counts

Examples of what we want to hear about:

- A way to make sne install or run something other than the file whose SHA-256 it verified.
- A way for an archive to write files outside the folder it is being installed into.
- A problem in `install.ps1`, `install.sh`, the Windows setup wizard or `sne update` that lets someone
  else's code run.
- A release file whose contents do not match `SHA256SUMS.txt`.

## What sne verifies

- Every Node.js download is checked against the SHA-256 in the `SHASUMS256.txt` of the same server.
  This protects against a damaged or incomplete download. It does not protect against a mirror that
  serves a matching pair of altered files, so only set `SNE_NODE_MIRROR` to a server you trust.
- `sne update` and the install scripts check the release file against the release's `SHA256SUMS.txt`.
- The Windows installers and the macOS package are not code-signed yet.
- Downloads are made over HTTPS only. Plain `http://` is refused, except for `127.0.0.1` and
  `localhost`.
- Project version files (`.nvmrc`, `.node-version`, `package.json`) are read only when they are
  small regular files, never through a link, and a version containing control characters is ignored.

## Check a download

Every release file is listed in the release's `SHA256SUMS.txt`:

```sh
sha256sum -c SHA256SUMS.txt --ignore-missing
```

From 0.1.0-beta.5 on, each file also has a build provenance record. It proves that the file was built
by the release workflow of this repository from the tagged commit. With the
[GitHub CLI](https://cli.github.com):

```sh
gh attestation verify sne-0.1.0-beta.5-x64-setup.exe --repo sne-sh/sne
```
