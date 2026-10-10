# Martensite AIS selection actions

Martensite implements the pinned OCCT context's selection actions, selected-owner
queries and selected highlighting. This adds 53 exact declaration mappings,
including `AIS_StatusOfPick` and its values. The interactive context as a whole
remains partial; this slice does not establish renderer or AIS_Shape completion.

## Observable behavior

Owner-array, detected-owner, pixel, rectangle and polygon entry points use the
existing selection container and viewer selector. Deprecated Select/ShiftSelect
wrappers retain their Replace/XOR schemes and requested viewer updates. Pixel and
area selection validate the viewer before recording the active view, then use real
sensitive geometry and camera projections.

Selection membership and an owner's selected flag are distinct. Queries retain
the container's public cursor; reading the first object does not move it. Owner
and object replacement, toggling and explicit selected-state changes preserve
native filter checks, forced highlighting and fast paths. A selected owner can
still be removed when a filter rejects it. Detected selection sends the native
mouse-click callback except for Remove/Clear; a consumed click leaves selection
untouched and returns NothingSelected.

Automatic highlighting computes the ordered difference between old and new
selection. Nonautomatic owners are grouped by selectable object for
HilightSelected/ClearSelected callbacks. Style lookups retain the original drawer
slot and read replacements made by AcceptDisplayMode. Selection traversal follows
live list links, including owners appended by callbacks. Context/selection locks
are released before calling user overrides. A weak self handle lets context
assignment recurse through interactive children without an ownership cycle.

Highlight flags, style handles, presentation changes and selection membership
remain independently observable. Invalid owner HighlightStyle queries leave an
output handle untouched, while an unhighlighted object clears it. Exceptions
preserve native partial mutations; successful cleanup is not synthesized after a
callback fails. Global highlighting also covers objects without a global owner.

## Evidence and limits

`tests/ais_selection_actions_test.rs` exercises these contracts directly.
`tools/tkv3d_corpus_ais_selection_actions.py` supplies 648 deterministic native/Rust
requests, covering schemes, filters, automatic/nonautomatic highlighting, forced
owners, displayed/erased/unregistered objects, drawer replacements, failure and
recovery, public camera picks, callback appends and context assignment. The shared
fixture also retains the existing 1,332 detection comparisons.

The native fixture calls the real pinned AIS_InteractiveContext algorithms. The
named object/owner/filter subclasses expose callbacks and errors; the scene uses
real selection geometry with a null graphics driver. These checks validate
selection and presentation behavior, not rendered images or the full universe of
client callbacks. Vulkan rendering needs its own legacy-capability evidence under
[the visualization plan](visualization-design.md#6-vulkan-implementation-sequence).

Run the Rust checks with `cargo test -p rustycad-tkv3d --all-targets`. Run the native
comparison and translated legacy checks with
`python3 tools/verify_toolkit.py TKV3d --source ../OCCT --native-build target/occt-all-full --jobs 2`.
