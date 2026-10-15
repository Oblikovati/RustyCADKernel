# ADR 0064: Production semantic names for the annular prism

Status: private implementation on a worktree branch (2026-10-15).

## Decision

`owned_request::annular::names::issue_annular_names(request, region, receipt,
binding)` replaces the test-only authored fixtures (`annular_fixture::authored`,
`annular_history::fixture::authored_names`) on the production path. It issues the
publication, selected-copy and intermediate-scope bindings plus an
`AnnularNameDictionary` that maps every issued token to the caller's meaning.

Tokens are lexical, never positional: `AnnularNameSlot { category, scope,
boundary role, kind, variant }` packs five explicit `u8` codes under the fixed
namespace `0x4d41 << 48`, so annular tokens cannot coincide with single-boundary
slot tokens. The boundary role is Outer/Inner by the owned request's explicit
roles; rejected candidates are coded by their authored region role (inner
domain, outer complement); trim and endpoint variants come from the bound chart
trim role and the exact native range endpoint. No token derives from traversal
rank, coordinates, proximity or process addresses.

The two-loop proof already binds the caller operation and region names as
authored identities, so the operation and selected-region tokens are the caller
values themselves. A caller value that equals an issued role token is refused as
a typed `Collision` instead of aliasing. Every other token is caller
independent; caller identities (operation, region, wire names, boundary IDs,
closure vertices) live only in the dictionary.

## Evidence

Production names publish the owned annulus end to end (six continued originals,
two retired wires) with identical replay across independent preparations. The
issued publication names have exactly the equality pattern of the independently
authored fixture; the history names refine it, differing only where the fixture's
numeric scope bases coincide by accident (InputCopy outer/inner versus
Partition/CandidateCopy). Every published face key resolves: caps to the caller
annulus, sides to the caller boundary IDs. Changing caller IDs changes only the
dictionary and the two caller-valued tokens.

## Limits

Names cover the two nested closed polynomial loops only. Split identity across
general profiles, persistence of references across edits, save/reopen and
undo/redo remain open.
