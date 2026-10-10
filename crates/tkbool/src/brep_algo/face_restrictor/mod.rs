// Rust translation of ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo_FaceRestrictor.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepAlgo_FaceRestrictor`: the faces of the surface of a face bounded by a set of
//! closed wires, by the TopOpeBRepBuild wire to face builder or, with orientation
//! control, by the nesting of the wires.
mod correction;

use crate::top_ope_brep_build::TopOpeBRepBuildWireToFace;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomGeometry, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::toploc::Location;

/// The wires nesting a wire, or nested in it, by wire.
pub(crate) type WireNesting = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The builder of faces bounded by wires (`BRepAlgo_FaceRestrictor`); natively the
/// flags are uninitialized until Init and Perform (false here);
/// `let mut fr = BRepAlgoFaceRestrictor::new(); fr.init(&face, false, false); fr.add(&w); fr.perform()?;`.
#[derive(Clone, Default)]
pub struct BRepAlgoFaceRestrictor {
    my_done: bool,
    mode_proj: bool,
    my_face: Face,
    wires: List<Shape>,
    faces: List<Shape>,
    my_correction: bool,
    key_is_in: WireNesting,
    key_contains: WireNesting,
}

/// `ChangePCurve(E, S, L)` (static): the first p-curve of the edge set on (`s`, `l`);
/// true when the edge has none (and nothing is set).
fn change_pcurve(e: &Edge, s: &Handle<dyn GeomSurface>, l: &Location) -> Result<bool, Failure> {
    let c2 = BRepTool::curve_on_surface_by_index(e, 1)?.pcurve;
    if !c2.is_null() {
        BRepBuilder::default().update_edge_pcurve(e, &c2, s, l, precision::confusion(), None)?;
    }
    Ok(c2.is_null())
}

/// `ProjCurve3d(E, S, L)` (static): the projection on (`s`, `l`) of the trimmed 3d
/// curve of the edge set as its p-curve; false without 3d curve.
fn proj_curve3d(e: &Edge, s: &Handle<dyn GeomSurface>, l: &Location) -> Result<bool, Failure> {
    let c = BRepTool::curve_with_location(e)?;
    if c.curve.is_null() {
        return Ok(false);
    }
    let ct = Handle::new(GeomTrimmedCurve::new(
        &c.curve, c.first, c.last, true, true,
    )?);
    let ll = l.inverted()?.multiplied(&c.location)?;
    ct.transform(ll.transformation())?;
    let c2 = geom_proj_lib::curve2d(&ct.cast::<dyn GeomCurve>(), s)?;
    BRepBuilder::default().update_edge_pcurve(e, &c2, s, l, precision::confusion(), None)?;
    Ok(true)
}

impl BRepAlgoFaceRestrictor {
    /// `BRepAlgo_FaceRestrictor()`; `BRepAlgoFaceRestrictor::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init(F, Proj, ControlOrientation)` (natively defaulted to false, false): the
    /// face whose surface bounds the new faces; `fr.init(&face, false, false)`.
    pub fn init(&mut self, f: &Face, proj: bool, control_orientation: bool) {
        self.my_face = f.clone();
        self.mode_proj = proj;
        self.my_correction = control_orientation;
    }

    /// `Add(W)`: a closed wire; `fr.add(&wire)`.
    pub fn add(&mut self, w: &Wire) {
        self.wires.append(w.as_ref().clone());
    }

    /// `Clear()`: no wire and no face; `fr.clear()`.
    pub fn clear(&mut self) {
        self.wires.clear();
        self.faces.clear();
    }

    /// The p-curve of `e` on (`s`, `l`) when it has none: the projection of its 3d
    /// curve (Proj), else its first p-curve and then the projection (the native
    /// ChangePCurve answers whether there was none); false when a projection fails.
    fn update_pcurve(
        &self,
        e: &Edge,
        s: &Handle<dyn GeomSurface>,
        l: &Location,
    ) -> Result<bool, Failure> {
        if !BRepTool::curve_on_surface(e, s, l)?.pcurve.is_null() {
            return Ok(true);
        }
        if self.mode_proj {
            return proj_curve3d(e, s, l);
        }
        let ya_pcurve = change_pcurve(e, s, l)?;
        if !ya_pcurve {
            return proj_curve3d(e, s, l);
        }
        Ok(true)
    }

    /// `Perform()`: with orientation control the nesting of the wires, else the faces
    /// of the wire to face builder once each edge has a p-curve on the surface (not
    /// done when a projection fails); `fr.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        if self.my_correction {
            return self.perform_with_correction();
        }
        self.my_done = false;
        let (s, l) = BRepTool::surface_with_location(&self.my_face)?;
        let mut wtf = TopOpeBRepBuildWireToFace::new();
        for w in self.wires.iter() {
            for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
                if !self.update_pcurve(&topods::edge(&e)?, &s, &l)? {
                    return Ok(());
                }
            }
            wtf.add_wire(&topods::wire(w)?);
        }
        wtf.make_faces(&self.my_face, &mut self.faces)?;
        self.my_done = true;
        Ok(())
    }

    /// `IsDone()`; `fr.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }

    /// `More()`: faces remain; `fr.more()`.
    pub fn more(&self) -> bool {
        !self.faces.is_empty()
    }

    /// `Next()`: the first face removed; Standard_NoSuchObject without one; `fr.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol.
    pub fn next(&mut self) -> Result<(), Failure> {
        self.faces.remove_first()
    }

    /// `Current()`: the first face; Standard_NoSuchObject without one; `fr.current()?`.
    pub fn current(&self) -> Result<Face, Failure> {
        let first = self.faces.first().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "BRepAlgo_FaceRestrictor::Current: no face; expected More()",
                "BRepAlgo_FaceRestrictor",
            )
        })?;
        topods::face(first)
    }
}
