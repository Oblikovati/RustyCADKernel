# Selection images

`SelectMgrViewerSelectorImageMembers::to_pix_map` ports OCCT `3d097a0`
`SelectMgr_ViewerSelector::ToPixMap`. It performs public camera-based picking for
each image pixel in row-major order and fills the requested one-based hit rank.
`StdSelectTypeOfSelectionImage` preserves all nine numeric modes and unknown-value
handling. Empty images raise ProgramError before an unknown mode returns false.

`select_mgr_create_selection_image_filler` returns a borrowed boxed
`SelectMgrSelectionImageFiller`. Custom implementations provide `fill` and may
use the default no-op `flush`. `SelectMgrSelectionImageFillerBase` exposes the
native protected image, selector, Bullard generator and unique-color set. Rust
lifetimes encode the native non-owning image reference and selector pointer;
no image or selector lock spans virtual picking or sorting callbacks.

Depth normalization accumulates float pixels with double extrema, casts each
bound independently to float, and writes only at flush. Missing-hit sentinels
remain black even for inverted depth. Raw depth uses native RGB validation and
can fail outside [0,1]. Failures preserve preceding writes and skip final flush.
Surface normals use native float normalization and remapping.

Entity, runtime-type and owner palettes scan all recorded selections, including
inactive selections. Duplicate keys consume no random draw. Entity and owner
keys retain handles. Mode lookup uses the last matching selection, standard
mode colors, and lazily generated custom colors. Generated colors reproduce the
pinned GCC right-to-left constructor argument evaluation and native color
hash/equality semantics.

Eleven direct Rust tests cover these contracts. The native differential corpus
adds 2,254 cases across nine modes plus unknown kinds, eight image formats,
orientation, padding, rank errors, duplicate memberships, custom modes, float
rounding, extreme normals/depths, empty/unfilled images and public picking
failures. Both fixtures retain replaced selections: native selection destruction
clears entity owners, so releasing those selections before synthetic hit seeding
would make the fixture invalid.

Run `cargo test -p rustycad-tkv3d --all-targets` and
`python3 tools/verify_tkv3d_numeric.py --source ../OCCT --native-build target/occt-all-full`.
Both `DisplaySensitive` overloads are now mapped; see [sensitive display](sensitive-display.md).
