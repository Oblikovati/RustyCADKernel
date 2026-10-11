# Martensite native platform fonts

Windows CI's `every_flipping_mode_reaches_retained_draws` failed for `flip-text`
([run 38112822728](https://github.com/Oblikovati/RustyCADKernel/actions/runs/38112822728/job/114391747842),
Windows shard 4). The old runtime library adapter returned
missing for all non-Unix libraries, and font discovery scanned X11 directories
on every platform. No FreeType prerequisite was installed for Windows CI.

Martensite now loads Windows DLLs with `LoadLibraryW` and resolves their exports
with `GetProcAddress`, retaining the same process-lifetime ownership as Unix.
The FreeType adapter accepts native MSVC and MinGW DLL names. Font discovery
preserves pinned OCCT's Windows registry enumeration order, supported suffixes,
relative `Windows\\Fonts` paths and first-face checks. A named fake registry covers
pure path resolution. The FFI adapter owns and closes its registry key separately.
Failed fixed-buffer reads cannot expose stale path bytes.

macOS scans `/System/Library/Fonts` and `/Library/Fonts` in native directory-map
order, without X11 fontconfig queries or `fonts.dir` parsing. Linux retains its
fontconfig/configuration-file/directory behavior and XLFD handling.

Windows CI builds pinned external FreeType 2.13.3
(`42608f77f20749dd6ddc9e0536788eaad70ea4b5`) with MSVC and adds its DLL directory
to the test environment. External optional codecs are disabled; FreeType's bundled
inflate still decodes the embedded WOFF. The port does not link OCCT to supply fonts.

The required runtime regressions never skip a missing library or an empty native
Windows/macOS font catalog:

```sh
cargo test -p rustycad-tkservice --test font_platform_runtime --locked
cargo test -p martensite-vulkan --test group_flipping --locked
```

A Windows GNU build running under Wine reproduces the missing-library failure
before the fix and passes both embedded-glyph and registry-font checks afterwards.
All four retained group-flipping regressions pass there, including `flip-text`.
The host TKService suite passes 237 tests; macOS compilation and host/Windows
all-target Clippy pass. Wine provides executable Windows API evidence; native
MSVC CI and a native macOS font scan still require their actual runners. These
checks do not establish complete platform rendering or configuration parity.
