# Independent Martensite visualization performance audits

The coordinator independently verified source manifests and executable hashes
before and after the two agent performance batches on 2026-10-12. Both profiles
completed successfully with validation disabled: 8 workloads, 20 warmup frames,
and 100 retained samples each, totaling 3,200 raw samples across the two families.
The software device was llvmpipe LLVM 20.1.2 with default 256-bit width; the native
device was AMD Radeon 8060S Graphics (RADV STRIX_HALO).

| Family | Frozen source | Lavapipe peak RSS (KiB) | Native peak RSS (KiB) |
| --- | --- | ---: | ---: |
| Colored shape | `7cf634c331559ff2f61a903b0ed5e892bd6f79c7` | 110392 | 73384 |
| Clipping transport | `e4e51d28ec5cf536a8fa609b6ef41d1564e7ccaa` | 100292 | 63836 |

Peak RSS measures the process, not GPU allocation. GPU memory was not measured.
These measurements are evidence, not accepted performance baselines or API
completion claims. Colored-shape timings cover steady redraw/readback after
setup, including a state updated once before timing; they do not measure Compute
or Update creation cost. Clipping includes five retained states and three actual
per-frame mutation workloads. Raw receipts remain with the respective family
runtime evidence. Source and binary hashes, independent bookend checks, and
receipt artifact hashes are in the adjacent audit files. The clipping audit also
retains the original process-memory output and per-profile receipt.
