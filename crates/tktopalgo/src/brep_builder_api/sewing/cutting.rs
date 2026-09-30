// Rust translation of BRepBuilderAPI_Sewing::Cutting and ProjectPointsOnCurve
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cutting of the free boundaries at the projections of the nodes of other
//! boundaries that lie on them, and the projection of points on a curve.
use super::BRepBuilderAPISewing;
use super::evaluate::global_curve;
use super::failures::array1;
use crate::brep_builder_api::BRepBuilderAPIBndBoxTreeSelector;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{Array1, List, Sequence, UBTree, UBTreeFiller};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::bnd_lib::Add3dCurve;
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;

/// The native default `TolF` of `Extrema_ExtPC::Initialize`.
const EXTREMA_TOL_F: f64 = 1.0e-10;

/// The output arrays of `ProjectPointsOnCurve`: the distance (-1 when a point does not
/// project), the parameter and the projected point of each point.
pub(crate) struct Projections {
    pub dist: Array1<'static, f64>,
    pub para: Array1<'static, f64>,
    pub proj: Array1<'static, Point3>,
}

impl Projections {
    /// Arrays of `n` values, `NCollection_Array1(1, n)`.
    pub(crate) fn new(n: i32) -> Result<Self, Failure> {
        Ok(Self {
            dist: array1(n, 0.0)?,
            para: array1(n, 0.0)?,
            proj: array1(n, Point3::default())?,
        })
    }
}

/// The curve and its end points, shared by the projections of the points.
struct ProjectionCurve<'c> {
    adaptor: &'c GeomAdaptorCurve,
    range: (f64, f64),
    ends: (Point3, Point3),
}

impl BRepBuilderAPISewing {
    /// `ProjectPointsOnCurve(arrPnt, c3d, first, last, arrDist, arrPara, arrProj,
    /// isConsiderEnds)`: the nearest extremum of each point on the curve (not nearer
    /// than an end when the ends count, or for the first and last point); a maximum
    /// that an end beats is replaced by the end. With `is_consider_ends` a projection
    /// is kept within the tolerance only, and a point that does not project takes the
    /// nearer end within the tolerance (the minimal tolerance after an exception).
    pub(crate) fn project_points_on_curve(
        &self,
        arr_pnt: &Array1<'_, Point3>,
        c3d: &Handle<dyn GeomCurve>,
        (first, last): (f64, f64),
        out: &mut Projections,
        is_consider_ends: bool,
    ) -> Result<(), Failure> {
        out.dist.init(-1.0);
        let adaptor = GeomAdaptorCurve::with_curve(c3d)?;
        let mut projector = ExtPC::new();
        projector.initialize(&adaptor, first, last, EXTREMA_TOL_F)?;
        let ends = (adaptor.value(first)?, adaptor.value(last)?);
        let curve = ProjectionCurve {
            adaptor: &adaptor,
            range: (first, last),
            ends,
        };
        let lind = arr_pnt.length();
        for i1 in 1..=lind {
            let pt = arr_pnt[i1];
            let mut worktol: f64 = self.my_tolerance;
            let mut is_projected = false;
            let ends_count = (is_consider_ends || i1 == 1 || i1 == lind, is_consider_ends);
            let attempt = self.project_point(
                &mut projector,
                &curve,
                (i1, pt),
                ends_count,
                &mut is_projected,
                out,
            );
            // The native catch lowers the tolerance of the end test.
            if attempt.is_err() {
                worktol = self.min_tolerance();
            }
            if !is_projected && is_consider_ends {
                nearest_end(&curve, (i1, pt), worktol, out);
            }
        }
        Ok(())
    }

    /// The `try` block of one point: the nearest extremum (not farther than the nearer
    /// end when `consider`), and the end when the extremum is a maximum farther than it.
    fn project_point(
        &self,
        projector: &mut ExtPC<'_>,
        curve: &ProjectionCurve<'_>,
        (i1, pt): (i32, Point3),
        (consider, is_consider_ends): (bool, bool),
        is_projected: &mut bool,
        out: &mut Projections,
    ) -> Result<(), Failure> {
        let (pfirst, plast) = curve.ends;
        let (dist_f2, dist_l2) = (pfirst.square_distance(&pt), plast.square_distance(&pt));
        projector.perform(pt)?;
        if !projector.is_done() || projector.nb_ext()? <= 0 {
            return Ok(());
        }
        let mut dist2_min: f64 = if consider {
            std_min(dist_f2, dist_l2)
        } else {
            precision::infinite()
        };
        let mut ind_min: i32 = 0;
        for ind in 1..=projector.nb_ext()? {
            let d_proj2 = projector.square_distance(ind)?;
            if d_proj2 < dist2_min {
                ind_min = ind;
                dist2_min = d_proj2;
            }
        }
        if ind_min == 0 {
            return Ok(());
        }
        *is_projected = true;
        let mut param_proj = projector.point(ind_min)?.parameter();
        let mut pt_proj = curve.adaptor.value(param_proj)?;
        let mut dist_proj2 = pt_proj.square_distance(&pt);
        if !projector.is_min(ind_min)? && std_min(dist_f2, dist_l2) < dist2_min {
            (param_proj, dist_proj2, pt_proj) = if dist_f2 < dist_l2 {
                (curve.range.0, dist_f2, pfirst)
            } else {
                (curve.range.1, dist_l2, plast)
            };
        }
        let worktol = self.my_tolerance;
        if dist_proj2 < worktol * worktol || !is_consider_ends {
            out.dist[i1] = dist_proj2.sqrt();
            out.para[i1] = param_proj;
            out.proj[i1] = pt_proj;
        }
        Ok(())
    }
}

/// The nearer end of the curve for a point that did not project, within `worktol`.
fn nearest_end(
    curve: &ProjectionCurve<'_>,
    (i1, pt): (i32, Point3),
    worktol: f64,
    out: &mut Projections,
) {
    let (pfirst, plast) = curve.ends;
    let (dist_f2, dist_l2) = (pfirst.square_distance(&pt), plast.square_distance(&pt));
    if std_min(dist_f2, dist_l2) >= worktol * worktol {
        return;
    }
    if dist_f2 < dist_l2 {
        out.dist[i1] = dist_f2.sqrt();
        out.para[i1] = curve.range.0;
        out.proj[i1] = pfirst;
    } else {
        out.dist[i1] = dist_l2.sqrt();
        out.para[i1] = curve.range.1;
        out.proj[i1] = plast;
    }
}

/// The UB-tree of the boxes of the nodes' points, enlarged by half the tolerance.
fn node_tree(
    nodes: &super::SewingShapeIndexedMap,
    eps: f64,
) -> Result<UBTree<i32, BoundingBox3>, Failure> {
    let mut tree = UBTree::default();
    let mut filler = UBTreeFiller::new(&mut tree, None, true);
    for i in 1..=nodes.extent() {
        let point = BRepTool::pnt(&topods::vertex(nodes.find_key(i)?)?)?;
        let mut bounds = BoundingBox3::default();
        bounds.set_point(point);
        bounds.enlarge(eps);
        filler.add(i as i32, bounds);
    }
    filler.fill();
    drop(filler);
    Ok(tree)
}

impl BRepBuilderAPISewing {
    /// `Cutting(theProgress)`: each free boundary of faces is cut at the projections
    /// of the vertices of other nodes found near its curve; the sections of a cut
    /// boundary are recorded (`myBoundSections`, `mySectionBound`) and added to the
    /// sections of their nodes.
    pub(crate) fn cutting(&mut self, progress: &ProgressRange<'_>) -> Result<(), Failure> {
        let nb_vertices = self.my_vertex_node.extent();
        if nb_vertices == 0 {
            return Ok(());
        }
        let tree = node_tree(&self.my_vertex_node, self.my_tolerance * 0.5)?;
        let mut selector = BRepBuilderAPIBndBoxTreeSelector::new();
        let nb_bounds = self.my_bound_faces.extent();
        let scope =
            ProgressScope::with_literal(progress, "Cutting bounds", nb_bounds as f64, false);
        let mut i = 1;
        while i <= self.my_bound_faces.extent() && scope.more() {
            let bound = topods::edge(self.my_bound_faces.find_key(i)?)?;
            if !self.my_bound_faces.find_from_index(i)?.is_empty() {
                let sections = self.cut_bound(&bound, &tree, &mut selector)?;
                if sections.extent() > 1 {
                    self.record_sections(&bound, sections)?;
                }
            }
            i += 1;
            scope.next(1.0);
        }
        Ok(())
    }

    /// The candidate vertices of a boundary: the vertices of the nodes whose boxes meet
    /// the curve's box enlarged by the tolerance, except the boundary's own nodes; None
    /// when no box is met. Standard_NoSuchObject when a boundary vertex has no node.
    fn candidate_vertices(
        &self,
        bound: &Edge,
        (c3d, first, last): (&Handle<dyn GeomCurve>, f64, f64),
        tree: &UBTree<i32, BoundingBox3>,
        selector: &mut BRepBuilderAPIBndBoxTreeSelector,
    ) -> Result<Option<(IndexedMapOfShape, Vertex, Vertex)>, Failure> {
        let mut global_box = BoundingBox3::default();
        let adaptor = GeomAdaptorCurve::with_range(c3d, first, last)?;
        Add3dCurve::add(&adaptor, self.my_tolerance, &mut global_box)?;
        selector.set_current(global_box);
        tree.select(selector);
        if selector.res_ind().is_empty() {
            return Ok(None);
        }
        let (v1, v2) = topexp::vertices(bound, false);
        let node1 = self.my_vertex_node.find_from_key(&v1)?.clone();
        let node2 = self.my_vertex_node.find_from_key(&v2)?.clone();
        let mut candidates = IndexedMapOfShape::default();
        for &index in selector.res_ind().iter() {
            let node = self.my_vertex_node.find_from_index(index as usize)?;
            if !node.is_same(&node1) && !node.is_same(&node2) {
                candidates.add(self.my_vertex_node.find_key(index as usize)?.clone());
            }
        }
        selector.clear_res_list();
        Ok(Some((candidates, v1, v2)))
    }

    /// The sections of one boundary (empty when it is not cut).
    fn cut_bound(
        &mut self,
        bound: &Edge,
        tree: &UBTree<i32, BoundingBox3>,
        selector: &mut BRepBuilderAPIBndBoxTreeSelector,
    ) -> Result<List<Shape>, Failure> {
        let mut sections: List<Shape> = List::default();
        let (c3d, first, last) = global_curve(bound)?;
        if c3d.is_null() {
            return Ok(sections);
        }
        let curve = (&c3d, first, last);
        let Some((candidates, v1, v2)) = self.candidate_vertices(bound, curve, tree, selector)?
        else {
            return Ok(sections);
        };
        let nb_candidates = candidates.extent() as i32;
        if nb_candidates == 0 {
            return Ok(sections);
        }
        let mut arr_pnt = array1(nb_candidates, Point3::default())?;
        for j in 1..=nb_candidates {
            arr_pnt[j] = BRepTool::pnt(&topods::vertex(candidates.find_key(j as usize)?)?)?;
        }
        let mut projections = Projections::new(nb_candidates)?;
        self.project_points_on_curve(&arr_pnt, &c3d, (first, last), &mut projections, true)?;
        let (mut seq_node, mut seq_para) = (Sequence::default(), Sequence::default());
        let ends = (v1.into_shape(), v2.into_shape());
        self.create_cutting_nodes(
            &candidates,
            bound,
            &ends,
            &projections,
            &mut seq_node,
            &mut seq_para,
        )?;
        if seq_para.is_empty() {
            return Ok(sections);
        }
        self.create_sections(bound, &seq_node, &seq_para, &mut sections)?;
        Ok(sections)
    }

    /// The sections of a cut boundary added to the sections of their nodes, bound to
    /// the boundary, and the boundary bound to them.
    fn record_sections(&mut self, bound: &Edge, sections: List<Shape>) -> Result<(), Failure> {
        for section in sections.iter() {
            let mut it = ShapeIterator::new(section, true, true);
            while let Ok(child) = it.value() {
                let mut vertex = child.clone();
                if let Some(node) = self.my_vertex_node.seek(&vertex) {
                    vertex = topods::vertex(node)?.into_shape();
                }
                match self.my_node_sections.change_find(&vertex) {
                    Ok(list) => {
                        list.append(section.clone());
                    }
                    Err(_) => {
                        let mut list = List::default();
                        list.append(section.clone());
                        self.my_node_sections.bind(vertex, list);
                    }
                }
                it.next();
            }
            self.my_section_bound
                .bind(section.clone(), bound.clone().into_shape());
        }
        self.my_bound_sections
            .bind(bound.clone().into_shape(), sections);
        Ok(())
    }
}
