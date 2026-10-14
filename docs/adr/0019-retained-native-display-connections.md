# Refuse replacement of retained native display connections

Decision: **native-display-lifecycle behavior v1**, 2026-10-14. This is separate
from the delivered-window-destruction decision in ADR0018. No workspace version
change or publication is authorized here; the coordinator owns that decision.

## Evidence and scope

The revision-bound diagnostic at `c814d12b3` already reproduces shared
`AspectDisplayConnection::init(0)` while an owned window and Vulkan view retain
the connection. The original native display is closed; subsequent queries use a
replacement connection with the old XID. The preserved runs include fatal
`X_GetWindowAttributes/BadWindow` and an instrumented allocator abort. The latter's
precise mechanism remains unproven. Neither is attributed to the historical
intermittent GUI failure.

An ordinary `Handle` retains the Rust wrapper, whose native pointer remains
mutable. It does not retain the native connection lifetime. Existing Vulkan
retention until instance teardown must therefore retain a lifetime lease.

## Behavior v1

A per-connection lease records a native consumer: an owned/wrapped XwWindow or a
Vulkan instance. Acquisition and reset admission use the same connection-state
lock. Reset with any consumer returns a typed refusal before closing/opening a
display, clearing atoms, or modifying metadata. Ordinary aliases are not consumers.
The refused operation preserves the old connection and its window/view results.

XwWindow acquires before native create/wrap calls and retains through destruction.
Failed setup releases its temporary lease. Conservatively, even a closed window
object retains its lease until drop. The Vulkan runtime retains a lease through
`destroy_instance`, including after individual views or surfaces are removed.
Deduplication uses retained connection-object identity, never numeric `Display*`
equality. Different wrappers of an externally owned address remain distinct;
they do not establish or certify that external address's lifetime.

`init_checked` returns immutable operation, connection identity, original display,
requested replacement, consumer counts, failure classification, cause and behavior
version. A native initialization error retains the original `Failure` as its
source. Renderer admission errors retain that typed display cause. The result
means completion or refusal; a refused reset does not retire the window or serve
as a successful reset.

Unleased initialization retains the owned fixtures pinned before refactoring at
`da38095c0`. In particular, this bounded change preserves two existing hazards:
self-aliasing an owned native pointer, and failed unleased reset closing the prior
connection. Fake traces document these hazards without executing a freed pointer.
Applications should create another connection instead of relying on either case.
XwWindow native calls, destruction, delete-atom lookup and renderer admission now
use the original leased binding through `AspectWindow::native_display_connection`.
Replacing `window_base().fields_mut().display` changes metadata only: it cannot
redirect an existing XID to another connection. The trait default retains metadata
behavior for custom window implementations; their binding ownership is their
responsibility. Other protected-field mutation (including XID/visual/configuration),
external close/reset of borrowed or independently aliased raw displays, and
unreported external window destruction remain outside this acceptance scope.

The explicit non-null `invalidate_content(display)` override retains its documented
selection behavior. A temporary checked Window lease spans the send callback (included in window consumer counts); the caller
still supplies a live same-server XID and owns any raw external lifetime. Null uses
the pinned window binding. Failed lease admission produces no send: the existing
unit-return adapter loses the typed failure and is not evidence of successful
invalidation. This decision does not certify event delivery or deprecate the
explicit override; replacing that API requires its own migration decision.

## Migration and deletion task

Existing `init` keeps its signature as a legacy adapter to `init_checked`. Its
`Failure` preserves prior native-error formatting and names lifecycle refusal,
but **does not carry the structured lifecycle payload**. New callers use
`init_checked` and match its typed cause. Existing native error signatures on
legacy constructors remain unchanged in this bounded correction.

Deletion task **NATIVE-DISPLAY-INIT-ADAPTER**: migrate all `init` callers to the
checked result and remove the legacy entrypoint and its internal lossy mapping
where no legacy constructor requires it. This adapter has a maximum lifetime of
one migration milestone: delete it by completion of the next GitHub milestone
following v1 integration. The coordinator must assign the deletion task to that
milestone before closing the current milestone; retaining it longer blocks that
milestone's acceptance. Removal requires the corresponding semantic-version and
caller migration decision, not publication by this ADR.

Gate: owned unleased fixtures unchanged; named-fake refusal has zero native calls;
lease counts, setup failure and teardown order pass; ordinary host-thread
admission and Miri results are recorded; native reset refusal leaves original
window/view captures intact on both drivers. Kernel acceptance remains separate
from application acceptance, performance and concurrent native window operations.
Earlier unsafe shared reset remains available only as revision-bound diagnostic
replay at `c814d12b3`, with the recorded migration to refusal in v1.
