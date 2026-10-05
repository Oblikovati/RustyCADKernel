# NCollection Mat3/Mat4 elementwise operators

OCCT `Add` and `Subtract` use the element assignment operators. `Added` and
`Subtracted` copy the matrix and then call those methods; binary matrix
operators delegate to the copied forms. Rust now follows that call chain and
requires `Copy + AddAssign` or `Copy + SubAssign` for just this functionality.
The matrix product and other scalar algorithms retain their existing
`MatrixScalar` constraints. Iteration remains column-major, matching
`myMat[0..N*N]`.

The regression element implements only `AddAssign` and `SubAssign`; it has no
binary `Add` or `Sub`. Both the Rust and native fixture therefore verify that
the elementwise family does not rely on the unrelated binary operators. The
probe checks `Add`, `Subtract`, `Added`, `Subtracted`, `operator+=`, and
`operator-=` for both dimensions against
`tkernel_matrix_element_assignment.expected`.

## Compiler-selected declarations

These declaration identities are from
`porting/api-inventory/occt-declarations.jsonl.gz` in the current source-bound
capture. `operator-` entries below are the binary overloads taking a matrix;
the separate unary overload is not part of this slice.

| Native member | Compiler declaration ID | Rust target |
|---|---|---|
| Mat3::Add | `ae7844c8d1988eb2ede85bda79dbed4e54ddfb19d414a7fb0e66336c37d56eaa` | `rustycad_tkernel::ncollection::Mat3::add` |
| Mat3::operator+= | `31efdf711633072fb214aa416a3ec49e9ca32e8cc00b63a91cbfb860a0eef499` | `rustycad_tkernel::ncollection::Mat3::<core::ops::arith::AddAssign>::add_assign` |
| Mat3::Subtract | `8c8fd3187b88f9896578e8b3058d949669f77833d84109fcbd291dc610333bf0` | `rustycad_tkernel::ncollection::Mat3::subtract` |
| Mat3::operator-= | `81c5ebe34d460cba1023299b164631c0f8614ffd42b8aebe302600b5da98dbc9` | `rustycad_tkernel::ncollection::Mat3::<core::ops::arith::SubAssign>::sub_assign` |
| Mat3::Added | `7937c9575c2a259e3a9cc12861f8974127300140bad708343beb59f832ec4204` | `rustycad_tkernel::ncollection::Mat3::added` |
| Mat3::operator+ | `e57fc296fdd957a69d739a3fe69cbb8ebd7ad391fc9595375344c741bcf85423` | `rustycad_tkernel::ncollection::Mat3::<core::ops::arith::Add>::add` |
| Mat3::Subtracted | `746d16826c5db7d53ca274d7d97cb13d79794eadb90b00a9c2eb02658d2cd11f` | `rustycad_tkernel::ncollection::Mat3::subtracted` |
| Mat3::operator- (binary) | `10e2e596af19504b6abf6ffc4c23931c4a965fac1b664b37c2bdb7dee888d9b0` | `rustycad_tkernel::ncollection::Mat3::<core::ops::arith::Sub>::sub` |
| Mat4::Add | `1cfdf792459a42becbc27a478188aff2500a2790c8f12b0289053f557436128c` | `rustycad_tkernel::ncollection::Mat4::add` |
| Mat4::operator+= | `43f49b6c346fffcd399641d28acd1d83bafaa24fa71c817e7533086bf03e9e5b` | `rustycad_tkernel::ncollection::Mat4::<core::ops::arith::AddAssign>::add_assign` |
| Mat4::Subtract | `7d004d9a1beb257002bd5b85198dcab926f72db60f3f3572fcf857e6b974cd4e` | `rustycad_tkernel::ncollection::Mat4::subtract` |
| Mat4::operator-= | `a384cd65fbef8d3b09fee8ef34271749efe5d0d5fdc75e0d5d3e81cc1a626b5b` | `rustycad_tkernel::ncollection::Mat4::<core::ops::arith::SubAssign>::sub_assign` |
| Mat4::Added | `ea65066a23af417b474b58c57472faad8524978cea1c965f6b64f1503850b297` | `rustycad_tkernel::ncollection::Mat4::added` |
| Mat4::operator+ | `b7315099d50050e5cb46ba48f4b1727b522505e13287e1f1172a4bac8167a279` | `rustycad_tkernel::ncollection::Mat4::<core::ops::arith::Add>::add` |
| Mat4::Subtracted | `0c47f613bc3f82681d3312838c3eaaf04f93ae7ef0459d88f92f31660c87c51b` | `rustycad_tkernel::ncollection::Mat4::subtracted` |
| Mat4::operator- (binary) | `08796bbd20ff656ab7aa7e21a239f9f5d1ada9853bae2a8896ac82a8402121b6` | `rustycad_tkernel::ncollection::Mat4::<core::ops::arith::Sub>::sub` |

## Native replay

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Header SHA-256 values: Mat3
`a6e8f11773efab41789dedd45e55d05f556f420c7073f3fd01c96d0e079afa51`, Mat4
`1b2e16e62d4c520c3939ee6916a9607bd8d128d83850fbca2b52f0b8498e2df0`.

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  -I/home/vmiguel/git/oblikovati-workspace/OCCT/src/FoundationClasses/TKernel/NCollection \
  -I/home/vmiguel/git/oblikovati-workspace/OCCT/src/FoundationClasses/TKernel/Standard \
  tools/oracle/tkernel_matrix_element_assignment.cxx \
  -o /tmp/tkernel_matrix_element_assignment
/tmp/tkernel_matrix_element_assignment > /tmp/tkernel_matrix_element_assignment.out
diff -u tools/oracle/tkernel_matrix_element_assignment.expected \
  /tmp/tkernel_matrix_element_assignment.out
```

The native source emits the expected results using the custom assignment-only
element. This is a compile-time and value-order check, not a claim that OCCT
supports arbitrary non-arithmetic element types in every Mat3/Mat4 member.

## Validation receipt

Rust base revision: `332c3dfebff188cfef8830ba60b7a71b78be128e`. Full TKernel
tests passed 2,784 cases across 169 result groups, with zero failures or
ignored tests; strict all-target Clippy passed. Rust compiler: 1.98.1
(`48a229ceaefd4985c50990b14116b6d856af0985`). Native compiler: g++ 13.3.0.
The exact source/header/oracle hashes are recorded below; complete Rust and
native outputs are archived at
`porting/runtime-evidence/tkmath-matrix-element-assignment.tar.gz` (SHA-256
`91dfe469a5e3ef18ee69276b720ab1cd409c4152ecb45f06f8c1b74a2dc0ed1a`).

Source hashes:

- `NCollection_Mat3.hxx`: `a6e8f11773efab41789dedd45e55d05f556f420c7073f3fd01c96d0e079afa51`
- `NCollection_Mat4.hxx`: `1b2e16e62d4c520c3939ee6916a9607bd8d128d83850fbca2b52f0b8498e2df0`
- Rust `square_matrix.rs`: `7830d8dad6cff316f740f3068a4551ee48a940dc552c36bbd02d8e7206aeb959`
- Rust fixture: `ff6b54893181dd540bde3b4ff40d2ed9fc34126a2369b6d010c4bb7de1055a70`
- Native probe: `0bc0b4da3754fc45ba396151a8a597239849cfd08cb641272ee97bc8286a35ae`
- Native expected output: `4cc454430d54bcf3970a8e41bf3d182dae7183f9b132f11764637c3b5ff766b5`
- Compiler declaration capture: `4bc75a502ac655a45bacbc426fb5159028d3fb88deb4829135aca2466cd89095`
