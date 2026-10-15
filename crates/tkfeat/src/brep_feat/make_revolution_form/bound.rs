// Rust translation of the new bounding face of BRepFeat_MakeRevolutionForm::Init of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeRevolutionForm.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bounding face of a fixed rib of revolution: the bounding face of the basis cut
//! by the axis, the part holding the material (the check point) kept.
use super::BRepFeatMakeRevolutionForm;
use crate::brep_feat::rib_slot::{ProfileInput, int_par};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbool::brep_algo;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomSurface};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api;
use rustycad_tkgeomalgo::geom2d_api::InterCurveCurve;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// The wire being built from the bounding wire: the vertices where the axis cuts it.
#[derive(Default)]
struct Cut {
    new_v1: Vertex,
    last_v: Vertex,
}

impl BRepFeatMakeRevolutionForm {
    /// The vertex where the axis (2d line `ln2d`) cuts the edge `e` within its range.
    fn axis_cut(
        &self,
        input: &ProfileInput,
        e: &Edge,
        ln2d: &Handle<dyn Geom2dCurve>,
    ) -> Result<Vertex, Failure> {
        let (ln, f, l) = BRepTool::curve(e)?;
        let l2d = geom_api::to_2d(&ln, &input.plane()?.pln())?;
        let intcc = InterCurveCurve::between(&l2d, ln2d, precision::confusion())?;
        let mut vv = Vertex::default();
        if intcc.nb_points() > 0 {
            let p = intcc.point(1)?;
            let point = input.plane()?.d0(p.x(), p.y())?;
            let par = int_par(&ln, point)?;
            if f <= par && l >= par {
                BRepBuilder::default().make_vertex_at(&mut vv, point, precision::confusion())?;
            }
        }
        Ok(vv)
    }

    /// The bounding wire from the first cut of the axis to the second one, closed back
    /// on the first cut.
    fn cut_wire(&self, input: &ProfileInput) -> Result<Wire, Failure> {
        let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(self.axe)).cast();
        let ln2d = geom_api::to_2d(&line, &input.plane()?.pln())?;
        let bu = BRepBuilder::default();
        let mut wiwiwi = Wire::default();
        bu.make_wire(&mut wiwiwi);
        let www = topods::wire(
            Explorer::new(&input.bnd_face, ShapeEnum::Wire, ShapeEnum::Shape)?.current(),
        )?;
        let mut explo = WireExplorer::with_wire(&www)?;
        let mut cut = Cut::default();
        while explo.more() {
            let e = explo.current().clone();
            let v2 = topexp::last_vertex(&e, true);
            let vv = self.axis_cut(input, &e, &ln2d)?;
            match (vv.is_null(), cut.new_v1.is_null()) {
                (true, true) => {}
                (false, true) => {
                    cut.new_v1 = vv;
                    cut.last_v = v2;
                    let ee1 = BRepLibMakeEdge::from_vertices(&cut.new_v1, &cut.last_v)?.edge()?;
                    bu.add(&mut wiwiwi, &ee1)?;
                }
                (true, false) => {
                    // natively an unused edge from the last vertex is also made here
                    cut.last_v = v2;
                    bu.add(&mut wiwiwi, &e)?;
                }
                (false, false) => {
                    let ee1 = BRepLibMakeEdge::from_vertices(&cut.last_v, &vv)?.edge()?;
                    bu.add(&mut wiwiwi, &ee1)?;
                    let ee2 = BRepLibMakeEdge::from_vertices(&vv, &cut.new_v1)?.edge()?;
                    bu.add(&mut wiwiwi, &ee2)?;
                    break;
                }
            }
            explo.next()?;
        }
        wiwiwi.set_closed(BRepTool::is_closed(&wiwiwi)?)?;
        Ok(wiwiwi)
    }

    /// The bounding face on the side of the axis holding the check point (None when it
    /// is not valid, InvShape natively).
    pub(super) fn new_bound_face(&self, input: &ProfileInput) -> Result<Option<Face>, Failure> {
        let pln = input.plane()?.pln();
        let wiwiwi = self.cut_wire(input)?;
        let mut new_bnd_face = BRepLibMakeFace::from_plane_wire(pln, &wiwiwi, true)?.face()?;
        let cl = BRepTopAdaptorFClass2d::new(&new_bnd_face, precision::confusion())?;
        let (paru, parv) = elslib::parameters(&pln, input.check_pnt);
        if cl.perform(Point2::new(paru, parv), true)? == State::Out {
            let c = BRepAlgoAPICut::from_shapes(&input.bnd_face, &new_bnd_face)?;
            let a_cur_wire = topods::wire(
                Explorer::new(c.shape(), ShapeEnum::Wire, ShapeEnum::Shape)?.current(),
            )?;
            new_bnd_face = BRepLibMakeFace::from_plane_wire(pln, &a_cur_wire, true)?.face()?;
        }
        if !brep_algo::is_valid(&new_bnd_face)? {
            return Ok(None);
        }
        Ok(Some(new_bnd_face))
    }
}
