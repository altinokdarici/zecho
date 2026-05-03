# Release Skill Design

## Overview

A `/release` Claude Code skill that automates the full Zecho release process: analyzing changes, generating friendly release notes, bumping versions, validating the build locally, and publishing to GitHub.

## Skill Flow

### 1. Analyze Changes

- Gather commits since the last version tag (`git log <last-tag>..HEAD --oneline`)
- Categorize changes: new features, improvements, bug fixes
- Recommend a semver bump (patch/minor/major) with reasoning based on the changes
- Present recommendation and ask user to confirm or override

### 2. Generate Release Notes

- From the categorized commits, write user-facing release notes in a friendly tone
- Focus on what the user can now do or how their experience improved — not commit messages
- Format:
  ```
  ## What's new in Zecho vX.Y.Z

  - **Feature name** — What the user can now do, in plain language.
  - **Fix description** — How the experience improved.
  ```
- Present notes for user approval/editing before proceeding

### 3. Bump Version

- Update version in `src-tauri/Cargo.toml`
- Update version in `src-tauri/tauri.conf.json`

### 4. Build & Deploy Locally

Run the standard deploy cycle to validate the build compiles:

1. `rm -rf target/debug/build/zecho-* && touch build.rs && cargo build`
2. `pkill -f "[Zz]echo"`
3. `cp src-tauri/target/debug/zecho /Applications/Zecho.app/Contents/MacOS/zecho`
4. `codesign --force --sign "Developer ID Application: DAVID BENJAMIN ZEARING (VMGW2V57S7)" /Applications/Zecho.app`
5. `open /Applications/Zecho.app`

If the build fails, stop and report the error.

### 5. Commit, Tag, Push

- Commit the version bump: `Bump version to X.Y.Z`
- Create annotated tag: `git tag vX.Y.Z`
- Push commit and tag: `git push && git push --tags`

### 6. Create GitHub Release

- Use `gh release create vX.Y.Z --draft` with the release body containing:
  - Generated friendly release notes (top)
  - Installation instructions (bottom, standard boilerplate)
- Created as a draft so it's not publicly visible until CI uploads the signed `.dmg`
- The CI workflow will upload artifacts and publish the release

### 7. Report

- Show summary of what was released
- Link to the GitHub Actions run
- Link to the draft release

## CI Workflow Changes

The existing `.github/workflows/release.yml` needs these changes:

### Trigger

Replace `workflow_dispatch` with tag push:

```yaml
on:
  push:
    tags:
      - 'v*'
```

### Remove `set-version` Job

The version is already correct in the committed code. Delete the entire `set-version` job.

### Update `build-macos` Job

- Remove `needs: set-version`
- Remove the artifact download step
- Extract version from the tag ref: `echo "VERSION=${GITHUB_REF#refs/tags/v}" >> $GITHUB_ENV`
- Configure tauri-action to upload to the existing draft release instead of creating a new one:
  - Remove `tagName`, `releaseName`, `releaseBody`, `releaseDraft`, `prerelease`
  - Add `releaseId` pointing to the draft release, or use `tagName` with the existing tag

### Update `update-homepage` Job

- Extract version from the tag ref instead of `inputs.version`
- Update all `${{ inputs.version }}` references to use the extracted version

## Skill File

Location: `.claude/commands/release.md` in the project repo.

The skill file will contain the instructions Claude follows when `/release` is invoked. It will reference this design for the full flow.

## Release Body Template

```markdown
## What's new in Zecho vX.Y.Z

{generated changelog}

---

### Installation
Download the `.dmg` file, open it, and drag Zecho to Applications.
Models (~1.1GB) are downloaded automatically on first launch.

### Requirements
- macOS 12+ (Apple Silicon)
- Grant Accessibility permission for FN key recording
```
