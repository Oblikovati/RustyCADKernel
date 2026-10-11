# Martensite AIS camera frustums

`AisCameraFrustum` ports the defined behavior of `AIS_CameraFrustum` at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. Create it with
`AisCameraFrustum::new()?`, then call `set_camera_frustum(&camera)?` before using
edge or volume selection. The object snapshots the camera without retaining it.
A null camera preserves cached geometry and presentation update state.

The constructor installs white solid borders, plastified front/back materials,
explicit blending, transparency 0.7 and shaded display mode. Shaded computation
adds triangles and borders; wireframe adds borders only. Empty and unsupported
presentations still become infinite, matching native behavior. Color changes
keep the aspect handles live in existing groups. Unsetting color restores white
aspect colors while retaining the drawer's last requested color. Unsetting
transparency clears both material and drawer transparency, preserving blend mode.

Camera updates keep the eight-vertex primitive arrays and their buffer identities.
The twelve triangles preserve native face order and winding. The twenty-four
border segments deliberately include each geometric edge twice, once per face.
Double camera corners narrow to single precision when stored in graphic buffers.
Repeated updates schedule presentation recomputation without rebuilding selection.

Edge selection contains one group of twenty-four sensitive segments with a common
owner. It snapshots single-precision endpoints. Volume selection aliases the
triangle buffers; its center and bounds remain cached until selection is rebuilt.
This distinction is retained rather than silently refreshing old selections.

The native implementation leaves its ninth, unused point uninitialized and
null-dereferences geometry when selection is computed before a camera is assigned.
Martensite initializes unused points to zero and returns a typed null-object error
for that invalid selection call. Defined upstream paths retain native behavior.

`tools/oracle/martensite_camera_frustum.cxx` generates the committed native fixture
locally. Its two executions were byte-identical. The provenance JSON records the
upstream header, generator, null driver and fixture hashes. The 120 snapshots cover
eight orthographic/perspective/stereo/mono-left camera configurations, default and
changed appearance, all indexed vertices and edges, selection centers/bounds and
owner identity, empty/populated presentation modes, retained arrays and selection
lifecycle. Ten Rust tests replay these assets without OCCT at test time and
exercise live aspects, camera independence, error boundaries and enum values.

Run the affected toolkit with `cargo test -p rustycad-tkv3d --all-targets --locked`.
Renderer submission interfaces are unchanged; this slice uses the existing indexed
triangle, segment and aspect paths. Strict Vulkan raster/depth and performance
gates remain tracked by the renderer work. Windows/macOS validation is deferred.
