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

Push a version tag to trigger the automated build & release workflow:

```bash
git tag -a v0.5.0 -m "v0.5.0"
git push origin v0.5.0
```

The workflow builds on `windows-latest` and attaches installers to a GitHub Release.

### release notes

The workflow does not accept release notes as input — they are added **after** the release is created. Use either:

```bash
gh release edit v0.5.0 --notes "## What's New
- Feature one
- Bug fix two"
```

Or edit directly on the [GitHub Releases page](https://github.com/aloiva/chasm/releases).
