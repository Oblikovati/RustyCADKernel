# Continue after the selector traversal/pick checkpoint

Use `/tmp/rustycad-resume` on `porting/resume-claude` at parent
`7fac5b5bf9ca8b35f7b33b8462dfa1a7943e14d5`. Git metadata is read-only in this
session. Preserve every original Claude checkout and the 25 committed
continuation commits outside main. Do not retry integration by bypassing policy.

Source pin: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Scope remains 56 toolkits / 76,041 APIs. This slice maps seven native declarations:
checkOverlap, traverseObject, TraverseSensitives and four Pick overloads.
Fourteen direct tests and 1,408 new numeric requests are registered in normal gates.
Native private bodies are invoked through explicit member-pointer access in fixtures;
no replacement implementation stands in for the native overlap/traversal code.

The new modules are `viewer_selector_overlap`, `_walk`, `_clipping`, `_object_transform`,
`_object`, `_traversal_view`, `_traversal`, `_pick`. The sorter body is publicly
exported so derived selectors can perform the native base call without duplication.
The native shared null driver's SetClipPlanes now stores its argument like the Rust driver.

Fidelity details to preserve:
- Capture selectable before Matches, reread live owner afterward and at new insertion.
- Existing owner count advances before point update; box repeats skip point update.
- Only exact TMF_2D plus presentations prefers display priority; actual AIS lookup matters.
- Object/entity persistence uses the original view camera; bounds bypass for persistent
  entities requires a camera, whereas flipping always bypasses sensitive bounds.
- Flipper world-view includes object model transform. Inverse order is flipper * inverse.
- Global box clipping still applies after object override. Partial leaf clipping rejects
  non-overlap selection except a touching box.
- Owner-complete filtering affects only owners newly inserted by this object traversal.
- Five subsets, with native matrix mutation across subset order, not independent matrices.
- Sentry covers clear through virtual SortResult; failure leaves completed unsorted results.
- No-camera traversal preserves the prior selector camera frame/scale.
- Point Pick assigns tolerance; Box does not; Axis assigns neither camera nor window size.
- Null-window Pick safely reports NullObject after volume/camera initialization.
- Native polyline orientation assumes one-based inputs; arbitrary negative lower bounds
  throw during Build. Valid traversal fixture geometry uses a one-based polygon.

A narrow follow-up before declaring exhaustive callback parity: native TraverseSensitives
holds a reference to the original active intersector camera slot, while the Rust frame
captures a Handle. A Matches callback replacing that slot may change later native
persistence transforms. Shared camera-object property edits already remain visible.
Use a targeted native regression and preserve object/subset/entity callback timing.
Do not silently claim graph mutations that invalidate native borrowed references.

Remaining ViewerSelector methods are DisplaySensitive (two overloads) and ToPixMap.
- DisplaySensitive depends on unported SelectMgr::ComputeSensitivePrs, with many sensitive
  primitive types. Existing ClearSensitive is ported and native-compared.
- ToPixMap calls SelectMgr_SelectionImageFiller::CreateFiller, then row-major Pick/Fill,
  followed by Flush; empty image is ProgramError, unknown filler kind returns false.
- SelectionImageFiller has eight private concrete fillers: entity colors, runtime-type
  colors, normalized depth, unnormalized depth, owner colors, selection-mode colors,
  detected object attribute colors and surface normals. Inspect handle lifetimes, map iteration,
  deterministic native random color generation, float depth rounding, missing-pick alpha,
  image formats and row orientation before porting.
- Sources are in OCCT/src/Visualization/TKV3d/SelectMgr; enum is under StdSelect.

AIS context display/erase/remove and interactive selection lifecycle remain another
substantial continuation route. AIS context and ViewerSelector class rows stay partial.

Verification commands use PATH with rustup 1.98.1 toolchain and ~/.cargo/bin and
CARGO_TARGET_DIR=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target.
Normal command: python3 tools/verify_toolkit.py TKV3d --source <OCCT>
--native-build <main>/target/occt-all-full --jobs 2.
Freeze crates/tools/ledger before gate + full rustdoc inventory. Compare all gate source
hashes again before archiving. Historical snapshots must not be replaced.

Further image-filler inspection:
- Constructor color scans include every recorded selection/entity, without activation
  filtering despite their comments. Duplicate entity/type/owner keys consume no RNG draw.
- DetectedObjectColorFiller reads Selectable()->Attributes()->Color(); no AIS cast is used.
- Random colors use three Bullard NextInt()%256 draws, divided by 255 and interpreted as
  sRGB. Preserve the native compiler's constructor argument evaluation order in the oracle;
  inspect actual channel order rather than assuming left-to-right C++ evaluation.
- Selection-mode search breaks only its inner entity loop: the last selection containing
  a shared entity wins. Known modes use native named colors; custom modes allocate colors
  lazily. Mode -1 remains black even if a selection with mode -1 contains the entity.
- Normalized depth records extrema in double but stores each pixel as float. Flush casts
  both extrema separately, subtracts in float, and resets delta to 1 when <= float epsilon.
  Sentinel +/-float max pixels become opaque black, even for inverted depth.
- Surface normals normalize the float vector before component mapping x*.5+.5; zero normals
  remain black. Image pixel conversion remains owned by existing TKService ImagePixMap.
