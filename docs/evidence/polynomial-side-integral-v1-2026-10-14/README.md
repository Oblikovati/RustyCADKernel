# Conditional polynomial extrusion-side integral v1

Decision: [ADR0024](../../adr/0024-conditional-polynomial-extrusion-side-integral.md).
Base: `0e1f2ef8f`; shared square-root extraction: `19d6c14fd`;
additive source freeze: `e60e9b30b59d837184956b277f9ad09e37a7469d`.

The new operation certifies a conditional scalar integral over the full native
[0,1] polynomial domain. It does not bind actual faces, validate geometry or
complete the region/prism operation. No TKBO, native OCCT or GPU work is included.

## Independent expectations

`ph-jacobian.json` retains canonical rational inputs and actual Jacobian
`polynomial.sos.decomposition.check` outputs. Both checks returned `is_valid=true`:

    (3-3t²)² + (6t)² = 9t⁴ + 18t² + 9 = (3+3t²)².

The cubic controls `(0,0),(1,0),(2,1),(2,3)` therefore have speed `3+3t²` and exact
integral 4 for unit orthogonal extrusion. The reversing quadratic with controls
`0,-1,1` has coordinate `3t²-2t`, stationary parameter 1/3 and exact total
variation `1/3+4/3=5/3`. These expectations are independent of the Bernstein
bounding implementation. Line norms, constant/parallel/zero cases, affine shear,
degree ladders and extreme-scale identities supply further closed forms.

The active closed cubic with controls `(0,0),(4,0),(0,4),(0,0)` and extrusion
`(0,0,2)` exercises completion under the finite default budget. Its computed
interval is not used as a replacement independent oracle or accepted face area.

## Structural prerequisite and development records

The original upper square-root implementation was tested on base `0e1f2ef8f`
with the new owned upper-endpoint baseline test added before extraction:
`sqrt-before-rust199.log` records 4 passing tests. After endpoint extraction,
`sqrt-after.log` records 5 passing tests, including the new lower endpoint.
Those development runs preceded commit `19d6c14fd`; unlike the final matrix,
they did not capture full source-hash bookends. Commit `19d6c14fd` retains the
baseline test and the exact structural diff against its parent. Later cleanup
only simplified delegation/test formatting; final frozen tests cover that code.

`sqrt-before.log` preserves the initial command's system Rust 1.95 rejection
before compilation. Subsequent checks explicitly select Rust 1.99.0. Development
Clippy failures are retained: the initially large error was boxed, and a
noncanonical `PartialOrd` delegation was restored to the canonical total-order
form. These failed/pre-freeze runs are not final acceptance evidence. Early
workload diagnostic rows used Rust Debug formatting; final workload rows use
structured JSON. All original logs remain unedited in the development archive.

## Final matrix and limits

All frozen checks passed: 27 focused tests; 203 TKMATH unit tests; 2,926 toolkit
tests; 52 doctests; documentation build; all-target/all-feature Clippy with
warnings denied; formatting and diff checks. There were no failed or ignored
tests and no warning/error lines in the final logs. All 15,627 recorded
source/build/fixture files matched before and after. The literal post-format
audit found 95 new side-module functions of 4–20 lines and a maximum source
file length of 163 lines.

`frozen-e60e9b30b.tar.gz` contains logs, full source-hash bookends, source-style
audit, exact command receipt and the runner. `frozen-integrity.json` records
each member's SHA256; `SHA256SUMS.json` covers the evidence files themselves.
`workload-policy.json` retains the final structured workload records.

The final matrix uses Rust 1.99.0, two Cargo build jobs and one test thread in
this isolated worktree. `checks.json` records exact commands, outcomes, host,
compiler, executable hashes and source-bookend comparison. Archived source
manifests cover tracked crate source/fixtures/build inputs and workspace Cargo,
lockfile and toolchain configuration. Source changed during no accepted run.
The executable paths in the receipt identify artifacts at check time; worktree
cleanup may remove them after coordinator integration and replay.

Concurrency evidence consists of compile-time Send/Sync assertions, immutable
result reads, ordinary-host independent operation comparisons and byte-identical
serial/threaded subprocess records. It is not a TSan result. The retained
[ordinary-host TSan startup failure](../native-display-lifecycle-v1-2026-10-14/TSAN.md)
remains incomplete; this side-integral suite was not built or run under TSan.
No unsafe Send/Sync implementation was introduced. No performance comparison,
Miri run, application acceptance or topology/regularity claim is included.

Default policy is area error 1e-5, degree cap 16 and 4096 subdivisions, chosen
from actual bounded owned workloads. Limits are never widened. Work exhaustion,
cancellation and unrepresentable conservative bounds remain incomplete even if
the mathematical integral itself could fit in binary64. Diagnostic intervals
on errors are not completed results. Source/entity binding and total world-area
budget composition remain with the owning region transaction.
