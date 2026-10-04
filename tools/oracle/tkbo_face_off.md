# Angular face selection

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The probe records 280 GetFaceOff observations on a box, cylinder, sphere, and
thin box: four reference-face orientations, empty candidate lists, adjacent
faces, same/reversed occurrences, and duplicate candidates. Rows capture status,
ambiguity, selected face identity/orientation, and retained list length.
Replay: `cargo test -p rustycad-tkbo --test face_off`.

The source translation includes the approximate angle formula, minimum spatial
step, small-face detection, iterative surface/plane projection, and hatching
fallback. It preserves ambiguity checks before negative-angle normalization,
strict minimum selection and unchanged output for empty lists.
CoupleOfShape retains independent occurrences sharing topology.

Partial coverage: general freeform supports, large coordinates/radii, singular
normals, arbitrary locations, pathological projection failures and complete
legacy/API parity still need proof. Debug-only native diagnostic output is not
reproduced. These are supplemental oracle cases, not newly mapped legacy tests.

The same 280 rows also record IsInternalFace(face, edge, first, last), including
identical boundary faces. Status 2 denotes angular ambiguity, distinct from a
caught failure. Replay asserts the row count to reject empty fixture output.
