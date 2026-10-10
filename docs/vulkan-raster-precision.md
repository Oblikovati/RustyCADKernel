# Martensite persistence raster precision diagnosis

The 34 native AMD persistence capture differences are explained by rasterizer
precision, not by different persistence matrices. This finding does **not** turn
the existing raw native comparisons into passes. Their exact-RGBA / `1e-6` depth
gate remains unchanged, as do the production renderer and timing baselines.

Mesa 26.1.6's [radeonsi viewport state](https://gitlab.freedesktop.org/mesa/mesa/-/blob/mesa-26.1.6/src/gallium/drivers/radeonsi/si_state_viewport.c)
selects 12 fractional coordinate bits for these small viewports. Its
[RADV command state](https://gitlab.freedesktop.org/mesa/mesa/-/blob/mesa-26.1.6/src/amd/vulkan/radv_cmd_buffer.c)
uses eight. Both APIs' advertised minimum precision alone did not expose this
difference. Downloaded source hashes are recorded in
[mesa-sources.json](../porting/runtime-evidence/raster-precision-2026-10-10/mesa-sources.json).

## Independent depth-plane check

For the unmodified `64-64-0-0-pers-axial` scene, reconstructing the triangle plane
from the recorded clip matrix and separately rounding its screen coordinates
predicts the actual depth buffers. Maximum error on foreground pixels:

| Coordinate precision | Native AMD OpenGL | AMD Vulkan |
| --- | ---: | ---: |
| 8 fractional bits | `1.40e-6` | `2.74e-8` |
| 12 fractional bits | `2.95e-8` | `1.38e-6` |

The [calculation](../porting/runtime-evidence/raster-precision-2026-10-10/reconstruct-plane.py)
and [results](../porting/runtime-evidence/raster-precision-2026-10-10/planes.json)
include both software outputs too. Their depth planes fit effectively unsnapped
coordinates more closely. Coverage precision therefore cannot by itself predict
depth interpolation behavior on every driver.

## Reproducible control

The native test oracle accepts an explicit `raster8` argument **only** for its
red persistence-triangle fixtures. A custom native vertex shader retains OCCT's
projection, world-view, model and persistence state, then snaps projected XY to
1/256 pixel. It leaves clip Z/W unchanged. This is an experimental control, not
OCCT's default shader and not a Martensite production shader.

```sh
python3 tools/verify_vulkan_raster_precision.py \
  --source ../OCCT --build target/occt-all-full \
  --receipt porting/runtime-evidence/vulkan-persistence-main-2026-10-10.json \
  --output target/raster-control.json --captures target/raster-control-captures
```

This command reuses the original Vulkan raw buffers only after verifying their
renderer source fingerprints and current device/driver identity. It requires all
114 persistence scenes on each profile. It fingerprints reused and new buffers,
retains original comparison status, and uses the original strict comparator.
Missing outputs cannot reuse stale control captures. A different native driver,
missing control marker or incomplete case matrix fails explicitly.

The [worktree control receipt](../porting/runtime-evidence/vulkan-raster-control-worktree-2026-10-10.json)
records **114/114 host controls passing**, including every original failure.
Forcing the same control on software introduces **30/114 depth failures**, despite
all original software captures passing. This countercheck prevents interpreting
the control as a universal replacement oracle. The command consequently returns
failure for the combined matrix; inspect profile results rather than treating it
as a general renderer acceptance command.

The [default-path regression](../porting/runtime-evidence/vulkan-raster-default-regression-2026-10-10.json)
compares all 480 new default-oracle captures against their original native buffers
byte for byte, including device identity. Invalid control arguments are rejected.
The ordinary acceptance command never requests the control. No tolerance, raw
failure, timing budget or production shader is changed.

Native bitwise parity and an accepted persistence timing baseline remain open.
This diagnosis removes the reason to keep changing camera matrices or depth
formats to chase these driver-dependent differences. Continue renderer capability
work while retaining both the original failures and the isolated precision evidence.
