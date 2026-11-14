# Release notes

One file per release, named after its tag: `vX.Y.Z.md` for a final release, and for release candidates either `vX.Y.Z-rc.md` (shared by every candidate of the version, updated as work lands) or `vX.Y.Z-rc.N.md` (one candidate only, takes precedence). The release workflow publishes the file as the body of the release; it reads it from `main`, so notes can be fixed after the tag is pushed and before the workflow runs again. A release with no notes file gets notes generated from the merged pull requests.

The candidate notes file is the gate that a version is ready for testers: without it, merges into `main` cut no candidate. Plain Markdown, in English, with what testers need first:

```markdown
## What changed
- ...

## What to try
- ...

## Known issues
- ...
```

Final release notes summarize the changes since the previous final release for the people who install the Hub.

## Releasing

1. Keep the version of the next release in `Cargo.toml` (`[workspace.package]`) and its candidate notes in `docs/releases/vX.Y.Z-rc.md` on `main`.
2. After every green Tests run on `main`, the candidate workflow (`.github/workflows/candidate.yml`) tags the head `vX.Y.Z-rc.N` (next N) and publishes the pre-release. It skips, with a note in its summary, when `vX.Y.Z` is already released, when the commit already has a candidate, when `main` has moved on, or when there are no candidate notes. A candidate tag can still be pushed by hand.
3. When a candidate is accepted, add `docs/releases/vX.Y.Z.md` on `main` and run the promote workflow with the candidate tag (`gh workflow run promote.yml -f candidate=vX.Y.Z-rc.N`): it tags the candidate's commit `vX.Y.Z` and publishes it as the latest release. Promote is the owner's release gate and is never run automatically.
4. Bump the version in `Cargo.toml` for the next release; until then no new candidate is cut.
