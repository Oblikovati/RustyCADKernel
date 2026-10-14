# TKBool progress

TKBool (`crates/tkbool`) ports the OCCT topological operation toolkit at
3d097a0328e71b826377d4814ab05ec3c3d23871: BRepAlgo, BRepFill, BRepProj, TopOpeBRepTool,
TopOpeBRepDS, TopOpeBRepBuild and TopOpeBRep. This file records the TopOpeBRep* packages
ported after the BRepFill/BRepAlgo work, their mapping rules and the native quirks kept.

## TopOpeBRepTool

Every public class and function of the package is ported (the debug-only `STATE`,
`trace*` and `faulty` files print traces and are not part of the API).

- `TopOpeBRepTool_TOOL` is the namespace struct `TopOpeBRepToolTOOL` with associated
  functions; the members ported before it (`Nt`, `NggeomF`, `minDUV`, `TggeomE`,
  `UVISO`) stay free functions.
- The `TOPOLOGY.hxx` functions (and `FUN_tool_getgeomxx/getxx/getdxx` of `matter.cxx`)
  are free functions; overloads take distinct names (`fun_tool_value_on_edge`,
  `fun_tool_value_on_face`, ...).
- Output references become returned values; an output the native code writes before a
  failure stays a `&mut` argument (`ParISO`, `Getstp3dF`, `uvApp`, `FUN_tool_parE`,
  `FUN_tool_parF`, `FUN_nearestISO`, `TgINSIDE`'s orientation, `mkTondgE::MkTonE`).
- The package functions (`Regularize*`, `PurgeClosingEdges`, `CorrectONUVISO`,
  `MakeFaces`, `Print`) are free functions of the module.
- `TopOpeBRepTool_FuseEdges` is an older copy of `BRepLib_FuseEdges`; it runs the
  TKTopAlgo port in its TopOpeBRepTool variant (no B-spline concatenation, no closed
  B-spline edge, the edge range always moved onto the old p-curves).

### Native quirks kept

- `FUN_tool_bounds(F, u1, u2, v1, v2)` reads its box with `Get(u1, v1, u2, v1)`: `v1`
  ends as the maximal V and `v2` is never written.
- `FUN_tool_closedS(F, inU, xmin, xper)` returns false in every case.
- `FUN_tool_comparebndkole` never returns 1 (its branch needs six equal bounds, which
  return 0 first).
- `TopOpeBRepTool_CLASSI`'s `FUN_thegreatest` reads the state of the point already
  classified (the classifier is not run on its new point, whose parameter is
  `(1 - x) f + x + l`).
- `FUN_tool_ClassifW` moves only the lower wires of a smaller wire into the greater one
  (the smaller wire itself is dropped); a wire already done and unbound would make the
  native loop endless (a `Standard_ProgramError` here).
- `TopOpeBRepTool_REGUW::InitBlock` tests the closing edge on the null `myed`, so the
  first edge of a closing vertex is taken.
- `TopOpeBRepTool_REGUS::NextinBlock` removes edges from the map it iterates; the port
  iterates a snapshot of the map.
- `FUN_tool_ClosedW` reads the REVERSED map twice where the FORWARD one is meant.
- `TopOpeBRepTool::PurgeClosingEdges(Fin, FF, ...)` returns true at the first wire with
  at most one closing edge.
- `TopOpeBRepTool_makeTransition::MkT2donE` passes `(mypb, mypa, mypE)` as the
  `(par1, f1, l1)` of `FUN_mkT2dquad`.
- `TopOpeBRepTool_mkTondgE` makes the directions of the face normals before testing
  that the normals were computed (Standard_ConstructionError for a failed one), and its
  members cross `xxi` with `faxis` as directions (unit results), unlike `FUN_MkTonE`.
- `NgApp` normalizes the unit normal again (`gp_Dir(gp_Vec)`), `CurvE` returns false
  with the curvature written in the general case.

### Verification

`python3 tools/verify_tkbool_numeric.py` runs `tt_tool`, `tt_topology`,
`tt_make_transition`, `tt_mktondge`, `tt_classi`, `tt_regularize`, `tt_corriso`,
`tt_fuse_edges` and `tt_purge_internal` (`tools/oracle/tkbool_probe_topopebreptool_tool.cxx`,
`tools/oracle/tkbool_probe_topopebreptool_regu.cxx`, corpus
`tools/tkbool_corpus_topopebreptool_tool.py`). The faces of `Regularize` come out of a
map of shapes whose native order follows memory addresses; the probes print them sorted
by their boxes. Unit tests with closed-form expectations are in
`crates/tkbool/tests/top_ope_brep_tool_tool.rs`.

## TopOpeBRepDS (TKI, connex, TOOL, EXPORT, ProcessInterferencesTool, repvg, redu, funk, interference tools, EIR, FIR, Filter, Reducer, Check, Dumper)

The free functions of `TopOpeBRepDS_EXPORT.cxx` live in `top_ope_brep_ds::export`, those
of `TopOpeBRepDS_ProcessInterferencesTool.cxx` in
`top_ope_brep_ds::process_interferences_tool`, the edge and face filters of
`TopOpeBRepDS_Process{Edge,Face}Interferences.cxx` in
`top_ope_brep_ds::process_{edge,face}_interferences`. A function taking the handled
structure locks it once (`change_ds`) and works on the structure; `_ds` twins take the
structure itself (a read lock is never taken again while one is held). Interference
lists walked with removals at iterators are walked as vectors: a removal at an index is
the native removal at an iterator, and the list is written back on the error paths that
the native code leaves half done (`FUN_select2dI`). The FDSCNX maps are a thread-local
cache (`fdscnx_close` empties it).

### Native quirks kept

- `FDS_repvg` drops the groups of a single interference per transition face from the
  list it reduces (only the groups of several are appended back).
- `FUN_select3dI` removes residual interferences with `lFE.Remove(it)` while `it` walks
  `lFEresi`: at the head of `lFEresi` the first interference of `lFE` goes and the walk
  continues in `lFE`; later the residual node is unlinked from `lFEresi` (natively `lFE`
  then miscounts, which is not modelled).
- `FUN_ds_completeforSE8` never stores its regrouped lists (`hasnew` stays false); only
  the transitions of the reduced 2d interferences change through their handles.
- `FUN_ds_completeforSE4` and `FUN_ds_completeforSE5` return out of the whole completion
  on inconsistent data; `FUN_ds_completeforSE5` drops the interferences of a point whose
  faces lie on the same side; `FUN_ds_completeforE7` selects the supports into `lREV`
  (whose size is tested) and rewrites every edge list grouped by geometry.
- `FUN_reclSE` keeps only the VERTEX geometries; `FUN_ProcessEdgeInterferences` drops
  the interferences on neither FACE nor EDGE transitions and the residual 3d ones;
  `TopOpeBRepDS_EIR::ProcessEdgeInterferences(EIX)` skips its disabled PNC filtering.
- `FUN_ReducerSDEdge` appends unpaired later interferences once per earlier one;
  `FUN_reducedoublons` treats every edge/vertex pair on a closed edge as the same
  vertex (`vG.IsSame(vG)`).
- `FUN_resolveFUNKNOWN` sets the classifier reference to the shape whose index is the
  rank of the support face; `FUN_UNKFstasta` adds point representations of the edge
  vertices on the edge rebuilt on the other face surface.
- `FUN_ds_mkTonFsdm` without `pardef` projects the origin; `CheckEdgeParameter` compares
  int-truncated parameters with 1e50; `TopOpeBRepDS_Check::ChkIntg` keeps only the check
  of the last shape list before the geometries; `ChkIntgSamDom` never resets its found
  flag; `FDS_EdgeIsConnexToSameDomainFaces` compares each face with itself.
- `TopOpeBRepDS_FaceInterferenceTool::isLine` is always false, so only the curvature
  frames are used; `STATIC_TOREVERSE` is a thread-local flag.

### Verification

`python3 tools/verify_tkbool_numeric.py` runs `dsx`
(`tools/oracle/tkbool_probe_topopebrepds_ops.cxx` and `_run.cxx`, corpus
`tools/tkbool_corpus_topopebrepds_ops.py`): structures of two solids (overlapping boxes,
a cylinder through a box, boxes sharing a face) with points, same domain pairs, section
edges and pseudo-random interferences of the native types, each of TKI, the selectors,
the 3d/2d/1d selections, the reductions (doubles, unknown, symmetric, repvg, reclSE,
scanloi, GetTr, hasI2d, redu2d1d), EIR, FIR, Filter, the EXPORT/TOOL/connex queries,
Check/Dumper and every FUN_ds_* completion printing the resulting interferences of every
shape (the section edges sorted: `TOOL::EShareG` adds them in the order of a shape map).
Unit tests with closed-form expectations are in
`crates/tkbool/tests/top_ope_brep_ds_ops.rs`.
