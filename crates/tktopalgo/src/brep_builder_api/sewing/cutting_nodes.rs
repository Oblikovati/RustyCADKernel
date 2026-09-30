// Rust translation of BRepBuilderAPI_Sewing::CreateCuttingNodes and CreateSections
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cutting vertices of a boundary (at the projections of the candidate nodes) and
//! the sections between them, with the boundary's p-curves.
use super::BRepBuilderAPISewing;
use super::cutting::Projections;
use super::evaluate::global_curve;
use super::vertex_tolerance::find_nm_vertices;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::gp::Point3;

/// The ordered cutting points of a boundary, its two ends included while they are
/// built (a distance of -1 marks an end not yet bound to a node).
struct CuttingPoints<'s> {
    vertices: &'s mut Sequence<Shape>,
    params: &'s mut Sequence<f64>,
    dists: Sequence<f64>,
    points: Sequence<Point3>,
}

/// The indices of the projected candidates, by increasing distance (the first of equal
/// distances first).
fn ordered_by_distance(
    dist: &rustycad_tkernel::ncollection::Array1<'_, f64>,
    nb_proj: i32,
) -> Result<Vec<i32>, Failure> {
    let mut ordered_index: Sequence<i32> = Sequence::default();
    let mut ordered_distance: Sequence<f64> = Sequence::default();
    for i in 1..=nb_proj {
        let dist_proj = dist[i];
        // Skip vertex if not projected.
        if dist_proj < 0.0 {
            continue;
        }
        let mut is_inserted = false;
        let mut j = 1;
        while j <= ordered_index.length() && !is_inserted {
            is_inserted = dist_proj < *ordered_distance.value(j as usize)?;
            if is_inserted {
                ordered_index.insert_before(j as usize, i)?;
                ordered_distance.insert_before(j as usize, dist_proj)?;
            }
            j += 1;
        }
        if !is_inserted {
            ordered_index.append(i);
            ordered_distance.append(dist_proj);
        }
    }
    Ok(ordered_index.iter().copied().collect())
}

impl CuttingPoints<'_> {
    /// The index of the cutting point nearest to `point` (the first of equal ones).
    fn nearest(&self, point: &Point3) -> Result<(usize, f64), Failure> {
        let mut index_min = 1;
        let mut dist_min = point.distance(self.points.value(1)?);
        for j in 2..=self.points.len() {
            let dist = point.distance(self.points.value(j)?);
            if dist < dist_min {
                dist_min = dist;
                index_min = j;
            }
        }
        Ok((index_min, dist_min))
    }

    /// A new cutting vertex inserted before the first point of larger or equal
    /// parameter (from the second one); a parameter beyond the last point inserts
    /// nothing, as natively.
    fn insert(
        &mut self,
        vertex: &Vertex,
        (par, dist, point): (f64, f64, Point3),
    ) -> Result<(), Failure> {
        for j in 2..=self.params.len() {
            if par <= *self.params.value(j)? {
                self.vertices
                    .insert_before(j, vertex.clone().into_shape())?;
                self.params.insert_before(j, par)?;
                self.dists.insert_before(j, dist)?;
                self.points.insert_before(j, point)?;
                break;
            }
        }
        Ok(())
    }
}

impl BRepBuilderAPISewing {
    /// `CreateCuttingNodes(MapVert, bound, vfirst, vlast, arrDist, arrPara, arrPnt,
    /// seqVert, seqPara)`: by increasing distance, each projected node not yet bound
    /// is bound to the nearest end (when near it and the end is free), to nothing (a
    /// secondary node, near an end already bound or a cutting vertex), or to a new
    /// cutting vertex inserted by parameter; the links node / cutting vertex are
    /// recorded, and the inner cutting vertices and parameters are returned.
    pub(crate) fn create_cutting_nodes(
        &mut self,
        map_vert: &IndexedMapOfShape,
        bound: &Edge,
        (vfirst, vlast): &(Shape, Shape),
        projections: &Projections,
        seq_vert: &mut Sequence<Shape>,
        seq_para: &mut Sequence<f64>,
    ) -> Result<(), Failure> {
        let ordered = ordered_by_distance(&projections.dist, map_vert.extent() as i32)?;
        if ordered.is_empty() {
            return Ok(());
        }
        let (c3d, first, last) = global_curve(bound)?;
        let adaptor = GeomAdaptorCurve::with_curve(&c3d)?;
        seq_vert.prepend(vfirst.clone());
        seq_vert.append(vlast.clone());
        seq_para.prepend(first);
        seq_para.append(last);
        let mut points = CuttingPoints {
            vertices: seq_vert,
            params: seq_para,
            dists: Sequence::default(),
            points: Sequence::default(),
        };
        points.dists.prepend(-1.0);
        points.dists.append(-1.0);
        points.points.prepend(adaptor.value(first)?);
        points.points.append(adaptor.value(last)?);
        let mut node_cutting_vertex = super::SewingShapeIndexedMap::default();
        for index in ordered {
            let node = self
                .my_vertex_node
                .find_from_key(map_vert.find_key(index as usize)?)?
                .clone();
            if node_cutting_vertex.contains(&node) {
                continue;
            }
            let (dis_proj, pnt_proj) = (projections.dist[index], projections.proj[index]);
            let (index_min, dist_min) = points.nearest(&pnt_proj)?;
            if dist_min <= std_max(dis_proj * 0.1, self.min_tolerance()) {
                let cvertex = if *points.dists.value(index_min)? < 0.0 {
                    points.dists.set_value(index_min, dis_proj)?;
                    points.vertices.value(index_min)?.clone()
                } else {
                    Shape::default()
                };
                node_cutting_vertex.add(node, cvertex);
                continue;
            }
            let mut cvertex = Vertex::default();
            BRepBuilder::default().make_vertex_at(
                &mut cvertex,
                pnt_proj,
                precision::confusion(),
            )?;
            node_cutting_vertex.add(node, cvertex.clone().into_shape());
            points.insert(&cvertex, (projections.para[index], dis_proj, pnt_proj))?;
        }
        self.link_cutting_nodes(&node_cutting_vertex)?;
        seq_vert.remove(1)?;
        seq_vert.remove(seq_vert.len())?;
        seq_para.remove(1)?;
        seq_para.remove(seq_para.len())
    }

    /// The links node -> cutting vertex (an end is replaced by its node) and cutting
    /// vertex -> node (for a new cutting vertex).
    fn link_cutting_nodes(
        &mut self,
        node_cutting_vertex: &super::SewingShapeIndexedMap,
    ) -> Result<(), Failure> {
        for i in 1..=node_cutting_vertex.extent() {
            let mut cnode = node_cutting_vertex.find_from_index(i)?.clone();
            // Skip secondary nodes.
            if cnode.is_null() {
                continue;
            }
            let node = node_cutting_vertex.find_key(i)?.clone();
            if let Some(end_node) = self.my_vertex_node.seek(&cnode) {
                cnode = end_node.clone();
            } else {
                let mut list = List::default();
                list.append(node.clone());
                self.my_cutting_node.bind(cnode.clone(), list);
            }
            match self.my_cutting_node.change_find(&node) {
                Ok(list) => {
                    list.append(cnode);
                }
                Err(_) => {
                    let mut list = List::default();
                    list.append(cnode);
                    self.my_cutting_node.bind(node, list);
                }
            }
        }
        Ok(())
    }

    /// `CreateSections(section, seqNode, seqPara, listEdge)`: empty copies of the
    /// boundary between consecutive cutting vertices, each with its range and the
    /// non-manifold vertices inside it, then the boundary's p-curves (the seam pairs
    /// too) copied onto each.
    pub(crate) fn create_sections(
        &self,
        section: &Edge,
        seq_node: &Sequence<Shape>,
        seq_para: &Sequence<f64>,
        list_edge: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let (mut nm_vertices, mut nm_pars) = (Sequence::default(), Sequence::default());
        find_nm_vertices(section, &mut nm_vertices, &mut nm_pars)?;
        let (first, last) = BRepTool::range(section)?;
        let len = seq_para.len() + 1;
        for i in 1..=len {
            let (par1, par2, v1, v2) = if i == 1 {
                (
                    first,
                    *seq_para.value(i)?,
                    topexp::first_vertex(section, false).into_shape(),
                    seq_node.value(i)?.clone(),
                )
            } else if i == len {
                (
                    *seq_para.value(i - 1)?,
                    last,
                    seq_node.value(i - 1)?.clone(),
                    topexp::last_vertex(section, false).into_shape(),
                )
            } else {
                (
                    *seq_para.value(i - 1)?,
                    *seq_para.value(i)?,
                    seq_node.value(i - 1)?.clone(),
                    seq_node.value(i)?.clone(),
                )
            };
            let edge = section_piece(
                section,
                (&v1, &v2),
                (par1, par2),
                (&mut nm_vertices, &mut nm_pars),
            )?;
            list_edge.append(edge.into_shape());
        }
        let list_faces = self.my_bound_faces.find_from_key(section)?;
        if list_faces.is_empty() {
            return Ok(());
        }
        let tol_edge = BRepTool::edge_tolerance(section)?;
        for face in list_faces.iter() {
            copy_pcurves(section, &topods::face(face)?, list_edge, tol_edge)?;
        }
        Ok(())
    }
}

/// One section: an empty copy of the boundary with its two vertices, its range and the
/// non-manifold vertices inside the range (removed from the sequences once used).
fn section_piece(
    section: &Edge,
    (v1, v2): (&Shape, &Shape),
    (par1, par2): (f64, f64),
    (nm_vertices, nm_pars): (&mut Sequence<Shape>, &mut Sequence<f64>),
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = section.clone();
    edge.empty_copy()?;
    let mut tmp = edge.oriented(Orientation::Forward);
    builder.add(&mut tmp, &v1.oriented(Orientation::Forward))?;
    builder.add(&mut tmp, &v2.oriented(Orientation::Reversed))?;
    builder.range(&Edge::unchecked(tmp.clone()), par1, par2, false)?;
    let mut k = 1;
    while k <= nm_pars.len() {
        let par = *nm_pars.value(k)?;
        if par >= par1 && par <= par2 {
            builder.add(&mut tmp, nm_vertices.value(k)?)?;
            nm_vertices.remove(k)?;
            nm_pars.remove(k)?;
            continue;
        }
        k += 1;
    }
    Ok(edge)
}

/// The boundary's p-curve(s) on one face copied onto every section (a seam pair ordered
/// by the section's orientation in the face); a face without p-curve is skipped.
fn copy_pcurves(
    section: &Edge,
    face: &Face,
    list_edge: &List<Shape>,
    tol_edge: f64,
) -> Result<(), Failure> {
    let c2d = BRepTool::curve_on_face(section, face)?.pcurve;
    let Some(c) = c2d.get() else { return Ok(()) };
    let is_seam = BRepTool::is_closed_on_face(section, face)?;
    let mut c2d1: Handle<dyn Geom2dCurve> = Handle::default();
    if is_seam {
        c2d1 = BRepTool::curve_on_face(&Edge::unchecked(section.reversed()), face)?.pcurve;
        if c2d1.is_null() {
            return Ok(());
        }
    }
    let builder = BRepBuilder::default();
    for shape in list_edge.iter() {
        let edge = topods::edge(shape)?;
        BRepTool::range(&edge)?;
        let c2d_new = c.copy()?.cast::<dyn Geom2dCurve>();
        let c2d1_new = match c2d1.get() {
            Some(c1) => c1.copy()?.cast::<dyn Geom2dCurve>(),
            None => Handle::default(),
        };
        if !is_seam && c2d1_new.is_null() {
            builder.update_edge_pcurve_on_face(&edge, &c2d_new, face, tol_edge)?;
            continue;
        }
        let mut ori = edge.orientation();
        if face.orientation() == Orientation::Reversed {
            ori = top_abs::reverse(ori);
        }
        let pair = if ori == Orientation::Forward {
            (&c2d_new, &c2d1_new)
        } else {
            (&c2d1_new, &c2d_new)
        };
        builder.update_edge_pcurves_on_face(&edge, pair, face, tol_edge)?;
    }
    Ok(())
}
