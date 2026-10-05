# LocalArray replacement-allocation exception state

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`NCollection_LocalArray::Reallocate` at lines 91–160 constructs a replacement
before publishing `myPtr`/`mySize`. This derived probe seeds one live inline
item directly, then throws from every default construction attempted by a
no-copy growth replacement. After catching the exception, the original object
still reports size 1 and value 42.

OCCT does not clean up the raw partially constructed replacement buffer on
this exceptional path. Rust's temporary `SmallVec` will clean up its partial
replacement; the planned parity claim is limited to the retained original
array state, not native leak behavior.

Header SHA-256:
`516ac5faff95f20fa5638c2cf8819c0730f6940289316a6a662fbbb6981f141f`.
Test source SHA-256:
`d24eb586570d9f970f864f655ec071b07351a66517059b86d0fcaddf1100a864`.

Build and replay with the repository's pinned Linux OCCT installation:

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkernel_local_array_reallocate_exception.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKernel -o /tmp/tkernel-local-array-reallocate-exception
/tmp/tkernel-local-array-reallocate-exception
cmp tools/oracle/tkernel_local_array_reallocate_exception.expected <( /tmp/tkernel-local-array-reallocate-exception )
```
