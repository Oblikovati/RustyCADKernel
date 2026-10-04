# `Standard_MMgrOpt::FreePools()` protected-hook oracle

`FreePools()` releases the linked small-block pools without touching separately
allocated medium blocks. The fixture exposes the protected member through a derived
class, allocates one small and one medium block, releases the pools, checks the medium
payload, then frees the medium block. It intentionally does not dereference or free the
small pointer after release.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Header SHA-256: `9ca7d6b876c018ff937746df766eac2568ab1dbafd8a27b2ba745adfea032a22`.
Expected output is in `tkernel_mmgr_free_pools.expected`.

```sh
occt_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation
kernel="$occt_build/lin64/gcc/libi/libTKernel.so"
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_mmgr_free_pools.cxx "$kernel" \
  -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-mmgr-free-pools
/tmp/tkernel-mmgr-free-pools
```

Native source calls `FreePools()` from the destructor after `Purge`; its header warns
that freeing pools while storage from them remains in use is dangerous. Rust exposes
the operation with a safe boundary: pool-backed `StorageAddress` values are invalidated,
pool nodes are removed from free lists, and the cursor is reset so the manager can be
used again. Those post-call safety transitions are Rust guarantees, not claims about
native follow-up behavior. Medium and large allocations remain owned and usable.
