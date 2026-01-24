---
description: Release workflow for MindFry - version bump, test, publish, git tag
---

# Release Workflow

// turbo-all

## Pre-release Checks

1. Run clippy and fix any lints:

```bash
cargo clippy
```

2. Run full test suite:

```bash
cargo test
```

3. Build release binary:

```bash
cargo build --release
```

## Impact Analysis

4. **SDK Impact Check** - Review changes for SDK compatibility:
   - [ ] New protocol opcodes → Update SDK encoder/decoder
   - [ ] New error codes → Update SDK error handling
   - [ ] Changed message formats → Update SDK message types
   - [ ] New public API methods → Add SDK bindings
   - If any checked, create issue in SDK repo before release

5. **Documentation Impact Check** - Review changes for docs updates:
   - [ ] New features → Add to README.md and docs/
   - [ ] Changed behavior → Update relevant docs
   - [ ] New CLI commands → Update CLI docs
   - [ ] Breaking changes → Add migration guide
   - [ ] New config options → Document in config reference

6. **CHANGELOG Update** - Add entry to CHANGELOG.md:
   - [ ] New features under "Added"
   - [ ] Bug fixes under "Fixed"
   - [ ] Breaking changes under "Changed" or "Removed"
   - [ ] Security fixes under "Security"
   - Format: `## [vX.Y.Z] - YYYY-MM-DD`

7. **README Check** - Update README.md if needed:
   - [ ] Feature highlights in intro
   - [ ] Installation instructions still accurate
   - [ ] Usage examples reflect new features
   - [ ] Badge versions updated

## Version Bump

8. Update version in `Cargo.toml`:
   - Patch: x.y.Z (bug fixes)
   - Minor: x.Y.0 (new features, backward compatible)
   - Major: X.0.0 (breaking changes)

## Publish

9. Commit all changes:

```bash
git add -A
git commit -m "feat: <description> (vX.Y.Z)"
```

10. Create git tag:

```bash
git tag vX.Y.Z
```

11. Push to remote with tags:

```bash
git push origin main --tags
```

12. Publish to crates.io:

```bash
cargo publish
```

13. Wait for CI to pass.

## Post-release

14. Generate release notes with highlights
15. Update SDK if impact detected (step 4)
16. Update docs if impact detected (step 5)
17. Create GitHub Release from tag (optional)
18. Announce on blog/socials
