# chasm — build & release

## development

```bash
npm install
npm run tauri dev
```

## production build

```bash
npm run tauri build
```

Produces three artifacts in `src-tauri/target/release/`:
- `chasm.exe` — standalone executable
- `bundle/nsis/chasm_<version>_x64-setup.exe` — NSIS installer with Start Menu entry
- `bundle/msi/chasm_<version>_x64_en-US.msi` — MSI installer for enterprise deployment

## version files

When releasing a new version, bump the version in all three files:

| File | Field |
|------|-------|
| `package.json` | `"version": "x.y.z"` |
| `src-tauri/tauri.conf.json` | `"version": "x.y.z"` |
| `src-tauri/Cargo.toml` | `version = "x.y.z"` |

## github releases

Push a version tag to trigger automated builds:

```bash
git tag -a v0.3.0 -m "v0.3.0"
git push origin master --tags
```

The GitHub Actions workflow builds on `windows-latest` and attaches installers to a GitHub Release.

### release notes

The workflow accepts an optional **release_notes** input when triggered manually via `workflow_dispatch`. To include release notes:

1. Go to **Actions → Build & Release → Run workflow**
2. Fill in the **Release notes** text field (supports markdown, use `\n` for line breaks)
3. Click **Run workflow**

If provided, a "What's New" section appears above the Downloads section in the release. If left empty, only the Downloads section is shown.
