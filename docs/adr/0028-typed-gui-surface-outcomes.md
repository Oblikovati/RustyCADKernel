# ADR 0028: typed GUI surface outcomes

Status: accepted; implementation complete, final verification pending.

Owner: Martensite Vulkan maintainer; application consumer Oblikovati#3630.
This is active application-blocker priority 2, independent of the C1 geometry work.

## Decision

Add immutable `RenderGuiSurfacePolicy::{LegacyV1, CheckedV2}` selection through
additive driver constructors. Existing constructors keep `LegacyV1`. The checked
policy applies to GUI-attached work; ordinary model rendering, map/unmap routing,
native closure, and display admission retain their existing contracts.

The public inspection seam is `RenderError::gui_surface_outcome()`. It separates
a surface-change-only refusal from a primary renderer fault accompanied by an
observed surface change. The latter retains the original `RenderError`, typed
window/display causes, and error source chain. Consumers must propagate that
primary fault. `primary_failure()` is the immediate retained layer, not a leaf
identity. `observed_failure()` exposes the latest distinct real observation fault
at that layer. Repeated observation replaces this one companion rather than
appending a collection; the original primary remains fixed. Closure has a
separate typed cause. Compound observed errors retain their own primary, closure
and surface evidence. Follow the primary/source chain and companion getter;
`FaultWithSurfaceChange.fault` may itself be compound. No message parsing or
automatic retry is part of this contract.

A change records the operation phase, accepted extent, observed extent when
known, separate target and composition revision evidence, and whether the current
packet needs replacement. An out-of-date WSI result alone does not establish a
new extent. Surface loss, device loss, allocation failure, and timeout remain
faults. Suboptimal presentation alone does not establish a resize.

Target revision is operation-local view evidence, distinct from extent and
composition replacement. Delivered resize and explicit rebind invalidate an old
target even when its extent or native handle compares equal. Refusing an older
draw must not invalidate a newer packet installed after that draw started.

Successful publication and capture require a current target and current packet.
They never substitute a previous successful image for refused or failed work.
Native checks use current surface capabilities, not only a cached swapchain
extent or application wrapper metadata. Unknown (`u32::MAX`) or zero native
capability dimensions refuse authoritative observation as a real fault; requested
or clamped dimensions are not presented as native observations. Legacy swapchain
configuration selection is unchanged. These are observations, not an atomic
guarantee against arbitrary unreported native mutations.

## Presentation ownership

Keep acquisition, submission, and presentation ownership bookkeeping intact.
Vulkan considers certain rejected presentations, including out-of-date results,
enqueued for synchronization purposes; their semaphore waits still execute.
Typed classification follows that bookkeeping, including pending presentation
fences and acquired-image release. See the primary
[vkQueuePresentKHR specification](https://docs.vulkan.org/refpages/latest/refpages/source/vkQueuePresentKHR.html)
and [vkAcquireNextImageKHR specification](https://docs.vulkan.org/refpages/latest/refpages/source/vkAcquireNextImageKHR.html).

The existing legacy acquire recreation remains revision-bound replay behavior.
Checked GUI work refuses positively observed target change without adding a
retry loop. Private operation-local presentation probes have a production no-op
adapter and named fake/native-resize test adapters; there is no public test hook
or global mutable injection state. Any failed native GUI operation retires its
swapchain through the existing release/fence path, including a failure after
acquisition or submission but before present. Checked out-of-date acquisition
marks the chain stale before refusal. Recovery may use the same presenter but
cannot reuse signalled binary semaphores or an acquired image from the failed
operation. A target closing during checked redraw/capture stores a typed failure,
retains an earlier fault, and cannot leave a successful stale frame published.

## Baseline and migration

Source baseline: `7f27d9b17b99fab5a93e755e331a14bf3bcf5270`.
Owned legacy fixtures pin current simultaneous resize/fault ordering and capture
ordering before structural or behavioral changes. The old publication ordering
can replace a device failure with a composition-change error; capture can replace
a stored fault with a stale-packet error. CheckedV2 deliberately changes this
ordering to retain both pieces of evidence. Legacy presentation also treats
out-of-date as success; CheckedV2 refuses affected GUI publication.

Migration task `GUI-SURFACE-LEGACY-DELETE`, owned by the Martensite Vulkan
maintainer together with the Oblikovati#3630 consumer: adopt CheckedV2 in the
application, pass its command-loop race gate against a tested release, record
the migration and semantic version decision, then remove the superseded legacy
path within the next milestone after that adoption. A default change or deletion
is not authorized by this additive implementation. No package release is claimed.

## Verification gate and limits

Use named fake backend/WSI fixtures for exact fault precedence, equal-extent
revision changes, fresh-packet state, rejected replacement atomicity, and native
synchronization ownership. Exercise real native resize synchronously at acquire,
pre-submit, pre-present, and post-present/pre-publication seams on AMD and
lavapipe. Preserve original failures, exact framebuffer baselines, source hashes,
driver and validation-layer identity, and all settings. Run the existing native
map/unmap, closure, display-lifetime and fresh-composition gates unchanged.

Retain paired LegacyV1/CheckedV2 timing and peak-memory measurements for identical
accepted workloads using the existing harness. Record host contention explicitly;
shared-host results do not establish controlled release performance.

Kernel acceptance is separate from application acceptance. This decision does
not claim physical presentation visibility, detection of an unreported resize
away and back wholly between observations, or arbitrary concurrent native/view
operation safety. Thread-safety claims require their own host concurrency and
ThreadSanitizer evidence; prior host startup failures remain incomplete evidence.
