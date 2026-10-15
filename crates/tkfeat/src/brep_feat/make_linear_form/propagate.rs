// Rust translation of BRepFeat_MakeLinearForm::Propagate of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeLinearForm.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Propagate`: the faces of the basis the profile crosses, from the first face along
//! the sections of the profile with them, each section edge recorded as sliding on its
//! face; the rib keeps sliding only when every edge crossed is perpendicular to the
//! profile.
use super::BRepFeatMakeLinearForm;
use crate::errors::no_such_object;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPISection;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_extrema::{BRepExtremaExtPC, BRepExtremaExtPF};

/// The section edges of the profile `fac` with the face `f` (approximated).
pub(crate) fn section_edges(fac: &Face, f: &Face) -> Result<Vec<Shape>, Failure> {
    let mut sect = BRepAlgoAPISection::from_shapes(fac, f, None, false)?;
    sect.approximation(true);
    sect.build()?;
    Ok(Explorer::new(sect.shape(), ShapeEnum::Edge, ShapeEnum::Shape)?.collect())
}

/// The state of the walk over the section edges (the native locals, kept from one
/// section to the next as natively).
#[derive(Default)]
pub(crate) struct SectionWalk {
    pub(crate) v1: Vertex,
    pub(crate) v2: Vertex,
    pub(crate) p1: Point3,
    pub(crate) p2: Point3,
    pub(crate) t1: f64,
    pub(crate) t2: f64,
    pub(crate) v1_ok: bool,
    pub(crate) v2_ok: bool,
    pub(crate) first_ok: bool,
    pub(crate) last_ok: bool,
}

impl SectionWalk {
    /// The ends of the section edge `ec` taken as the current vertices.
    pub(crate) fn take(&mut self, ec: &Edge) -> Result<(), Failure> {
        self.v1 = topexp::first_vertex(ec, true);
        self.v2 = topexp::last_vertex(ec, true);
        self.t1 = BRepTool::vertex_tolerance(&self.v1)?;
        self.t2 = BRepTool::vertex_tolerance(&self.v2)?;
        self.p1 = BRepTool::pnt(&self.v1)?;
        self.p2 = BRepTool::pnt(&self.v2)?;
        Ok(())
    }

    /// The first section edge with an end at the first or last point (its flags set).
    pub(crate) fn first_edge(
        &mut self,
        edges: &[Shape],
        firstpoint: Point3,
        lastpoint: Point3,
    ) -> Result<Edge, Failure> {
        for ec in edges {
            let ec = topods::edge(ec)?;
            self.take(&ec)?;
            let c1f = self.p1.distance(&firstpoint) <= self.t1;
            let c2f = self.p2.distance(&firstpoint) <= self.t2;
            let c1l = self.p1.distance(&lastpoint) <= self.t1;
            let c2l = self.p2.distance(&lastpoint) <= self.t2;
            if c1f || c2f || c1l || c2l {
                self.v1_ok |= c1f || c1l;
                self.v2_ok |= c2f || c2l;
                self.first_ok |= c1f || c2f;
                self.last_ok |= c1l || c2l;
                return Ok(ec);
            }
        }
        Ok(Edge::default())
    }

    /// The vertex the walk continues from and its point.
    pub(crate) fn previous(&self) -> (Vertex, Point3) {
        if self.v1_ok {
            (self.v2.clone(), self.p2)
        } else {
            (self.v1.clone(), self.p1)
        }
    }

    /// The section edge with an end at `ptprev` (the last edge examined otherwise, its
    /// flags false; none without section edge, the flags kept).
    pub(crate) fn next_edge(&mut self, edges: &[Shape], ptprev: Point3) -> Result<Edge, Failure> {
        let mut edg1 = Edge::default();
        for e in edges {
            edg1 = topods::edge(e)?;
            self.take(&edg1)?;
            self.v1_ok = self.p1.distance(&ptprev) <= self.t1;
            self.v2_ok = self.p2.distance(&ptprev) <= self.t2;
            if self.v1_ok || self.v2_ok {
                break;
            }
        }
        Ok(edg1)
    }

    /// The far end of the section edge checked against the first and last points
    /// (within twice its tolerance, which grows to the distance); false when the edge
    /// does not continue the chain.
    pub(crate) fn reach_ends(
        &mut self,
        firstpoint: Point3,
        lastpoint: Point3,
    ) -> Result<bool, Failure> {
        let (v, p, t) = if self.v1_ok {
            (self.v2.clone(), self.p2, self.t2)
        } else if self.v2_ok {
            (self.v1.clone(), self.p1, self.t1)
        } else {
            // end by chaining the section
            return Ok(false);
        };
        let bb = BRepBuilder::default();
        if !self.first_ok {
            let dp = p.distance(&firstpoint);
            if dp <= 2. * t {
                self.first_ok = true;
                bb.update_vertex_tolerance(&v, dp)?;
            }
        }
        if !self.last_ok {
            let dp = p.distance(&lastpoint);
            if dp <= 2. * t {
                self.last_ok = true;
                bb.update_vertex_tolerance(&v, dp)?;
            }
        }
        Ok(true)
    }
}

/// The edge of `face` the vertex lies on (the nearest projection within the edge
/// tolerance), with whether that edge is perpendicular to the profile (checked while
/// `result` holds); the previous edge when none.
pub(crate) fn edge_at_vertex(
    face: &Face,
    vprevious: &Vertex,
    fac: &Face,
    result: &mut bool,
    first_edge: &mut Edge,
) -> Result<(), Failure> {
    for rfe in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let rfe = topods::edge(&rfe)?;
        let proj_f = BRepExtremaExtPC::with_vertex_edge(vprevious, &rfe)?;
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
        let tol = BRepTool::edge_tolerance(&rfe)?;
        if index == 0 || dist2min > tol * tol {
            continue;
        }
        *first_edge = rfe.clone();
        // If the edge is not perpendicular to the plane of the rib it is required to set
        // Sliding(result) to false.
        if *result {
            *result = false;
            let ve1 = topexp::first_vertex(&rfe, true);
            let ve2 = topexp::last_vertex(&rfe, true);
            let mut perp =
                BRepExtremaExtPF::with_vertex_face(&ve1, fac, ExtFlag::MinMax, ExtAlgo::Grad)?;
            if perp.is_done() {
                let pe1 = perp.point(1)?;
                perp.perform(&ve2, fac)?;
                if perp.is_done() && pe1.distance(&perp.point(1)?) <= tol {
                    *result = true;
                }
            }
        }
        break;
    }
    Ok(())
}

/// The face other than `current` sharing the edge `first_edge` (or `current`).
pub(crate) fn next_face(
    mapedges: &IndexedDataMapOfShapeListOfShape,
    first_edge: &Edge,
    current: &Face,
) -> Result<Face, Failure> {
    let l = mapedges
        .find_from_key(first_edge)
        .map_err(|_| no_such_object("Propagate", "the edge is not in the basis"))?;
    for ff in l.iter() {
        if !ff.is_same(current) {
            return topods::face(ff);
        }
    }
    Ok(current.clone())
}

impl BRepFeatMakeLinearForm {
    /// The section edge `e` recorded as sliding on the face `f` (the face re-bound).
    fn slide_on(&mut self, f: &Face, e: &Edge) {
        let mut thelist = List::default();
        thelist.append(e.clone().into_shape());
        self.slface.bind(f.clone().into_shape(), thelist);
        self.list_of_edges.append(e.clone().into_shape());
    }

    /// `Propagate(SliList, fac, Firstpnt, Lastpnt, falseside)`: the faces crossed by the
    /// profile `fac` from the first face of `sli_list`; `falseside` turns false when
    /// the profile misses the first face; returns whether the rib can slide.
    pub fn propagate(
        &mut self,
        sli_list: &mut List<Shape>,
        fac: &Face,
        firstpoint: Point3,
        lastpoint: Point3,
        falseside: &mut bool,
    ) -> Result<bool, Failure> {
        let mut result = true;
        let mut current_face = topods::face(
            sli_list
                .first()
                .ok_or_else(|| no_such_object("Propagate", "no face"))?,
        )?;
        let mut sw = SectionWalk::default();
        let eb = sw.first_edge(&section_edges(fac, &current_face)?, firstpoint, lastpoint)?;
        if eb.is_null() {
            *falseside = false;
            return Ok(false);
        }
        self.list_of_edges.clear();
        self.slide_on(&current_face, &eb);
        // two points are on the same face.
        if sw.last_ok && sw.first_ok {
            return Ok(result);
        }
        let mut mapedges = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.rib.sbase,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut mapedges,
        )?;
        let mut first_edge = Edge::default();
        while !(sw.last_ok && sw.first_ok) {
            let (vprevious, ptprev) = sw.previous();
            // find edge connected to v1 or v2:
            edge_at_vertex(&current_face, &vprevious, fac, &mut result, &mut first_edge)?;
            current_face = next_face(&mapedges, &first_edge, &current_face)?;
            let edg1 = sw.next_edge(&section_edges(fac, &current_face)?, ptprev)?;
            if !sw.reach_ends(firstpoint, lastpoint)? {
                return Ok(false);
            }
            self.slide_on(&current_face, &edg1);
        }
        Ok(result)
    }
}
