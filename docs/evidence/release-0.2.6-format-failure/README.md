# Unpublished v0.2.6 formatting failure

The complete local extracted gate passed23,183 tests with111 existing ignores.
The hosted exact formatter found two included test files that cargo fmt did not
visit. Root reproduced the failure, formatted both files, and added the exact
CI formatter to package preparation. A named fake regression first failed, then
passed, proving the formatter error prevents Cargo packaging. Production Rust
and shader files did not change. The immutable failed tag remains unchanged;
v0.2.7 is the corrected release candidate.
