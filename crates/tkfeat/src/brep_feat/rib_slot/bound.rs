// Rust translation of the parts shared by BRepFeat_MakeLinearForm::Init and
// BRepFeat_MakeRevolutionForm::Init of ModelingAlgorithms/TKFeat/BRepFeat at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The setup shared by the linear and revolution forms: the tolerance of the form, the
//! face of the bounding box of the basis in the plane of the wire, the sliding face of
//! the base wire and the shell of the faces crossed by a fixed profile.
use super::RibSlot;
use crate::errors::null_object;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICommon;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, Shell, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// The largest tolerance of the vertices of `s`, at least `tol`;
/// `max_vertex_tolerance(&shape, 1e-7)?`.
pub(crate) fn max_vertex_tolerance(s: &Shape, mut tol: f64) -> Result<f64, Failure> {
    for v in Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let t = BRepTool::vertex_tolerance(&topods::vertex(&v)?)?;
        if t > tol {
            tol = t;
        }
    }
    Ok(tol)
}

/// The shell of the faces of `faces`, closed when it is.
pub(crate) fn shell_of(faces: &List<Shape>) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut shell = Shell::default();
    b.make_shell(&mut shell);
    let mut comp = shell.into_shape();
    for f in faces.iter() {
        b.add(&mut comp, f)?;
    }
    comp.set_closed(BRepTool::is_closed(&comp)?)?;
    Ok(comp)
}

impl RibSlot {
    /// The bound of the basis and the face of its bounding box (moved twice the bound
    /// away) in the plane `pln`; `rib.bound_face(&pln)?`.
    pub(crate) fn bound_face(&self, pln: &Handle<GeomPlane>) -> Result<(f64, Face), Failure> {
        let (bnd, first_corner, last_corner) = self.height_max(&self.sbase, &Shape::default())?;
        let bnd_box = BRepPrimAPIMakeBox::from_points(first_corner, last_corner)?.solid()?;
        let pln = pln
            .get()
            .ok_or_else(|| null_object("BRepFeat_RibSlot", "null plane"))?
            .pln();
        let b = 6. * bnd;
        let plane_face = BRepLibMakeFace::from_plane_bounds(pln, (-b, b, -b, b))?.face()?;
        let plane_s = BRepAlgoAPICommon::from_shapes(&bnd_box, &plane_face)?;
        let www = topods::wire(
            Explorer::new(plane_s.shape(), ShapeEnum::Wire, ShapeEnum::Shape)?.current(),
        )?;
        Ok((
            bnd,
            BRepLibMakeFace::from_plane_wire(pln, &www, true)?.face()?,
        ))
    }

    /// The base wire made the sketch face and the profile of a sliding form, on the
    /// plane with the tolerance `tol`, without shape Until.
    pub(crate) fn sliding_face(
        &mut self,
        pln: &Handle<GeomPlane>,
        w: &Wire,
        tol: f64,
    ) -> Result<(), Failure> {
        let b = BRepBuilder::default();
        let mut f = Face::default();
        b.make_face_on_surface(&mut f, &pln.clone().cast(), tol)?;
        w.set_closed(BRepTool::is_closed(w)?)?;
        b.add(&mut f, w)?;
        self.skface = f.clone();
        self.pbase = f;
        self.s_until.nullify();
        Ok(())
    }

    /// The faces of the basis bound to themselves in the history (the end of `Init`).
    pub(crate) fn bind_basis_faces(&mut self) -> Result<(), Failure> {
        for f in Explorer::new(&self.sbase, ShapeEnum::Face, ShapeEnum::Shape)? {
            let mut thelist = List::default();
            thelist.append(f.clone());
            self.map.bind(f, thelist);
        }
        Ok(())
    }
}
