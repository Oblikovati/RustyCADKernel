# Windows locale conversion adapter

`NCollection_UtfString.cxx` at OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871` selects Windows system ACP conversion: `MultiByteToWideChar(CP_ACP, MB_PRECOMPOSED, ...)` and `WideCharToMultiByte(CP_ACP, 0, ...)`. The Rust adapter in `crates/tkernel/src/ncollection/utf_string/locale_windows.rs` uses those exact APIs behind a project-owned `WindowsLocaleCalls` seam. It preserves the two-call sizing pattern and the extra terminator allocated by native `FromLocale`.

The injectable Latin-1 fake tests cover NUL termination, native extra terminator storage, round-trip bytes, and a short output buffer (4 locale unit tests, 2 doctests). The adapter also cross-compiles with `cargo check -p rustycad-tkernel --target x86_64-pc-windows-gnu --lib` and passes strict library Clippy on both Linux and the Windows GNU target. This is compile and fake-call evidence; no Windows OCCT runtime is available here, so all four locale API mappings remain partial.

Pinned upstream source hashes:

- `NCollection_UtfString.cxx`: `a2b5535f15ce8153e8d2f8ce1e712011a8a5a8e9b1a8931e5d5e7056871b4112`
- `NCollection_UtfString.hxx`: `9012a0843c03f1afb7db2c8c57c218954c3a2ca5cd2e357c2fb65f11bd9c10eb`

The Windows adapter source hash is `5533ee0a13085cbd304842b624b27cf81e90218d738e1905660a362ff93e094f`; `windows-sys` 0.61.2 is already pinned by the TKernel crate.

Reproduce fake codec tests on the host:

```sh
PATH=/home/vmiguel/.cargo/bin:$PATH cargo test -p rustycad-tkernel --lib ncollection::utf_string::locale
PATH=/home/vmiguel/.cargo/bin:$PATH cargo check -p rustycad-tkernel --target x86_64-pc-windows-gnu --lib
PATH=/home/vmiguel/.cargo/bin:$PATH cargo clippy -p rustycad-tkernel --target x86_64-pc-windows-gnu --lib -- -D warnings
```
