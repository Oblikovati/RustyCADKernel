# Release reference storage maintenance

Source `255f626f905d62bfdd6688e21e6e2b0c6c8dfc7d`, based on
`9bf9fbf1e0218cb2e1774ae815db21f574922eca`. This fixes the automatic maintenance
error observed while integrating Kernel #10. It does not authorize or create a
release, change package acceptance, or repair existing release provenance.

## Reproduction and diagnosis

An isolated temporary repository contains one commit and one existing annotated
`v0.2.2` fixture tag, created before installing the actual owner hook. With no
release receipt, two `git pack-refs --all` invocations and `git gc --quiet` each
returned 128. The hook reported a missing `MARTENSITE_RELEASE_RECEIPT`.

The prepared transaction supplied zero as the old OID and the existing annotated
tag object as the new OID. At that same checkpoint, a full-ref lookup returned
exactly the new object. The peeled commit was different. Hook cwd was the correct
temporary repository. This confirms storage insertion was mistaken for logical
tag creation; wrong checkout and commit-vs-tag comparison hypotheses were ruled
out by the captured operands.

[Git's primary hook documentation](https://git-scm.com/docs/githooks#_reference_transaction)
explains that zero-old does not establish absence and recommends inspecting the
current ref. The retained traces are from Git 2.43.0. The five-test initial
regression failed only the packing and GC cases; new-tag and changed-tag guards
already passed. All initial output is retained in `runs.tar.gz`.

## Bounded decision: maintenance admission V1

For a prepared release-ref update, resolve its **full unpeeled existing object**.
An unchanged direct ref is a no-op only when the actual object equals the new
OID and the supplied old OID is either zero or that same object. Symbolic refs
are rejected: replacing an alias with a direct ref can change later behavior
even when both currently resolve to the same object. Malformed OIDs, warning/error
reads and unexpected read failures do not become no-op evidence.

A different existing object remains an immutable-tag violation, including a new
annotation pointing at the same commit. An absent ref still follows the original
receipt, annotated-object, source, clean-checkout, command-roster and asset checks.
Receipt format 1 and all 15 existing verification functions are unchanged by AST
comparison. The hook shell script is byte-identical to the baseline. Ordinary
refs, non-prepared callbacks and the existing deletion policy are unchanged.
No broad structural replacement or version change of the release receipt is made;
this is an additive tooling bug fix at the transaction-admission boundary.

## Frozen checks

All checks below ran on the committed source above with `PYTHONPATH=tools` and
identical before/after hashes for all **3,739 tracked tool/hook files**:

| Check | Result |
| --- | --- |
| `python3 -m unittest discover -s tools/tests -p 'test_release_*.py' -v` | 94 passed |
| `python3 tools/run_tool_tests.py --jobs 2 --pattern 'test_pre_release*.py'` | 4 modules passed |
| `python3 -m unittest discover -s tools/tests -p test_verify_release_gate.py -v` | 8 passed |
| Black 25.1.0 check of the three changed Python files | Passed |
| Original isolated repository: pack twice, then GC | All passed; exact refs/tag object unchanged |

The 94 tests include 13 new regressions and all 10 existing tag-guard tests,
including the real valid-candidate flow and tampered assets. New cases also cover
SHA-1/SHA-256 read shapes, inconsistent old IDs, missing/malformed/error reads,
symbolic alias replacement, and a no-op row followed by an unauthorized new tag
in the same transaction. The latter rejects the whole transaction.

The first broad test invocation omitted the repository runner's `PYTHONPATH`
setup, producing two import errors. That failed log and unchanged source bookends
are preserved separately; correcting only the runner environment produced the
frozen passing run. No production source was changed to hide that failure.

## Retention and boundaries

`receipt.json` records platform, exact commands, source hashes and archive hash.
`runs.tar.gz` retains 28 payloads plus an internal checksum list: red/green logs,
original transaction traces, test runner, both source bookends and the exact
changed source plus unchanged hook. Outer `SHA256SUMS` covers this note, receipt
and archive. No temporary repository or formatter installation is needed later.

All deliberate tag/configuration experiments were confined to temporary test
repositories. No workspace Git configuration, actual release tag, receipt
requirement, host protection or remote was changed. Existing valid-release tests
use their named fixture receipts only in isolated repositories. Main still used
the old guard until root integration, so its automatic GC rejection during the
source commit is not represented as a passing main maintenance run.

Root integration and its checks remain required. This tooling result grants no
Kernel #10 geometric, naming, package, sanitizer or application acceptance.
