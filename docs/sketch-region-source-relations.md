# Martensite private validation source relations

The ADR0014 validation adapter now consumes complete operation-local
`OriginalMemberIdentity` relations from the already-owned arrangement stage.
Preserved, modified and generated members remain separate. Each key retains its
caller source ID, original topology token, actual location identity and
orientation. The token is provenance within this operation; it is never a
persistent name or a pointer used to recover geometry.

`source_relations` captures arrangement metadata, and
`validate_owned_arrangement` passes the actual owned candidate to the existing
ordered validation stage. Neither function changes mapped checker status,
context, failure, numerical or validity behavior. The stage remains private and
incomplete: analytic incidence, self-intersection and tolerance acceptance are
not provided by this adapter.

Output lookup uses exact `IsEqual`, including location and orientation. All
matching rows contribute complete member sets; deduplication uses full key
equality inside each relation kind. Equal numeric placement matrices are not a
substitute for native location identity. Same-ID members at different locations
or orientations remain distinct, and an original member can retain more than
one relation kind.

Each diagnostic's affected occurrence and optional context are attributed
independently. Root source IDs are derived conveniences, never replacements for
member relations. An absent row, an empty parent set, or an unknown matching row
keeps attribution `None`. Context attribution does not inherit an affected
occurrence's or container's parents.

Ten Rust regressions cover kind retention, exact-key deduplication, output
orientation/location, equal numeric transforms with distinct location keys,
unknown attribution, real checker context faults, typed failure causes, and the
actual aggregate-copy/General-Fuse-to-validation path. Existing p-curve and
coverage tests continue to exercise mapped behavior. The tests do not run OCCT
and do not claim a completed region-to-solid operation or immutable published
application result.
