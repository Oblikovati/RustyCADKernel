# Martensite context isoline defaults

`AisInteractiveContext` exposes global isoline counts, planar and triangulated
isoline policies, and relative and angular meshing deviation defaults. These
methods change the current default drawer. They do not redisplay existing
objects; object-specific overloads remain separate porting work.

Counts retain the OCCT behavior, including negative values. Reading `Both`
returns the shared U/V count or `-1` when they differ. Replacing the V aspect
with the actual U aspect makes subsequent V changes visible through U too.
The representable unnamed native enum value `3` does nothing on assignment
and returns zero on query, even when the default drawer is null. Martensite
names this Rust case `Unnamed`; there is no corresponding named upstream
enumerator. Other null-drawer queries are not covered by this defined case.

Deviation setters preserve zero and negative inputs and the drawer's previous
values and ownership flags. A replacement drawer can inherit initialized
defaults from a live link; later changes to that link remain observable.

Two independent executions of the local OCCT probe at revision
`3d097a0328e71b826377d4814ab05ec3c3d23871` produced 22 policy rows and one
null-drawer row. Rust replays the committed assets with exact floating-point
bit comparisons. The initial probe omitted initialization of the replacement
link's iso aspects and dereferenced a native null aspect. The corrected probe
calls `SetupOwnDefaults` on that actual link; the invalid setup is excluded
from the oracle assets. OCCT is only needed to regenerate these assets.

Run the Rust checks with:

```sh
cargo test -p rustycad-tkv3d --all-features --all-targets
```

Native outputs, source hashes and check logs are recorded under
`porting/runtime-evidence/context-isoline-policies`. This narrow default-policy
proof does not establish meshing output or whole-context completion.
