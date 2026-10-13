# Mesa 25.2.8 CPU override semantics

Inspected upstream tag `mesa-25.2.8`, commit `f97d7dcca0e51d7e20ed4342bb5ea8afea7db7fa`.
The GitLab tag API receipt and fetched source SHA256s are beside this note;
the four critical source files were fetched again at the immutable commit and
matched the tag bytes. No build, graphics probe, or production change was made.
Ubuntu package patches and actual failing-runner machine code were not audited.

`GALLIUM_OVERRIDE_CPU_CAPS=sse2` is not merely a width hint on x86. It clears
`has_sse3`; the following dependency cascade clears SSSE3, SSE4.1/4.2, AVX,
F16C, FMA, AVX2 and AVX512 flags. Maximum vector width is then recalculated
from the overridden flags (128 without AVX). This applies to Mesa's CPU-cap
record, not the instruction selection of every prebuilt process/library.
[CPU detection/override, lines603–690](https://gitlab.freedesktop.org/mesa/mesa/-/blob/f97d7dcca0e51d7e20ed4342bb5ea8afea7db7fa/src/util/u_cpu_detect.c#L603)

Gallivm's x86 feature list explicitly feeds the overridden flags to LLVM,
including `-avx`, `-f16c`, `-fma`, `-avx2` and the listed AVX512 attributes.
Do not conclude from a retained host CPU name alone that FMA remains enabled.
[Target feature construction, lines342–380](https://gitlab.freedesktop.org/mesa/mesa/-/blob/f97d7dcca0e51d7e20ed4342bb5ea8afea7db7fa/src/gallium/auxiliary/gallivm/lp_bld_misc.cpp#L342)

`LP_NATIVE_VECTOR_WIDTH=128` directly overwrites `lp_native_vector_width` after
the default is selected from CPU caps. That function neither clears CPU
features nor chooses a generic LLVM CPU.
[Width initialization, lines67–79](https://gitlab.freedesktop.org/mesa/mesa/-/blob/f97d7dcca0e51d7e20ed4342bb5ea8afea7db7fa/src/gallium/auxiliary/gallivm/lp_bld_init_common.c#L67)

Both JIT implementations retain host-specific target selection: MCJIT sets
feature attributes, obtains `llvm::sys::getHostCPUName()`, then calls
`builder.setMCPU(MCPU)`; ORC similarly adds attributes then sets the host CPU.
The override also leaves other detected fields, such as `has_popcnt`, unchanged.
Therefore these two environment variables do not establish a single generic
CPU target, identical host scheduling/cost models, or uniform Rust/libm behavior.
[MCJIT, lines521–584](https://gitlab.freedesktop.org/mesa/mesa/-/blob/f97d7dcca0e51d7e20ed4342bb5ea8afea7db7fa/src/gallium/auxiliary/gallivm/lp_bld_misc.cpp#L521),
[ORC, lines436–525](https://gitlab.freedesktop.org/mesa/mesa/-/blob/f97d7dcca0e51d7e20ed4342bb5ea8afea7db7fa/src/gallium/auxiliary/gallivm/lp_bld_init_orc.cpp#L436),
[POPCNT detection, line821](https://gitlab.freedesktop.org/mesa/mesa/-/blob/f97d7dcca0e51d7e20ed4342bb5ea8afea7db7fa/src/util/u_cpu_detect.c#L821).

Conclusion: AVX/FMA are explicitly disabled in the audited Mesa capability and
LLVM-feature paths; vector width is constrained, but complete CPU-independent
code generation is not established. This source finding does **not** explain
the three release pixel failures or justify a driver workaround, golden change,
tolerance change, or claim that a fresh green run fixed their cause.
