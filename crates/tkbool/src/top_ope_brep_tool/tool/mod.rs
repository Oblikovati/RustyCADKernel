// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_TOOL.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871 (Nt, NggeomF and minDUV are in normal.rs, TggeomE
// in tool_tangent.rs and UVISO in tool_iso.rs, ported earlier).

//! `TopOpeBRepTool_TOOL`: the static helpers of the topological operations on edges
//! and faces: orientations of sub-shapes, vertices and parameters of edges, UV
//! tangents and translations of p-curves, matter angles between faces, curvatures,
//! splits of edges and faces made of wires.
mod curvature;
mod matter;
mod orientation;
mod uv;

use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Face, Shape, Shell};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// The orientation codes of `OriinSor`, `OnBoundary` and `TgINSIDE`: FORWARD.
pub const TOOL_FORWARD: i32 = 1;
/// REVERSED.
pub const TOOL_REVERSED: i32 = 2;
/// INTERNAL.
pub const TOOL_INTERNAL: i32 = 3;
/// EXTERNAL.
pub const TOOL_EXTERNAL: i32 = 4;
/// CLOSING.
pub const TOOL_CLOSING: i32 = 5;

/// The `stuvF` codes: below the first bound.
pub const TOOL_INFFIRST: i32 = -1;
/// Above the last bound.
pub const TOOL_SUPLAST: i32 = -2;
/// On the first bound.
pub const TOOL_ONFIRST: i32 = 1;
/// On the last bound.
pub const TOOL_ONLAST: i32 = 2;

/// The static helpers of `TopOpeBRepTool_TOOL` (a namespace; the members ported
/// before are the free functions `nt`, `nggeom_f`, `min_duv`, `tggeom_e` and `uviso`);
/// `TopOpeBRepToolTOOL::remove(&mut list, &shape)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct TopOpeBRepToolTOOL;

/// A map of shapes to lists of shapes (`TopTools_DataMapOfShapeListOfShape`).
pub type ShapeListMap = DataMap<Shape, List<Shape>, ShapeMapHasher>;

impl TopOpeBRepToolTOOL {
    /// `MkShell(lF, She)`: a new shell of the faces; `TopOpeBRepToolTOOL::mk_shell(&faces)?`.
    pub fn mk_shell(l_f: &List<Shape>) -> Result<Shape, Failure> {
        let bb = BRepBuilder::default();
        let mut shell = Shell::default();
        bb.make_shell(&mut shell);
        let mut she = shell.into_shape();
        for f in l_f.iter() {
            bb.add(&mut she, f)?;
        }
        Ok(she)
    }

    /// `Remove(loS, toremove)`: every shape equal to `toremove` removed from the list;
    /// whether one was; `TopOpeBRepToolTOOL::remove(&mut l, &s)`.
    pub fn remove(lo_s: &mut List<Shape>, toremove: &Shape) -> bool {
        let before = lo_s.len();
        lo_s.retain(|s| !s.is_equal(toremove));
        lo_s.len() != before
    }

    /// `WireToFace(Fref, mapWlow, lFs)`: for each wire key, an empty copy of the face
    /// oriented FORWARD holding the wire and its lower wires, REVERSED when `fref` is;
    /// `TopOpeBRepToolTOOL::wire_to_face(&f, &map, &mut faces)?`.
    pub fn wire_to_face(
        fref: &Face,
        map_wlow: &ShapeListMap,
        l_fs: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        let bb = BRepBuilder::default();
        let f = topods::face(&fref.oriented(Orientation::Forward))?;
        let toreverse = fref.orientation() == Orientation::Reversed;
        for (key, lower) in map_wlow.items() {
            let mut ff = f.empty_copied()?;
            bb.add(&mut ff, &topods::wire(key)?.into_shape())?;
            for wwi in lower.iter() {
                bb.add(&mut ff, &topods::wire(wwi)?.into_shape())?;
            }
            if toreverse {
                ff.set_orientation(Orientation::Reversed);
            }
            l_fs.append(ff);
        }
        Ok(true)
    }
}
