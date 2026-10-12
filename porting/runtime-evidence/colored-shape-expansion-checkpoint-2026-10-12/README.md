# Martensite ColoredShape expansion prerequisite checkpoint

This unfinished checkpoint implements native protected topology expansion and visibility through the actual owned map. It does not claim AIS_ColoredShape completion, appearance, copy, Compute, selection, or rendering. Current-source compiler claim validation remains pending.

The pinned OCCT native probes use actual MakeEdge/MakeFace topology and the real AIS_ColoredShape object. Each committed corpus was captured twice with identical output. Five alias rows prove a retained Key/Value reference, unchanged addresses and ownership counts, later-bucket insertion visibility, and source/output map identity. Fifteen topology rows prove wire/shell/solid expansion, compsolid direct binding, existing-leaf preservation, explicit-leaf replacement, external compound routing, owning-compound inclusion, append semantics, orientation-sensitive base exclusion, and hidden-state visibility.

The cursor follows actual allocated nodes and later live bucket heads without copying keys or values. Native BasicIterator caches its bucket/end pointers: rehash invalidates it. Martensite's checked invalidation tests are safety extensions for invalid native lifetimes, not native parity claims. Node keys and assignable handle slots are borrowed independently so same-map expansion does not hold a value read guard while updating it.

The pre-existing 47-row ColoredShape owner/dispatch/destructor corpus and map ownership/alias tests remain required. Erased aliases release their owned TShape/drawer fields immediately; metadata leases do not delay native ownership release. Existing scalar extent and native destruction ordering remain unchanged.

Both native executables use `c++ -std=c++17 -O2`, the checked pinned OCCT headers and existing `target/occt-all-full/lin64/gcc/libi`, linking TKV3d, TKTopAlgo, TKBRep, TKGeomBase, TKG3d, TKService, TKMath and TKernel. The expansion source also uses `-Itools/oracle`. Full frozen source hashes and executable hashes are recorded in native-provenance.json; final source/header/library manifest and scoped API receipts will accompany the completed feature.

Next adopt the checked AIS_Shape appearance dependency from main, then implement genuinely required appearance/copy behavior. Class and rendering status stay partial.
