# Direct integration into main

All 176 local branch tips from the initial audit are reachable from the integration
revision in `result.json`. No pull requests were created.

The current porting branch first advanced `main`. Of the remaining historical
branches, 120 had only patch-equivalent commits already integrated; no unique
merge commits existed outside the integration history. Their merge commits retain
the integrated tree. The remaining historical differences were reviewed against
the successor implementations and tests before retaining those newer versions.
The review tables record moved modules, revised mappings, and rejected old claims.
New EF and localization changes were merged with their source and tests.

The compressed audit files preserve the initial refs, patch-equivalence results,
worktree status, and root review counterparts. Worktrees and branches were retained.
Uncommitted experiments remain preserved for validation; they are not silently
counted as completed ports. The original checkout edits are preserved in stash
`ef8a94585452872f002fc84ab146b769b60f219b`; its parity edit changes only JSON escaping,
and its oracle note is superseded by the integrated version.

Workspace verification at `dabc355a`: 12,787 passed, 0 failed, 70 ignored
documentation examples. See the revision-pinned receipt in
`../../runtime-evidence/cargo-workspace-dabc355a.json`. This verifies the Rust
workspace at that revision; full OCCT API and legacy runtime parity remain open.

`AGENTS.md` now requires completed agent work to merge directly into `main`, with
serialized integration and relevant checks. New worktree branches start from main.
