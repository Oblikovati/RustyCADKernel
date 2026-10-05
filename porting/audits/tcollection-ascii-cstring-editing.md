# TCollection_AsciiString C-string copy and insert slice

The native `Copy(const char*)` wrapper sends null as a zero-length copy, clearing the string; a non-null pointer and char-array template both use `strlen`, truncating at the first NUL. The two C-string `operator=` overloads dispatch to the same Copy overloads. By contrast, pointer+length Copy keeps the complete byte span.

`Insert`, `InsertAfter`, and `InsertBefore` char-pointer wrappers skip the call entirely for null, so even an invalid index is not checked. For non-null values they use `strlen`; a non-null empty prefix still enters the core method, which validates the index before returning for length zero. The array forms also use `strlen`. Rust adapters model null as `None`, preserve these early returns, and delegate non-null prefixes to existing span operations; existing span methods remain unchanged.

Native differential fixture: `tools/oracle/ascii_string_c_string_editing.cxx`; expected output: `tools/oracle/ascii_string_c_string_editing.expected`. It compares char-array and pointer calls, full pointer+length Copy/Insert/InsertAfter/InsertBefore spans, null Copy and insertion, and exception classes for non-null empty values at invalid indices. The fixture inserts NUL through `Insert(int,char)` because `SetValue` rejects NUL. Compile/run used g++ 13.3: `g++ -std=c++17 -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/include/opencascade tools/oracle/ascii_string_c_string_editing.cxx -L/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/lin64/gcc/libi -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/lin64/gcc/libi -lTKernel`.

The ten ledger rows updated are Copy/operator= pointer and array overloads, and Insert/InsertAfter/InsertBefore pointer and array overloads. No declaration was added; all other Copy/insert overloads retain their prior status and targets.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

| Input | SHA-256 |
|---|---|
| `TCollection_AsciiString.hxx` | `5ce11a239588c7bcdfe62652b4152ea0f283207909c125772ed737e833fe6e3c` |
| `TCollection_AsciiString.lxx` | `47aa5de12a3e74380006b3d31d43bbbb41cc23bab8529f7c8cc99fc8f11d8188` |
| `TCollection_AsciiString.cxx` | `e31486afffb8c13e128fee2f903646f47573309ea17392b9ccb21c5f3e3b003a` |
| native `libTKernel.so` | `d63309c5c80b99ea9a95baf9ed549fa2dd417f159c5f0cbe598cb4a282bd995b` |
| probe source | `365d4326f686c77aac53b3386922fb162acf296be82577d381ba7abeef25f152` |
| expected output | `d883f56114c6dadd294f38b86b0e3cc7879377e197a66fc2a6afd1b413d1af6d` |

Rust verification: focused `ascii_string_api_behavior` passed 52/52; full `cargo test -p rustycad-tkernel` aggregated 2,782 passed, 0 failed, 0 ignored across 167 result groups (actual command exit 0). `cargo clippy -p rustycad-tkernel --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check` passed.

| Rust input | SHA-256 |
|---|---|
| `c_string_editing.rs` | `cc64847dbc24ce1035f16aa04a0155ebfea6a92d370efd850e92d989a45b467d` |
| `ascii_string.rs` | `dd6b4c0f376607119c9400ccff264470c1c238b3c2953b89be1bb4a8a13109f3` |
| Rust focused tests | `ed2417a9181ab2b545d8585e2f92278176d34ec10b3e8cf770bc79076b41d96f` |
| parity rows | `6c686708199070b268a072165555936363e590be812569de54f2089fece5978e` |

Compressed full Cargo output SHA-256: `f6b1c219355f23a4c30976604f49c90d8029f58f84fc490be7227101ffadb9e2` (`tcollection-ascii-cstring-editing-cargo-test.log.gz`).
