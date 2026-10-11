# Martensite AIS view input buffer

`AisViewInputBuffer` now ports every public record, field, constructor and enum
from OCCT `AIS_ViewInputBuffer.hxx` at `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The buffer carries fit/orientation, dynamic highlight, selection, pan, object drag,
orbit, view rotation and Z rotation requests, plus a one-based scroll queue.
`AisViewInputBufferType::Gl` preserves the upstream rendering-thread enum name;
these records contain no OpenGL or Vulkan dependency.

Reset acknowledges pending requests and clears only queued scroll actions.
Selection tool/scheme/points, orientation, gesture coordinates, pan deltas and
Z rotation angle persist. Replacing the buffer with a fresh default would destroy
state that the native view controller keeps between frames. Rust clones own
independent selection and scroll sequences, preserving native value-copy behavior.
Both rotation records retain distinct public types with shared declaration logic.

The nine native constructor defaults, all 41 public fields and both enums are
covered by committed numeric oracle assets. The local C++ generator writes idle,
pending, reset, repeated reset, copy and modified-copy snapshots plus enum values.
Four Rust regression tests consume every snapshot with no OCCT build or runtime
requirement. Constructor defaults and request acknowledgment are tested through
the public API. The header, generator and fixture hashes are recorded alongside
the asset in `crates/tkv3d/tests/fixtures/view-input-buffer-provenance.json`.

```rust
use rustycad_tkv3d::ais::AisViewInputBuffer;
use rustycad_tkernel::ncollection::Vec2;
let mut requests = AisViewInputBuffer::new();
requests.selection.points.append(Vec2::new(10, 20));
requests.selection.to_apply_tool = true;
requests.reset();
assert!(!requests.selection.to_apply_tool);
assert_eq!(requests.selection.points.length(), 1);
```

Run all Rust toolkit tests with `cargo test -p rustycad-tkv3d --all-targets --locked`.
During porting, regenerate the local reference with the existing pinned OCCT build:

```sh
c++ -std=c++17 -O2 -Itarget/occt-all-full/include/opencascade \
  tools/oracle/martensite_view_input_buffer.cxx \
  target/occt-all-full/lin64/gcc/libi/libTKernel.so \
  -Wl,-rpath,"$PWD/target/occt-all-full/lin64/gcc/libi" \
  -o target/martensite-view-input-buffer-oracle
./target/martensite-view-input-buffer-oracle > \
  crates/tkv3d/tests/fixtures/view-input-buffer-native.txt
```

The compiler-bound parity ledger maps all 68 captured declarations in this header.
This supplies the event records required by the remaining AIS view controller
port; it does not claim that controller behavior or interactive gestures are done.
