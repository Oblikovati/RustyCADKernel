# Martensite release failure evidence

`python3 tools/release_packages.py --tag v0.2.3 --root ../kernel` runs the first extracted Cargo test
without a graphics probe, warming pass or retry. Each extracted command records
its actual arguments, working directory, stdout, stderr and exit status in a
unique directory under `target/release-diagnostics/`. Stream files have a 16 MiB
byte limit and explicitly report their original byte count and truncation.

After a failed command, before temporary-stage deletion, `failure-inputs.json`
records SHA-256 and size for all extracted source, shader, fixture and manifest
inputs outside the root build directory. It hashes Cargo's explicitly reported
failed executable; for ordinary test failures Cargo's last `Running ... (path)`
identifies that executable. Missing executable identification remains explicit.
The receipt also records CPU/platform, a small explicit environment whitelist,
selected ICD manifest/library identities where directly resolvable, and `ldd`
resolved ELF library hashes. These are post-exit dependency metadata, not proof
of runtime mappings or the selected physical GPU. No graphics device is opened
by the collector. Missing inspection tools remain diagnostics.

The original exception and exit status remain red even if diagnostic writing
fails. The stage is deleted by its existing temporary-directory scope after
receipt collection; failed assets are never sealed. Upload only
`target/release-diagnostics/**`: this contains bounded logs and JSON receipts,
never stage directories, executables, caches or complete archives. Environment
variables outside the explicit whitelist, including credentials, are excluded.
Command output is recorded as emitted by the release tests.

Run regressions with:

```sh
PYTHONPATH=tools:tools/tests python3 -m unittest test_release_failure_evidence test_release_packages test_ci_oracle_boundary
```

`--root` selects the immutable kernel checkout; its default is the repository
owning the tools. Updated orchestration can therefore test an older tagged
kernel without modifying that tag or requiring the logger in its source. Git
clean/tag checks, Cargo commands, packaged fixtures and diagnostics all use the
selected kernel root.

Named cold-command, missing-library-tool and diagnostic-volume fakes prove the
original failure, one cold attempt, cleanup, unsealed output, persistent exact
input/executable hashes, successful stderr recording and metadata exclusions.
