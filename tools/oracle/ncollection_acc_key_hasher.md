# AccAllocator protected Key/Hasher probe

The probe derives from `NCollection_AccAllocator`, exposes its protected `Key` and `Hasher`
types, then invokes both native hasher call operators. OCCT source revision is
`3d097a0328e71b826377d4814ab05ec3c3d23871`; `NCollection_AccAllocator.hxx` SHA-256 is
`5fe1b00032b8b1493a18d1a8c778410f929cb25a010757c15028d02fa490c585`. The linked
`libTKernel.so` SHA-256 is
`ddf94242442e02e57820d048b5296ee5e73ec579ae0d0fd8bddf26c7f8e1aefa`.

Native `Hasher(Key)` returns `Key::Value` unchanged. Its two-key overload compares those values.
The fixture checks zero, ordinary values, and `SIZE_MAX`; the final distinct-key comparison uses
defined unsigned wraparound. Rust exposes equivalent `AccBlockKey` and `AccBlockKeyHasher` value
types with explicit hash/equality methods. They do not provide allocator subclassing, access to
the native block map, or the protected `getKey` implementation.

Reproduce from the repository root with:

```sh
c++ -std=c++17 -O2 -Itarget/occt-tkernel-full/include/opencascade \
  tools/oracle/ncollection_acc_key_hasher.cxx \
  -Ltarget/occt-tkernel-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-tkernel-full/lin64/gcc/libi -lTKernel \
  -o /tmp/ncollection_acc_key_hasher
/tmp/ncollection_acc_key_hasher
```
