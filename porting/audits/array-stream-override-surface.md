# Remaining Standard_ArrayStreamBuffer override surface

Audit against OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` and Rust
`2a6f4eb0`. This records a remaining API gap; it adds no implementation or tests.

The native header exposes virtual `Init` (lines 60–64) and protected virtual
`underflow`, `uflow`, `pbackfail`, `showmanyc`, `seekoff`, and `seekpos`
(lines 66–94). The Rust concrete type supplies read, peek, availability, integer
putback, and seek adapters, but has no corresponding trait/base override surface.
Direct calls to those concrete algorithms do not prove subclass-dispatch parity.
The class and `Init` ledger claims therefore remain partial.

The native implementation's ordinary valid-position behavior is represented:

- Peek/read: `Standard_ArrayStreamBuffer.cxx:40–60`.
- Putback, including signed character promotion: lines 64–72.
- Availability: lines 76–83.
- Seek and absolute-position adapter: lines 87–133.
- Bulk read: lines 137–151.

Native `seekoff` also ignores `openmode` and has a default direction branch that
leaves position unchanged. Rust `SeekFrom` models the three normal directions;
it does not expose an arbitrary integer direction argument. This is an additional
input-surface limitation, not evidence of mismatching normal seeks.

Before-start and beyond-end native pointer arithmetic cannot be used as defined
oracle behavior. Rust's bounds rejection must not be replaced by invalid memory
access to make those cases superficially match. A future override port needs
shared state/dispatch tests in addition to the existing concrete legacy tests.
