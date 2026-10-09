// Rust translation of BRepAlgo_Loop::UpdateVEmap of ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo_Loop.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertex => edges map updated with the vertex images: the free vertices that are
//! images of a same vertex merged into one at the centre of their points.
use super::BRepAlgoLoop;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::gp::Point3;

/// The centre of the points of the vertices (GeomLib::AxeOfInertia) and the tolerance
/// covering them.
fn centre_and_tolerance(vlist: &List<Shape>) -> Result<(Point3, f64), Failure> {
    let mut max_tol: f64 = 0.;
    let mut points: Vec<Point3> = Vec::with_capacity(vlist.len());
    for v in vlist.iter() {
        let vertex = topods::vertex(v)?;
        max_tol = max_tol.max(BRepTool::vertex_tolerance(&vertex)?);
        points.push(BRepTool::pnt(&vertex)?);
    }
    let mut buffer = points.clone();
    let upper = i32::try_from(buffer.len()).map_err(|_| {
        Failure::new(&format!(
            "UpdateVEmap: {} points; expected an i32 count",
            points.len()
        ))
    })?;
    let array = Array1::from_slice_bounds(&mut buffer, 1, upper)
        .map_err(|e| Failure::new(&format!("UpdateVEmap: points array: {e:?}")))?;
    let centre = geom_lib::axe_of_inertia(&array, 1e-7)?.axe.location();
    let max_sq_dist = points
        .iter()
        .fold(0., |m: f64, p| m.max(centre.square_distance(p)));
    Ok((centre, max_tol.max(max_sq_dist.sqrt())))
}

impl BRepAlgoLoop {
    /// The free vertices (on one edge) that are images, grouped by the vertex they are
    /// the image of (FORWARD).
    fn group_images(
        &self,
        the_ve_map: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
        let mut ver_lver = IndexedDataMapOfShapeListOfShape::default();
        for ii in 1..=the_ve_map.len() {
            let a_vertex = the_ve_map.find_key(ii)?;
            if the_ve_map.find_from_index(ii)?.len() != 1 || !self.my_image_vv.is_image(a_vertex) {
                continue;
            }
            let a_pro_vertex = self.my_image_vv.image_from(a_vertex)?;
            if !ver_lver.contains(&a_pro_vertex) {
                ver_lver.add(a_pro_vertex.clone(), List::default());
            }
            ver_lver
                .change_from_key(&a_pro_vertex)?
                .append(a_vertex.oriented(Orientation::Forward));
        }
        Ok(ver_lver)
    }

    /// The vertex of the group kept: the first one on a constant edge, else the first.
    fn constant_vertex(
        &self,
        vlist: &List<Shape>,
        the_ve_map: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Shape, Failure> {
        for v in vlist.iter() {
            let an_edge = the_ve_map
                .find_from_key(v)?
                .first()
                .cloned()
                .unwrap_or_default();
            if self.my_const_edges.iter().any(|c| an_edge.is_same(c)) {
                return Ok(v.clone());
            }
        }
        Ok(vlist.first().cloned().unwrap_or_default())
    }

    /// The group merged: the kept vertex moved to the centre with the covering
    /// tolerance, and put in place of the others on their edges.
    fn merge_group(
        &self,
        vlist: &List<Shape>,
        the_ve_map: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<(), Failure> {
        let (centre, max_tol) = centre_and_tolerance(vlist)?;
        let const_vertex = self.constant_vertex(vlist, the_ve_map)?;
        let bb = BRepBuilder::default();
        bb.update_vertex_point(&topods::vertex(&const_vertex)?, centre, max_tol)?;
        for v in vlist.iter() {
            if v.is_same(&const_vertex) {
                continue;
            }
            let mut an_edge = the_ve_map
                .find_from_key(v)?
                .first()
                .cloned()
                .unwrap_or_default();
            an_edge.set_orientation(Orientation::Forward);
            let (v1, v2): (Vertex, Vertex) = topexp::vertices(&topods::edge(&an_edge)?, false);
            let to_remove = if v1.as_ref().is_same(v) { v1 } else { v2 };
            an_edge.set_free(true)?;
            bb.remove(&mut an_edge, to_remove.as_ref())?;
            bb.add(
                &mut an_edge,
                &const_vertex.oriented(to_remove.as_ref().orientation()),
            )?;
        }
        Ok(())
    }

    /// `UpdateVEmap(theVEmap)`: the free image vertices of a same vertex merged, then
    /// the map rebuilt from its edges (once each, IsSame); `l.update_ve_map(&mut mve)?`.
    pub fn update_ve_map(
        &self,
        the_ve_map: &mut IndexedDataMapOfShapeListOfShape,
    ) -> Result<(), Failure> {
        let ver_lver = self.group_images(the_ve_map)?;
        if ver_lver.is_empty() {
            return Ok(());
        }
        for ii in 1..=ver_lver.len() {
            let vlist = ver_lver.find_from_index(ii)?;
            if vlist.len() != 1 {
                self.merge_group(vlist, the_ve_map)?;
            }
        }
        let mut emap = IndexedMapOfShape::default();
        for ii in 1..=the_ve_map.len() {
            for e in the_ve_map.find_from_index(ii)?.iter() {
                emap.add(e.clone());
            }
        }
        the_ve_map.clear(false);
        for ii in 1..=emap.len() {
            topexp::map_shapes_and_ancestors(
                emap.find_key(ii)?,
                ShapeEnum::Vertex,
                ShapeEnum::Edge,
                the_ve_map,
            )?;
        }
        Ok(())
    }
}
