// Rust translation of the walks of BRepFeat_MakeRevolutionForm::Propagate of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeRevolutionForm.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The walks of the propagation: from one end of the section, the edge of the current
//! face under it leads to the next face, whose section edge continues the walk, until
//! the end reaches an end point of the wire; a walk turning back fails.
use super::BRepFeatMakeRevolutionForm;
use super::propagate::{Prop, at_ends};
use crate::brep_feat::make_linear_form::{next_face, section_edges};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_extrema::BRepExtremaExtPC;

/// One walk: the vertex it stands on, the two before it, the face it is on and the last
/// edge of the basis it crossed.
struct Walk {
    v: Vertex,
    vprevious: Vertex,
    vpreprevious: Vertex,
    current_face: Face,
    first_edge: Edge,
}

/// The point of a vertex (the origin for a null one, as the native default point).
fn pnt_or_origin(v: &Vertex) -> Result<Point3, Failure> {
    if v.is_null() {
        return Ok(Point3::default());
    }
    BRepTool::pnt(v)
}

impl Walk {
    /// Whether the walk turned back (its vertex on one of the two before it).
    fn turns_back(&self, my_tol: f64) -> Result<bool, Failure> {
        let pt = pnt_or_origin(&self.v)?;
        let ptprev = pnt_or_origin(&self.vprevious)?;
        let ptpreprev = pnt_or_origin(&self.vpreprevious)?;
        Ok(
            (!self.vprevious.is_null() && ptprev.distance(&pt) <= my_tol)
                || (!self.vpreprevious.is_null() && ptpreprev.distance(&pt) <= my_tol),
        )
    }

    /// The edge of the current face under the vertex (the nearest projection within the
    /// edge tolerance), kept from the previous step when none.
    fn edge_under(&mut self) -> Result<(), Failure> {
        for e in Explorer::new(&self.current_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let e = topods::edge(&e)?;
            let proj_f = BRepExtremaExtPC::with_vertex_edge(&self.v, &e)?;
            if !proj_f.is_done() || proj_f.nb_ext()? < 1 {
                continue;
            }
            let (mut dist2min, mut index) = (f64::MAX, 0);
            for sol in 1..=proj_f.nb_ext()? {
                if proj_f.square_distance(sol)? <= dist2min {
                    index = sol;
                    dist2min = proj_f.square_distance(sol)?;
                }
            }
            let tol = BRepTool::edge_tolerance(&e)?;
            if index != 0 && dist2min <= tol * tol {
                self.first_edge = e;
                break;
            }
        }
        Ok(())
    }

    /// The section edge of the face with an end at the vertex (the last one examined
    /// otherwise, none without section edge).
    fn section_at(&self, fac: &Face) -> Result<Edge, Failure> {
        let (p, t) = (
            BRepTool::pnt(&self.v)?,
            BRepTool::vertex_tolerance(&self.v)?,
        );
        let mut edg = Edge::default();
        for e in section_edges(fac, &self.current_face)? {
            edg = topods::edge(&e)?;
            let ppp1 = BRepTool::pnt(&topexp::first_vertex(&edg, true))?;
            let ppp2 = BRepTool::pnt(&topexp::last_vertex(&edg, true))?;
            if ppp1.distance(&p) <= t || ppp2.distance(&p) <= t {
                break;
            }
        }
        Ok(edg)
    }
}

impl BRepFeatMakeRevolutionForm {
    /// One step of a walk: the next face and its section edge recorded; false (and
    /// `false` returned natively) without section edge.
    fn step(
        &mut self,
        w: &mut Walk,
        fac: &Face,
        mapedges: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Option<Edge>, Failure> {
        w.edge_under()?;
        w.current_face = next_face(mapedges, &w.first_edge, &w.current_face)?;
        let edg = w.section_at(fac)?;
        let mut thelist = List::default();
        thelist.append(edg.clone().into_shape());
        self.slface
            .bind(w.current_face.clone().into_shape(), thelist);
        self.list_of_edges.append(edg.clone().into_shape());
        Ok(if edg.is_null() { None } else { Some(edg) })
    }

    /// The walk from the first end (`v1`), the faces met prepended to `sli_list`; None
    /// when it stops (`falseside` false when it turned back).
    fn walk_first(
        &mut self,
        sli_list: &mut List<Shape>,
        fac: &Face,
        w: &mut Walk,
        prop: &mut Prop,
        falseside: &mut bool,
        mapedges: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Option<()>, Failure> {
        while !prop.first_ok {
            if w.turns_back(self.tol)? {
                *falseside = false;
                return Ok(None);
            }
            let Some(edg1) = self.step(w, fac, mapedges)? else {
                return Ok(None);
            };
            sli_list.prepend(w.current_face.clone().into_shape());
            let vert = topexp::first_vertex(&edg1, true);
            let fp = BRepTool::pnt(&w.v)?;
            let tol = BRepTool::edge_tolerance(&edg1)?.max(BRepTool::vertex_tolerance(&w.v)?);
            w.vpreprevious = std::mem::take(&mut w.vprevious);
            w.vprevious = w.v.clone();
            w.v = if BRepTool::pnt(&vert)?.distance(&fp) <= tol {
                topexp::last_vertex(&edg1, true)
            } else {
                vert
            };
            if at_ends(&w.v, prop)? {
                prop.first_ok = true;
            }
        }
        Ok(Some(()))
    }

    /// The walk from the last end (`v2`), the faces met appended to `sli_list`; None
    /// when it stops (`falseside` false when it turned back).
    fn walk_last(
        &mut self,
        sli_list: &mut List<Shape>,
        fac: &Face,
        w: &mut Walk,
        prop: &mut Prop,
        falseside: &mut bool,
        mapedges: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Option<()>, Failure> {
        while !prop.last_ok {
            if w.turns_back(self.tol)? {
                *falseside = false;
                return Ok(None);
            }
            let Some(edg2) = self.step(w, fac, mapedges)? else {
                return Ok(None);
            };
            sli_list.append(w.current_face.clone().into_shape());
            let vert = topexp::first_vertex(&edg2, true);
            let fp = BRepTool::pnt(&w.v)?;
            if BRepTool::pnt(&vert)?.distance(&fp) <= BRepTool::vertex_tolerance(&w.v)? {
                w.vpreprevious = std::mem::take(&mut w.vprevious);
                w.vprevious = w.v.clone();
                w.v = topexp::last_vertex(&edg2, true);
            } else {
                // natively the previous vertices stay
                w.v = vert;
            }
            if at_ends(&w.v, prop)? {
                prop.last_ok = true;
            }
        }
        Ok(Some(()))
    }

    /// Both walks from the first face (the edge of the basis crossed carried from the
    /// first to the second as natively); None when one stops.
    pub(super) fn walk_both(
        &mut self,
        sli_list: &mut List<Shape>,
        fac: &Face,
        save_face: &Face,
        prop: &mut Prop,
        falseside: &mut bool,
    ) -> Result<Option<bool>, Failure> {
        let mut mapedges = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.rib.sbase,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut mapedges,
        )?;
        let mut w = Walk {
            v: prop.v1.clone(),
            vprevious: Vertex::default(),
            vpreprevious: Vertex::default(),
            current_face: save_face.clone(),
            first_edge: Edge::default(),
        };
        if self
            .walk_first(sli_list, fac, &mut w, prop, falseside, &mapedges)?
            .is_none()
        {
            return Ok(None);
        }
        let mut w2 = Walk {
            v: prop.v2.clone(),
            vprevious: Vertex::default(),
            vpreprevious: Vertex::default(),
            current_face: save_face.clone(),
            first_edge: w.first_edge,
        };
        if self
            .walk_last(sli_list, fac, &mut w2, prop, falseside, &mapedges)?
            .is_none()
        {
            return Ok(None);
        }
        Ok(Some(true))
    }
}
