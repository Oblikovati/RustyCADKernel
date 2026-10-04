# `NCollection_IncAllocator::SetThreadSafe` probe

Source revision: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Header SHA-256: `4fa6b2f70b28a1dfc9c4aa8e4b9a0120b60eb453220c36a390692c6432b7ab63`.
Probe SHA-256: `8a1bcf693f44fb479fa40eeec70cae055ecb51586d1e08fbf77ebbbb0afadf0a`.
Expected output SHA-256: `f755a111e8bb6cffe2db11d6ea9580a89e6cc23469d31b8113be7a3400a40ebf`.

The pinned `NCollection_IncAllocator.hxx` declares `SetThreadSafe(const bool = true)`. It enables or removes the allocator's optional shared mutex; the source warns that toggling it while allocations are concurrent is undefined. There is no distinct Rust incremental-allocator type: `BaseAllocator::incremental` constructs an `AllocatorKind::Incremental` stored behind a `Mutex`.

`tkernel_inc_allocator_thread_safe.cxx` enables native thread safety, runs four workers that each allocate 128 64-byte blocks, joins them, disables the native mutex after all workers stop, resets, and allocates again. Compile against the pinned install with:

```sh
g++ -std=c++17 -pthread \
  -I target/occt-modeling-full/include/opencascade \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  tools/oracle/tkernel_inc_allocator_thread_safe.cxx -lTKernel \
  -o /tmp/tkernel_inc_allocator_thread_safe
```

The native result, recorded in `tkernel_inc_allocator_thread_safe.expected`, was `512 512 1`: all requested allocations had distinct addresses, and post-reset allocation succeeded. The Rust integration test performs the same workload and toggles the request only after joining workers. A separate Rust test checks that common, heap, accumulating, and aligned allocators each return a type-mismatch error naming the actual and expected allocator kinds.

The focused Rust command `cargo test -p rustycad-tkernel --test ncollection_allocators_api_behavior` passed 14/14 tests.

Rust exposes `BaseAllocator::set_thread_safe(bool)` only for its incremental allocator variant. Its `Mutex` remains active for both boolean values to preserve Rust memory safety. The method makes the native request callable and preserves the defined allocation/reset outcomes, but does not reproduce native mutex creation/removal or performance; no lock-toggle parity is claimed. Calling it on another Rust allocator kind returns `Standard_TypeMismatch`, since those native allocator classes do not declare the method.
