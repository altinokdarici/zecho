## Release Zecho

Follow these steps exactly to release a new version of Zecho.

### Step 1: Analyze Changes

Run `git log $(git describe --tags --abbrev=0)..HEAD --oneline` to see all commits since the last release.

Categorize changes into: new features, improvements, bug fixes.

Recommend a semver bump (patch/minor/major) with clear reasoning based on the changes:
- **patch**: bug fixes, small improvements, prompt tweaks
- **minor**: new user-facing features, significant UX changes
- **major**: breaking changes, major rewrites

Present the recommendation and ask the user to confirm or override. Do NOT proceed until confirmed.

### Step 2: Generate Release Notes

From the categorized commits, write user-facing release notes. Rules:
- Write in a friendly tone focused on what the user can now do or how their experience improved
- Do NOT use raw commit messages — rewrite them as benefits
- Group related commits into single bullet points
- Format:
  ```
  ## What's new in Zecho vX.Y.Z

  - **Feature name** — What the user can now do, in plain language.
  - **Improvement** — How the experience got better.
  - **Fix** — What no longer happens / what works correctly now.
  ```

Present the notes and ask the user to approve or edit. Do NOT proceed until approved.

### Step 3: Bump Version

Update the version string in both files:
- `src-tauri/Cargo.toml` — the `version = "X.Y.Z"` line
- `src-tauri/tauri.conf.json` — the `"version": "X.Y.Z"` field

### Step 4: Build & Deploy Locally

Run the full deploy cycle to validate the build:

```bash
cd src-tauri
rm -rf target/debug/build/zecho-* && touch build.rs && ~/.cargo/bin/cargo build
```

If the build fails, stop and report the error. Do NOT continue.

If the build succeeds:
```bash
pkill -f "[Zz]echo"
cp src-tauri/target/debug/zecho /Applications/Zecho.app/Contents/MacOS/zecho
codesign --force --sign "Developer ID Application: DAVID BENJAMIN ZEARING (VMGW2V57S7)" /Applications/Zecho.app
open /Applications/Zecho.app
```

### Step 5: Commit, Tag, Push

```bash
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "Bump version to X.Y.Z"
git tag vX.Y.Z
git push && git push --tags
```

### Step 6: Create GitHub Release

Create a draft GitHub release with the friendly notes and installation instructions:

```bash
gh release create vX.Y.Z --draft --title "Zecho vX.Y.Z" --notes "$(cat <<'NOTES'
## What's new in Zecho vX.Y.Z

{the approved release notes from step 2}

---

### Installation
Download the `.dmg` file, open it, and drag Zecho to Applications.
Models (~1.1GB) are downloaded automatically on first launch.

### Requirements
- macOS 12+ (Apple Silicon)
- Grant Accessibility permission for FN key recording
NOTES
)"
```

### Step 7: Report

Show a summary:
- Version released
- Release notes
- Link to the GitHub Actions run (find it via `gh run list --workflow=release.yml --limit=1`)
- Link to the draft release
