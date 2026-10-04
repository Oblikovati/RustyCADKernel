# TKMath deferred triangulation filesystem forwarding

Pinned OCCT source: `3d097a0328e71b826377d4814ab05ec3c3d23871`,
`Poly_Triangulation.cxx` LoadDeferredData/DetachedLoadDeferredData.

The Rust deferred interface predated the TKernel OSD_FileSystem port and omitted
its argument. The new overloads forward `Option<&FileSystemHandle>` unchanged to
the injected storage, including `None` for native default/null handles. Existing
no-argument methods select that null default. Storage implementations that own
their input retain their existing loader through the default forwarding hook.

The native subclass probe captures filesystem identity, destination identity,
call count, loaded-purpose flags, detached object creation, failure handling and
the no-data early return. It performs no filesystem I/O. Compile the adjacent
C++ file with C++17, `-O0 -g0`, the pinned native include directory, TKMath/TKernel
libraries and native library rpath. The Rust integration test replays its stdout
from the adjacent expected file. A separate Rust test checks the base virtual
loader's native false result; existing deferred-storage tests exercise backwards
compatibility with the earlier injected loader interface.

This closes filesystem forwarding in the base TKMath interface; format-specific
deferred decoders remain separate upstream-toolkit work. No legacy declaration is
newly mapped by this supplemental probe.
