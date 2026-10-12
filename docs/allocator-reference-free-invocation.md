# Martensite reference-free allocator invocation

OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` permits a derived allocator's
`Allocate` body to release its final intrusive owner and then finish using
previously captured external locals. The executable oracle
`martensite_clip_allocator_active_owner_loss.cxx` demonstrates destruction at
count zero before `allocate-return`. Its destructor changes the actual caller's
clip-plane argument, so the subsequently constructed sequence node must contain
the replacement plane. Neither the native body nor Martensite accesses the
retired receiver after that callback.

A borrowed Rust `&Self` pinned by a `HandleObserver` through the entire virtual
body delayed actual destruction. `AllocatorInvocation` instead copies immutable
virtual code and retains weak receiver identity. Each unsafe, callback-free
`capture` reads the actual receiver briefly, returning native local operands or
actual field references. No receiver borrow may escape, and mutable fields used
later by native code must be captured again after callbacks. Metadata accessors
must never invoke callbacks or release native resources. An expired or wrong
projection raises a typed failure containing the pointer and expected shape.

`AllocatorPolicyInvocation` retains only the shared physical allocation engine.
It adds no intrusive reference or scratch allocation. The normal policy façade
and invocation reuse the same allocation/free equations, including common
optimal allocation, heap rounding and arena state. A failed first implementation
allocated a boxed policy façade per call; all eleven native no-scratch rehash
regressions exposed that error and pass after the flat physical lease correction.

Run the Rust replay and consumer checks with:

```sh
cargo test -p rustycad-tkernel -p rustycad-tkservice --all-targets
```

The four-row active-owner-loss fixture is independent of rendering and replays
entirely in Rust. Native binaries are local-only tools. This prerequisite does
not complete allocator classes, clipping classes, unrestricted receiver
projection, or unrelated virtual destruction protocols. Existing partial API
claims remain partial until their complete contract and compiler bindings pass.
