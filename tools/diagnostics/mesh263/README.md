# Cylinder mesh behavior-version diagnostic

The product checkout is fixed at `ab4bad4fce774001b192fd20ac4d7a9010e68fea`. The input is a cylinder of radius 5 and height 10, meshed at linear deflection 0.5 with the original default parameters. The two original failing test files and all their assertions remain unchanged.

The portable expected record contains all coordinates, UV parameters and triangle indices for three faces: 54 nodes/52 triangles, then 26/24 and 26/24. It was captured twice from the independently built native producer, with identical 10,289-byte output. Its existing three hashes already match the product's common fixture. `reference-provenance.json` binds the producer source, executable, loaded modeling libraries and build manifests. Rust output supplies no expected value.

`certify_reference.py` independently checks all 108 side-face x/y coordinates with directed MPFR sine/cosine bounds at 256 and 512 bits, followed by the documented binary64 radius-five product. Both directed endpoints round to the same primitive word. The periodic seam uses the copied endpoint at the stored full-turn parameter. This certifies the stated primitive composition; topology, cap UV values and triangle ordering remain independently captured reference fields.

The new workflow mode runs on native macOS ARM64 and Windows x64. Each runner must reproduce both original failures, match every expected record byte in two processes, apply the SHA-checked test-selector patch, and pass the same two tests. The fixed phase repeats the complete record comparison twice. A changed coordinate cannot pass merely by retaining matching hash rows. Five local guard tests cover verdict counts, ignored/zero-test rejection and that full-record requirement.

The patch selects the common fixture for the already approved portable math behavior version 2. It changes no production code, fixture value, assertion or tolerance. The older platform-specific files remain intact as host-libm reference records. Native diagnostic results are still required before admitting the correction; local Linux matches are not ARM64 or Windows acceptance.

The Windows-only retry preserves the first failed setup. Before any test runs, it records the exact dirty paths and diff summaries, restores modified regular tracked files from the frozen Git objects, and records their before/after hashes. Untracked paths are preserved and the original strict clean-checkout and source-hash checks still apply. The nine guard tests include untracked-file preservation, revision rejection and regular-file admission.
