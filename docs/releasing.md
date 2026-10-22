# Releasing EffectCraft

> **This fork (685ks/effectcraft).** Releases are made by
> [`.github/workflows/release.yml`](../.github/workflows/release.yml), which calls the shared
> Arthavan Projects workflow (`685ks/craft-actions`, `release-rust-app.yml`) with the ids in
> [`packaging/app.toml`](../packaging/app.toml) (bundle id `app.freedev.ainz.ks.EffectCraft`,
> publisher Arthavan Projects). Run it from **Actions › Release › Run workflow** with the version; it
> creates the GitHub Release `v<version>` (a draft unless `publish` is set), which the Arthavan
> Projects launcher installs and updates from. A copy started by the launcher
> (`ARTHAVAN_LAUNCHER=1`) says "Updates are managed by Arthavan Projects" in its About dialog. The
> upstream process below is kept for reference; its `release` branch trigger is not used here. This
> fork ships for Windows (and the web) only: the macOS, Linux and FreeBSD packaging was removed.

Upstream, every push to the `release` branch runs [`.github/workflows/release.yml`](../.github/workflows/release.yml).
It builds signed installers for Windows and the web
build, then creates or updates a
**draft** GitHub Release named `EffectCraft v<version>`. Nobody sees a draft until a maintainer
publishes it.

User-facing names say **EffectCraft**. Files, binaries and ids stay lowercase
(`effectcraft-<version>-<platform>-<arch>.<ext>`).

## Cutting a release

1. **Bump the version** on `main`. It lives in one place, `[workspace.package] version` in the
   root `Cargo.toml`:

   ```sh
   cargo xtask version                 # prints the current version, e.g. 0.3.1
   cargo xtask version set 0.4.0       # or 0.4.0-rc.1; updates Cargo.toml and Cargo.lock
   ```

   Open a PR titled `Release: EffectCraft v0.4.0` with that change (`Cargo.toml` and our crates
   in `Cargo.lock` only) and merge it.
2. **Check the gate.** Run `cargo xtask ci` on `main` (it runs clippy and the tests in release
   mode, as CI does), and check that `main`'s CI is green on every platform.
3. **Push `main` to `release`** (a fast-forward): `git push origin main:release`. The `release`
   branch is protected; only maintainers can push to it. The workflow starts by itself.
4. **Wait for the draft.** When every job is green, the
   Releases page has a draft `EffectCraft v0.4.0`, tagged `v0.4.0` on the pushed commit, with
   every artifact and `SHA256SUMS.txt`. Its notes are generated from the merged PRs.
5. **Check it.** Download an installer or two, check them against `SHA256SUMS.txt`, and run
   `effectcraft --version` / `effectcraft-cli --version`. Read the job summaries: a `::warning::`
   means a signing secret was missing and that artifact is unsigned.
6. **Publish** the draft in the GitHub UI (or `gh release edit v0.4.0 --draft=false --latest`),
   with a short *Highlights* section above the generated notes. Publishing creates the `v0.4.0`
   tag. Versions with a pre-release suffix (`-rc.1`) are marked as pre-releases.

Pushing to `release` again before you publish rebuilds the same draft and replaces its assets.
Once the draft is published, the workflow refuses to touch that version again
(`Release v<version> is already published. Bump the version (cargo xtask version set) before
pushing to release.`), so bump it first.

**Test runs:** *Actions › Release › Run workflow* runs the whole pipeline by hand. The optional
`version` input (such as `0.4.0-rc.1`) overrides `Cargo.toml` for that run only: the jobs apply it
with `cargo xtask version set` before building, so the binaries report it too. Signing and the
draft release need the `release` environment, which only the `release` branch can use. Run on
any other branch (`gh workflow run release.yml --ref <branch>`), it is a dry run: the jobs that
sign nothing (web) build and check everything, the Windows
jobs are refused by the environment's branch rule, and no draft release is made.

## What gets built

| Platform | Artifacts | Built on |
|---|---|---|
| Windows 10+ x64 | `effectcraft-<v>-windows-x64.msi`, `effectcraft-<v>-windows-x64-portable.zip` | `windows-latest` |
| Windows 10+ x86 (32-bit) | `effectcraft-<v>-windows-x86.msi`, `effectcraft-<v>-windows-x86-portable.zip` | `windows-latest` |
| Windows 11 ARM64 | `effectcraft-<v>-windows-arm64.msi`, `effectcraft-<v>-windows-arm64-portable.zip` | cross-compiled on `windows-latest` |
| Web | `effectcraft-web-<v>.zip` (a static site; see [web.md](web.md)) | `ubuntu-latest` |

The ARM64 MSI is installed and run on ARM64 hardware by
[`windows-arm64.yml`](../.github/workflows/windows-arm64.yml).

The MSI's setup wizard ([`installer-ui.wxs`](../packaging/windows/installer-ui.wxs)) lets people
choose the install folder; upgrades and repairs reuse it, and `INSTALLFOLDER="..."` picks it for a
silent (`/qn`) install. [`packaging-lint.yml`](../.github/workflows/packaging-lint.yml) install-tests
that with stub binaries ([`test-msi.ps1`](../packaging/windows/test-msi.ps1)): a custom folder,
an upgrade that keeps it, a repair, uninstall, and the Program Files default. The wizard pages
themselves are checked by hand.

The portable zip ships [`portable.txt`](../packaging/windows/portable.txt) beside `effectcraft.exe`.
That marker (or an `EffectCraft.portable` file) switches on portable mode: the settings folder
(preferences, shortcuts, recovery files, logs, plug-ins) becomes `EffectCraftData\` next to the
exe instead of `%APPDATA%\EffectCraft`. If that folder isn't writable the app logs a warning and
uses `%APPDATA%`. The MSI has no marker. The logic is in `crates/host/src/app_dirs.rs` and works
the same on macOS and Linux; `EFFECTCRAFT_CONFIG_DIR` still wins over it.

Every binary reports its version: `effectcraft --version`, `effectcraft-cli --version` and
*Help › About EffectCraft*.

## Signing

The Windows jobs and the draft-release job run in the `release` environment, which only
the `release` branch can use and which holds the signing secrets. The jobs that sign nothing run
without it and get no secrets. Every secret is optional: a missing one produces an unsigned artifact
and a warning in the job summary, never a failed build.

| Platform | Secrets |
|---|---|
| Windows (a certificate, or Azure Trusted Signing) | `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`, or `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE` |
