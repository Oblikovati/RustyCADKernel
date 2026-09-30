// Rust translation of BRepTools_Modifier::CreateNewVertices, FillNewCurveInfo,
// FillNewSurfaceInfo and CreateOtherVertices
// (ModelingData/TKBRep/BRepTools/BRepTools_Modifier.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry the modification gives, collected before the shape is rebuilt.
use super::{Modifier, set_shape_flags};
use crate::brep::BRepBuilder;
use crate::brep_tools::modification::{BRepToolsModification, NewCurve, NewSurface};
use crate::topexp::Explorer;
use crate::topods::{self, Face, Shape, Vertex};
use crate::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;

impl Modifier {
    /// `CreateNewVertices(theMVE, M)`: a new vertex for each vertex the modification
    /// moves (flags copied); with a mutable input, the others map to themselves.
    pub(super) fn create_new_vertices(
        &mut self,
        mve: &IndexedDataMapOfShapeListOfShape,
        m: &dyn BRepToolsModification,
    ) -> Result<(), Failure> {
        let (mut point, mut tolerance) = (Point3::default(), 0.0);
        for index in 1..=mve.extent() {
            let vertex = topods::vertex(mve.find_key(index)?)?;
            if m.new_point(&vertex, &mut point, &mut tolerance)? {
                let mut new_vertex = Vertex::default();
                BRepBuilder::default().make_vertex_at(&mut new_vertex, point, tolerance)?;
                set_shape_flags(&vertex, &new_vertex)?;
                self.store(&vertex, &new_vertex)?;
                self.has_new_geom.add(vertex.into_shape());
            } else if self.mutable_input {
                self.store(&vertex, &vertex.oriented(Orientation::Forward))?;
            }
        }
        Ok(())
    }

    /// `FillNewCurveInfo(theMEF, M)`: the new curve of each edge the modification
    /// changes. The output record is shared by the calls, as the native locals are.
    pub(super) fn fill_new_curve_info(
        &mut self,
        mef: &IndexedDataMapOfShapeListOfShape,
        m: &dyn BRepToolsModification,
    ) -> Result<(), Failure> {
        let mut info = NewCurve::default();
        for index in 1..=mef.extent() {
            let edge = topods::edge(mef.find_key(index)?)?;
            if m.new_curve(&edge, &mut info)? {
                self.nc_info.bind(edge.clone().into_shape(), info.clone());
                self.has_new_geom.add(edge.into_shape());
            }
        }
        Ok(())
    }

    /// `FillNewSurfaceInfo(M)`: the new surface of each face the modification changes;
    /// a face whose edges and vertices keep their geometry too is recorded as not
    /// updated.
    pub(super) fn fill_new_surface_info(
        &mut self,
        m: &dyn BRepToolsModification,
    ) -> Result<(), Failure> {
        let mut faces = IndexedMapOfShape::default();
        crate::topexp::map_shapes_of_type(&self.shape, ShapeEnum::Face, &mut faces)?;
        for index in 1..=faces.extent() {
            let face = topods::face(faces.find_key(index)?)?;
            let mut info = NewSurface::default();
            if m.new_surface(&face, &mut info)? {
                self.ns_info.bind(face.clone().into_shape(), info);
                self.has_new_geom.add(face.into_shape());
            } else if self.sub_shapes_kept(&face)? {
                self.non_upd_face.add(face.into_shape());
            }
        }
        Ok(())
    }

    /// Whether no edge of the face has a new curve and no vertex of its edges has a
    /// result yet.
    fn sub_shapes_kept(&self, face: &Face) -> Result<bool, Failure> {
        for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if self.nc_info.is_bound(&edge) {
                return Ok(false);
            }
            for vertex in Explorer::new(&edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                if !self.result_of(&vertex)?.is_null() {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// `CreateOtherVertices(theMVE, theMEF, M)`: each vertex without a result is
    /// replaced by an empty copy when one of its edges gets a new curve or a new p-curve
    /// on an updated face (asked with an empty copy of the edge), and kept otherwise; the
    /// result is FORWARD.
    pub(super) fn create_other_vertices(
        &mut self,
        mve: &IndexedDataMapOfShapeListOfShape,
        mef: &IndexedDataMapOfShapeListOfShape,
        m: &dyn BRepToolsModification,
    ) -> Result<(), Failure> {
        for index in 1..=mve.extent() {
            let vertex = mve.find_key(index)?.clone();
            if !self.result_of(&vertex)?.is_null() {
                continue;
            }
            let edges: Vec<Shape> = mve.find_from_index(index)?.iter().cloned().collect();
            let mut new_vertex = if self.vertex_to_replace(&edges, mef, m)? {
                vertex.empty_copied()?
            } else {
                vertex.clone()
            };
            new_vertex.set_orientation(Orientation::Forward);
            self.store(&vertex, &new_vertex)?;
        }
        Ok(())
    }

    /// Whether one of `edges` has a non-null new curve, or a new p-curve on a face that
    /// is updated.
    fn vertex_to_replace(
        &self,
        edges: &[Shape],
        mef: &IndexedDataMapOfShapeListOfShape,
        m: &dyn BRepToolsModification,
    ) -> Result<bool, Failure> {
        for shape in edges {
            let edge = topods::edge(shape)?;
            if self
                .nc_info
                .find(&edge)
                .is_ok_and(|info| !info.curve.is_null())
            {
                return Ok(true);
            }
            for face in mef.find_from_key(&edge)?.iter() {
                if self.non_upd_face.contains(face) {
                    continue;
                }
                let face = topods::face(face)?;
                // Some NewCurve2d use the new edge, so a null TShape would raise.
                let dummy = topods::edge(&edge.empty_copied()?)?;
                let (mut curve, mut tolerance) = (Handle::<dyn Geom2dCurve>::default(), 0.0);
                if m.new_curve2d(
                    &edge,
                    &face,
                    &dummy,
                    &Face::default(),
                    &mut curve,
                    &mut tolerance,
                )? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
