# TKDraw port status

**The three packages are ported and claimed; only the native window backend is open.**
`crates/tkdraw` (`rustycad-tkdraw`) is the port of the Draw toolkit TKDraw, on the
project-owned Tcl/Tk binding `crates/tcl` (`rustycad-tcl`). It depends on the TKernel,
TKMath, TKG2d, TKG3d, TKGeomBase, TKGeomAlgo, TKBRep, TKTopAlgo, TKHLR and TKMesh ports.
[draw-design.md](draw-design.md) fixes how the globals, the class hierarchies, the
window layer and the persistence map to Rust.

| Package | Module | State |
| --- | --- | --- |
| Draw | `draw` | ported; `Draw_Window` is a trait pair with the batch backend only |
| DrawTrSurf | `draw_tr_surf` | ported |
| DBRep | `dbrep` | ported |

`drawexe` registers `Draw::Commands`, `DBRep::BasicCommands` and
`DrawTrSurf::BasicCommands`, as `Draw_InitAppli` of the C++ DRAWEXE does.

## Checks

- `tools/verify_tkdraw_oracle.py --native-build target/occt-all-full` runs the scripts
  of `crates/tkdraw/tests/oracle` on native DRAWEXE and on `drawexe` (batch mode, empty
  `DRAWDEFAULT`) and compares the outputs: 9 of 9 match byte for byte. The scripts cover
  the interpreter, expressions, variables, the basic, graphic (batch), unit and message
  commands, `bsave`/`brestore`, every DrawTrSurf drawable kind restored from GeomTools
  records with all DrawTrSurf commands, and the DBRep commands on a BRep file of the
  repository (counts, explode/nexplode, compound/add/emptycopy, orientations, flags,
  isos/hlr/triangles/polygons settings, normals, `writebrep`/`readbrep` in ASCII and
  binary, `XProgress`).
- Batch mode never draws. The drawing paths (curves, splines with poles and knots,
  surfaces with isos, points, polygons, triangulations, shapes with isos, edges,
  orientation arrows, hidden lines, triangulations and polygons of meshed shapes) are
  unit-tested on a recording window system (`draw::fake_window`).
- OCCT has no GoogleTest suite for TKDraw.

## Known gaps

- Native window backend: `Draw_Window` (constructor, `init`, `Destroy`, the
  terminate callbacks, `InitBuffer`/`SetUseBuffer`/`GetUseBuffer`, `WExpose`),
  `Init_Appli`, `Run_Appli`, `Destroy_Appli` and the `xwd` snapshot wait for the window
  layer of the TKService port (draw-design.md section 3). The `Draw_Window` members the
  viewer calls are rows with status `partial` on the `DrawWindow`/`DrawWindowSystem`
  traits; the others are unclaimed.
- The polygon and triangulation `Dump`/`Save` of DrawTrSurf go through the TKMath Poly
  writer, which formats its numbers itself (`%g`) instead of honouring the stream
  (precision 15 for `dump`, scientific with 15 digits for `Save`): those rows are
  `partial`.
- The DrawTrSurf and DBRep drawing paths have no native byte-level oracle (native
  views need an X display; the PostScript `hardcopy` of a native view would be one).
