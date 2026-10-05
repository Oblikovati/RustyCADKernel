# TCollection_AsciiString C-string search slice

The pinned OCCT `Search(const char*)` and `SearchFromEnd(const char*)` wrappers return `-1` for null and delegate non-null values through `strlen`; char-array templates use the same NUL-terminated prefix. Pointer+length calls use the full supplied span. `FirstLocationInSet` and `FirstLocationNotInSet` have pointer+length, `AsciiString`, `string_view`, and array-template forms; the array template passes `N-1` bytes, retaining interior NULs. There is no pointer-only set overload.

Rust now exposes `search_c_string` and `search_from_end_c_string` over `Option<&[u8]>` (None models null), and `first_location_in_set_literal` / `first_location_not_in_set_literal` over fixed-size arrays. Existing generic methods remain full-span methods. This adds no legacy declarations; only the existing C-string and array rows in `porting/parity.json` target the new adapters.

Native differential fixture: `tools/oracle/ascii_string_c_string_search.cxx`; expected output: `tools/oracle/ascii_string_c_string_search.expected`. It was compiled with g++ 13.3 against `target/occt-foundation` and produced the committed expected output. The fixture inserts NUL through `Insert(int,char)` because the native length-taking constructor scans to NUL and `SetValue` rejects NUL. It also compares null, zero-length set calls before invalid-range validation. Compile/run used g++ 13.3: `g++ -std=c++17 -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/include/opencascade tools/oracle/ascii_string_c_string_search.cxx -L/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/lin64/gcc/libi -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/lin64/gcc/libi -lTKernel`.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

| Input | SHA-256 |
|---|---|
| `TCollection_AsciiString.hxx` | `5ce11a239588c7bcdfe62652b4152ea0f283207909c125772ed737e833fe6e3c` |
| `TCollection_AsciiString.lxx` | `47aa5de12a3e74380006b3d31d43bbbb41cc23bab8529f7c8cc99fc8f11d8188` |
| `TCollection_AsciiString.cxx` | `e31486afffb8c13e128fee2f903646f47573309ea17392b9ccb21c5f3e3b003a` |
| native `libTKernel.so` | `d63309c5c80b99ea9a95baf9ed549fa2dd417f159c5f0cbe598cb4a282bd995b` |
| probe source | `8407daedc5981eb7db25ff823dc585aeb1799647d74a3c9d897884199a1b7522` |
| expected output | `5d0e882353b2c2f9e5fcd14fa73e68e8d85d4096136892e16adb469be47a2651` |

Rust verification: focused `ascii_string_api_behavior` passed 46/46; full `cargo test -p rustycad-tkernel` aggregated 2,773 passed, 0 failed, 0 ignored across 166 result groups (actual command exit 0). `cargo clippy -p rustycad-tkernel --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check` passed.

Compressed full Cargo test output SHA-256: `95270ed48f686f26a0fc5ac1db33cfedfff8e485ba66fcfa1da5344ea7906612` (`tcollection-ascii-cstring-search-cargo-test.log.gz`).
