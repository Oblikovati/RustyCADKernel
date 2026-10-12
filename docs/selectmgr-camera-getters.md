# Martensite selecting-volume camera getters

`SelectMgrFrustumBuilder::camera_ref`, `SelectMgrBaseIntersectorMembers::camera_ref`,
and `SelectMgrSelectingVolumeManager::camera_ref` return live aliases of the actual
camera fields. Import `HandleReference` to inspect them or explicitly call
`clone_handle` when native ownership is desired. Existing copied `camera()`
conveniences remain separate unclaimed APIs.

For example, `let old = manager.camera_ref(); manager.init_selecting_volume(&next);`
keeps `old` bound to the originally selected volume's camera field. Later writes
to that field remain visible through `old`; it never repeats manager selection.
A getter called with no volume captures the same native function-static null slot
shared by all managers, even after the manager gains a volume.

Actual camera storage uses `Arc<SharedState>`; aliases capture only a weak field
reference. There is no additional camera copy, source owner, retained lock, or
mirrored field. Existing camera field reads/writes keep their ordinary
`.read()`/`.write()` access. The public intersector camera backing type changes
from `SharedState<Handle<Graphic3dCamera>>` to `Arc<SharedState<Handle<Graphic3dCamera>>>`;
external base literals wrap the backing state in `Arc::new`. The builder alias explicitly captures no self borrow,
so native-valid retained references do not prevent subsequent mutation.

Thirty pinned native CPU rows and Rust-only replay verify field identity, exact
counts, null/A/B assignments, old/new volume selection, static null identity,
qualified setter side effects, typed failure, unrelated panic, and retry. Additional
Rust tests prove explicitly copied ownership and safe expiration without native
owner retention. Expiration is outside the native valid owner lifetime; dangling
C++ alias/reallocation cases are excluded rather than reproduced.

Only these three getter claims are restored. Constructor/class and full Frame
contracts remain partial. The original Frame checkout is preserved independently.
