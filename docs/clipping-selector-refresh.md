# Martensite clipping integration selector audit

This metadata checkpoint audits eight existing bindings after checked clipping
integration. It adds no chain, renderer or class completion credit. The old
compiler capture at `8d8fa3b8f` uniquely contains each ledger selector; the fresh
root union capture uniquely contains the same public paths, declaration kinds and
implementation contexts. Exact old/current declaration JSON and source diffs are
retained under `porting/runtime-evidence/clipping-selector-refresh-2026-10-12/`.
Native declaration IDs and pinned header hashes remain unchanged.

Six signatures are unchanged. CView SetCamera retains its exact assignment body;
Rust's right-hand-side evaluation precedes its destination write guard. The
original-green camera self-assignment native fixture and bounded Rust regression
preserve slot/target identity and counts; this audit introduces no setter fix.
CView Lights and SetLights remain pure virtual borrowed getter/parameter
contracts, with unchanged signatures. Their existing 118-record live-light
corpus covers actual selected slots, input retargeting, intrusive counts and
failure ordering. Changes to other CView methods alter the containing source
hash, without changing these declarations or their mapped behavior.

V3d Source Redraw and RedrawImmediate retain their exact default dispatch bodies
and signatures. Their inherited helpers now reread the actual protected
`driver_view_slot` using noncounting observations wherever native code rereads
`myView`. Active/defined checks, two retries, intentional owning structure-manager
local, AutoZFit, immediate invalidation and virtual redraw ordering are unchanged.
V3d SetCamera likewise retains its signature and typed-failure wrapper; only its
receiver acquisition changes from the old fixed field to the actual current
protected slot before forwarding the borrowed argument and ImmediateUpdate.
Pinned V3d_View.cxx Redraw/RedrawImmediate/SetCamera bodies and the existing
35-record source/count/failure corpus support these dependencies. Checked root
union tests and both-device clipping captures verify the integrated source.

DSM Camera and CView Camera are **not** purely unchanged signature refreshes.
Their returned boxed reference no longer borrows the temporary Rust receiver;
it captures the actual selected weak field once. The CView field backing is now
Arc-owned by the real base, while aliases hold Weak backing and add no CView or
camera ownership. Reads do not rerun the virtual getter. Native const Handle&
semantics require this selected-field identity across protected-myView changes.
The ten-record `v3d-clip-view-retarget-native.txt` fixture and its Rust replay
prove old Camera aliases stay on the old actual field, fresh aliases select the
new field, View aliases follow myView, and both camera counts remain one. The
prior camera-alias corpus additionally covers actual alternate selected fields
and direct XR myCamera assignments. Safe expiration beyond native valid lifetimes
is unclaimed; this audit retains implementation credit for valid native aliases.

Root's checked clipping union ran 6,223 toolkit tests, 292 Kernel doctests,
four-crate Clippy, all 24 clipping bindings and six unchanged strict capture
comparisons successfully. These existing integration checks support the refreshed
source; this metadata-only checkpoint requires canonical preflight, without
inventing new native execution or claiming full class completion.
