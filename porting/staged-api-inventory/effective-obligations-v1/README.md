# Staged effective API obligations

This overlay adds demonstrated access routes to the primary declaration inventory.
It does not change native access or visibility and grants no implementation credit.
The published inventory and parity ledger are unchanged.

## Denominator policy

The bounded union has **108,639** unique native IDs: **108,622** primary candidates,
**15** inferred public members and **two** inferred record types. Original native
facts remain in every row; the complete native declarations remain in the hashed
`returned-types-v1/global` snapshot. The union is **not a complete denominator**.

The two records have separate type obligations because valid C++ expressions can
infer and use them. Those obligations cover only the operations listed in
`access-contracts.json`. Naming either private qualified type directly remains
invalid. Record access grants no access to private children and does not prove
every type operation, layout, special member, constraint or platform variant.

One native fixture demonstrates every added declaration. Its seven controls
cover successful access and six rejected forms. Sparse Block construction and
members run on valid aligned storage. The OSD interface is obtained from the
protected UniversalIterator alias through a derived class; a custom derived
implementation demonstrates virtual calls, cloning and base-pointer deletion.
The native abstract methods supply a customization contract, not default bodies.

All 7,352 returned-type observations remain in the immutable source snapshot,
including 7,341 unsupported template/dependent observations, five repeated/cycle
observations and six private-record traversals. No unresolved edge is treated as
implemented. Existing declaration conflicts and platform/configuration gaps also
remain open, including the conflicting public `math_Vector` alias.

## Reconciliation

| Scope | Current Rust evidence | Status |
|---|---|---|
| Sparse Block record, seven functions, three fields | Unique compiler declarations and current source hashes | 11 partial surfaces |
| Sparse Block Cell alias | `u8` used internally; no public named alias | One missing surface |
| OSD IteratorInterface record and four members | No equivalent public customization interface | Five missing surfaces |

The Rust inventory snapshot retains the complete TKernel crate from the current
published compiler capture. It is preserved separately so later inventory
publication cannot rewrite this receipt. Existing Sparse oracle, tests and
receipts remain historical evidence; they were not rewritten to grant new credit.

## Replay

```sh
PYTHONPATH=tools python3 tools/build_effective_api_obligations.py --verify
PYTHONPATH=tools python3 -m unittest tools.tests.test_effective_api_obligations tools.tests.test_effective_access_probe tools.tests.test_effective_obligation_artifacts
PYTHONPATH=tools python3 tools/probe_effective_api_access.py --source ../OCCT --output target/effective-access
```

The first command authenticates input hashes, recomputes every exact native ID,
rebuilds the union and reconciliation, and compares all staged rows. The final
command reruns native access controls against the pinned OCCT checkout.

## Next bounded ports

1. Add the public Rust alias for inferred `Block::Cell`; retain exact native ID
   and primitive identity without implying native nested-name compatibility.
2. Reconcile `Block::Size` constexpr evaluation and unsigned overflow, separating
   native size arithmetic from checked allocation layouts. Add the two ignored
   arguments to a faithful `ToArray` adapter while retaining current ergonomic
   callers. Use explicit overflow and pointer-offset native probes.
3. Port `OSD_Parallel::IteratorInterface` as a bounded Rust customization contract:
   equality, increment, owned clone and virtual destruction. Prove derived
   dispatch and ownership before adding wrappers. Existing `Parallel::for_each`
   must retain its support for non-Clone and borrowed iterators; do not introduce
   blanket `Clone` or `'static` bounds there. Native `IteratorWrapper`,
   `UniversalIterator` and `FunctorInterface` remain separate class obligations.

Each port needs fresh exact compiler bindings, original-test preservation and
native/Rust behavior probes. This staged inventory alone satisfies none of those
implementation acceptance criteria.
