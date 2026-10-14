# Martensite source-derived pair/self prototype

This checkpoint is private and test-gated. It is not the public complete pair
provider, an API mapping, or issue10/application acceptance. The original full
provider task remains open.

Actual captured Bernstein controls/weights become jointly normalized homogeneous
Nx, Ny, D. Both cross-multiplied coordinate equations retain raw coefficient
magnitude. Same-source equations divide exactly by t-u. Generic polynomial
pseudo-remainders in u produce a linear ideal row where available and retain
every divisor leading factor. Both ORIGINAL equations are substituted into that
row. Root-free alpha on the full first trim certifies a unique graph u=-beta/alpha;
all candidate t roots are isolated, the second original equation is checked by
Sturm-Tarski, and exact graph signs enforce the second trim and strict t<u.
Constructed u bounds use exact rational interval Horner and directed rounding;
world-point/error envelopes reuse the checked general line constructor.

Independent exact cases:
- Parabolas (t,t²), (u,1/2-u²): t=u, 4t²-1=0, bounded root1/2,
  world(1/2,1/4); Jacobian determinant is nonzero.
- Crossing cubic x=3-12t+12t², y=-9+66t-144t²+96t³:
  quotient x gives u=1-t; 16t²-16t+3 roots1/4,3/4; t<u picks1/4.
- Cusp y=-3+18t-36t²+24t³ uses same x and graph; 4t²-4t+1
  implies t=u=1/2, rejected by exact strict off-diagonal sign.
- Initial closed cubic x=12t-24t²+12t³, y=12t²-12t³:
  quotient rows both degree2 in u. Their generic leading row elimination yields
  u=1-t. Original substitutions have opposite primitive ±t(t-1).
  Independent Jacobian isolation confirms roots0,1; strict t<u leaves(0,1).

Local OCCT ordinary outputs were captured twice byte-identically, with pinned
headers/libraries/compiler/binary hashes. Rust-only replay keeps these assets
inside the crate. Native approximate point/parameter comparisons are ordinary
observations with explicit1e-12 observation tolerance; they never establish root
completeness or a world-error certificate. Exact fixture assertions do that.

Current checks:49 library tests pass, including21 pair tests and28 prior/shared
arithmetic tests; all-target/all-feature Clippy-Dwarnings and formatter pass.
Focused style audit reports no function over20lines and all files are under500.
No timing or performance acceptance was run.

Remaining: algebraic exceptional fibers, non-linear fibers, complete overlap
correspondence, explicit public policy/report/source binding, retained proof/work
accounting (including failed/cancelled subdivisions), refinement/resolution
receipts, extracted-package/whole-toolkit acceptance, census/consumer integration.
These remain typed incomplete paths, not completed-empty results. Generic raw
convolution and world extraction were committed separately from this behavior.
