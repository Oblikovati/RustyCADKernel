# Mat3/Mat4 vector products

Pinned OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871` evaluates
each result row as products joined by left-associative addition. The public
vector operators need only copyable elements with binary `*` and `+`; requiring
`MatrixScalar` incorrectly excluded valid element types.

The declarations are compiler-captured in
`porting/api-inventory/occt-declarations.jsonl.gz`:

| Owner | Compiler ID | Written parameter | Signature |
| --- | --- | --- | --- |
| `NCollection_Mat3::operator*` | `305bb133dd11aaa54fc1801b4aa52289e9484d4b724174a8219764586c5d2b34` | `const NCollection_Vec3<Element_t> &` | `NCollection_Vec3<type-parameter-0-0> (...) const noexcept` |
| `NCollection_Mat4::operator*` | `92efda460c7060fc03de3dd553826b026890916919c6d895b8c4f5de777f6567` | `const NCollection_Vec4<Element_t> &` | `NCollection_Vec4<type-parameter-0-0> (...) const noexcept` |

The normalized rustdoc paths are
`rustycad_tkernel::ncollection::Mat3::<core::ops::arith::Mul<rustycad_tkernel::ncollection::vector::Vec3<T>>>::mul`
and the corresponding `Mat4` / `Vec4` path. They appear in the published
reachable Rust inventory (`rust-api.json.gz`, SHA-256
`8819de64e814bddeaafbb9fc343fd25bf45b8ac725df500507f209e077b2e8fc`) and
were independently confirmed from fresh format-61 rustdoc JSON (SHA-256
`e1223e151746ad25bbe4df5c23be36a4e1495ab15defa684759676801d8fee43`).

The pinned headers have SHA-256 `a6e8f11773efab41789dedd45e55d05f556f420c7073f3fd01c96d0e079afa51`
(Mat3) and `1b2e16e62d4c520c3939ee6916a9607bd8d128d83850fbca2b52f0b8498e2df0`
(Mat4). Their implementations at Mat3 line 222 and Mat4 line 284 confirm
the required operations and term order.

The native probe is built with GCC 13.3 using:

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkernel_matrix_vector_products.cxx \
  -o /tmp/tkernel-matrix-vector-products-native
/tmp/tkernel-matrix-vector-products-native
```

Its four output rows are preserved in the adjacent `.expected` file. The
custom `BinaryOnly` element defines only construction, copy, `operator*`, and
`operator+`. Rust fixtures cover both Mat3 and Mat4 and intentionally lack
`PartialEq`, scalar
conversion, division, subtraction, and assignment arithmetic. The signed
integer cases cover the same Mat3/Mat4 result order with native `int` elements.
