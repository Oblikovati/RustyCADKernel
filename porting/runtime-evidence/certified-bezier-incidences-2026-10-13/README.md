# Certified Bézier incidence evidence

Base source is `7e754661e`. This is a source-bound Martensite extension with no
upstream API mapping or whole blocker/family completion credit. See
`docs/certified-bezier-line-incidences.md` for the operation contract and gaps.

`denominator-capture.diff` is the separately reviewed structural change: retain
the existing checked empty denominator isolation rather than discard everything
except its bisection count. The old wrapper still returns the identical count;
conversion, denominator-root failures and callback order are unchanged. The
70-case existing Bézier/ray/winding/wire run and full toolkit run passed.
This receipt separates that extraction from new domain/world behavior.

`nearest-candidate-red.log.gz` records an actual failing 3e-17 request under the
prototype lower-corner/rectangle-width policy, not an old mapped-API regression.
The red-source digest identifies that temporary experiment. The green run uses
a generic midpoint candidate and exact endpoint-distance certificate; no tolerance
was widened. The independent adjacent 1/3 floating-point test also rejects 1e-20.

Native ordinary support/segment outputs were run twice and byte-compared. Their
build and header/library hashes are in `native-provenance.json`. Native controls
are ordinary supplementary parity; exact root and world error certificates are
independently tested in Rust. The raw exact polynomial oracle JSONs are attributed
in the persistent Rust fixture directory. No native library is a production or
Rust-regression dependency.

`full-toolkit.log.gz` predates only the Box-owned failure-evidence representation;
`boxed-evidence-tests.log.gz` verifies that change. Clippy and executable docs
passed. Failed intermediate degree-signature compiler attempts are not behavioral
reds and are not used as evidence. Benchmark verification emitted no timings;
measurement acceptance remains pending a frozen quiet-host run.

Final additional source-isolation run passed 20 focused toolkit cases; degree-8/16
rational repeated-root replay passed separately. The higher-degree first attempt
had a usize/u16 test assertion compilation error, corrected before the genuine
passing runtime; it is not numerical-red evidence. Single-crate nightly rustdoc
format61 confirms 76 public incidence items and 13 real parent reexports. Three
unrelated inherited documentation-link warnings remain in CSLib/ElSLib modules.
