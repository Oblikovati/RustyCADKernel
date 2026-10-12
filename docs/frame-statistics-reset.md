# Martensite frame-statistics reset

OCCT `Graphic3d_FrameStatsData::Reset` fills existing counter/timer vectors;
it does not replace their backing storage. Martensite previously assigned a
new default frame, changing all four storage addresses. The correction fills
existing vectors and restores the four frame rates to -1.

Three repeatable native rows cover base reset, temporary-frame reset and repeated
reset. They compare actual protected vector addresses and every vector/rate value,
including resetting NaN/infinity seeds and producing positive zero. Rust replays
the committed asset and checks actual public mutable counter/timer field addresses.
The original two failing regressions are retained in the evidence directory.

Run `cargo test -p rustycad-tkservice --all-targets`. Native OCCT is used only
when regenerating the oracle fixture locally. This correction does not claim
whole statistics-class completion, assignment semantics, general live-reference
transport or measured GPU performance.
