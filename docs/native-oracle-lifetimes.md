# Martensite native capture ownership

The local `NativeTextPolicyProbe` owns its `V3d_Viewer`, view and workspace.
OCCT's `V3d_View::MyViewer` is a raw pointer, so retaining only the view does
not keep its viewer or graphic driver alive. The viewer is the first member
so C++ destroys it after the workspace and view.

The original software `nearest` texture capture exited 139 after the factory
returned. Retaining the viewer makes the same capture succeed twice on both
native AMD and llvmpipe. All 68 standard-texture modes were then captured twice
per profile. Every repeated buffer is identical, and all 138 AMD buffers,
including the intermediate failed-image capture, match the earlier oracle.

Rust Vulkan replay against these repaired same-profile native buffers passes
65/68 AMD cases and 66/68 software cases using exact RGBA and depth error at
most `1e-6`. Remaining failures are preserved without changing gates.

[The receipt and raw captures](../porting/runtime-evidence/native-viewer-lifetime-2026-10-12/integration.json)
bind the binaries, helper, profiles, repetitions and failed reproduction.
Other text/policy callers still need comparisons with the repaired factory;
the texture result does not establish their capture parity.

Run the original regression locally after compiling the texture oracle:

```sh
LIBGL_ALWAYS_SOFTWARE=1 target/martensite-surface-viewer-lifetime-after \
  nearest target/retained-viewer-nearest
```

Keep OCCT compilation and capture on the local porting host. Committed native
buffers remain available for Rust replay without loading OCCT in workflows.
