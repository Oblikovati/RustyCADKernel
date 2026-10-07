//! The seam shift of `BOPAlgo_PaveFiller::PerformFF`: a face with a closed edge is moved to
//! the exact intersection point of its edge with an edge of the other face.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::face_edges;
use crate::bop_ds::Pair;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::poly::Triangulation;
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmath::toploc::Location;

/// The data `IsClosedFF` reads for one face. The native code overwrites the surface location
/// with the triangulation location (the face location), and this quirk is kept.
struct ClosedEdgeProbe {
    surface: Handle<dyn GeomSurface>,
    triangulation: Handle<Triangulation>,
    location: Location,
    is_plane: bool,
}

impl ClosedEdgeProbe {
    fn new(face: &Face) -> Result<Self, Failure> {
        let (surface, _) = BRepTool::surface_with_location(face)?;
        let (triangulation, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
        let is_plane = is_plane_ff(&surface);
        Ok(Self {
            surface,
            triangulation,
            location,
            is_plane,
        })
    }

    /// Native static `IsClosedFF`.
    fn is_closed(&self, edge: &Edge) -> Result<bool, Failure> {
        if !self.is_plane && BRepTool::is_closed_on_surface(edge, &self.surface, &self.location)? {
            return Ok(true);
        }
        BRepTool::is_closed_on_triangulation(edge, &self.triangulation, &self.location)
    }
}

/// Native static `IsPlaneFF`: a plane, or an offset or trimmed surface based on a plane.
fn is_plane_ff(surface: &Handle<dyn GeomSurface>) -> bool {
    let offset = surface.cast::<GeomOffsetSurface>();
    if let Some(offset) = offset.get() {
        return offset.basis_surface().cast::<GeomPlane>().is_some();
    }
    let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
    if let Some(trimmed) = trimmed.get() {
        return trimmed.basis_surface().cast::<GeomPlane>().is_some();
    }
    surface.cast::<GeomPlane>().is_some()
}

impl BOPAlgoPaveFiller {
    /// Returns the (possibly moved) faces and the shift distance used as curve tolerance; the
    /// first EE vertex farther from the exact edge/edge point than its tolerance moves the face
    /// owning the closed edge.
    pub(super) fn shift_faces_to_edge_intersections(
        &self,
        faces: &[Face; 2],
        edge_points: &DataMap<Pair, Vec<i32>>,
    ) -> Result<([Face; 2], f64), Failure> {
        let mut shifted = faces.clone();
        let probes = [
            ClosedEdgeProbe::new(&faces[0])?,
            ClosedEdgeProbe::new(&faces[1])?,
        ];
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        for edge1 in face_edges(&faces[0])? {
            let index1 = structure.index(&edge1);
            for edge2 in face_edges(&faces[1])? {
                let index2 = structure.index(&edge2);
                let closed = (probes[0].is_closed(&edge1)?, probes[1].is_closed(&edge2)?);
                if !closed.0 && !closed.1 {
                    continue;
                }
                let Some(vertices) = edge_points.seek(&Pair::new(index1, index2)) else {
                    continue;
                };
                for vertex in vertices {
                    if let Some((transform, shift)) =
                        self.edge_shift(*vertex, &edge1, &edge2, closed.0)?
                    {
                        let target = if closed.0 {
                            &mut shifted[0]
                        } else {
                            &mut shifted[1]
                        };
                        target.move_by(&Location::new(transform), false)?;
                        return Ok((shifted, shift));
                    }
                }
            }
        }
        Ok((shifted, 0.))
    }

    /// The translation between the projections of the vertex on both edges when they are
    /// farther apart than the vertex tolerance.
    fn edge_shift(
        &self,
        vertex: i32,
        edge1: &Edge,
        edge2: &Edge,
        first_closed: bool,
    ) -> Result<Option<(Transform3, f64)>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let shape = Vertex::unchecked(structure.shape(vertex)?.clone());
        let point = BRepTool::pnt(&shape)?;
        let context = self.context_ref()?;
        let projector1 = context.proj_pc(edge1)?;
        let projector2 = context.proj_pc(edge2)?;
        let mut projection1 = projector1.write();
        let mut projection2 = projector2.write();
        projection1.perform(point)?;
        projection2.perform(point)?;
        if projection1.nb_points() == 0 && projection2.nb_points() == 0 {
            return Ok(None);
        }
        let on1 = if projection1.nb_points() > 0 {
            projection1.nearest_point()?
        } else {
            point
        };
        let on2 = if projection2.nb_points() > 0 {
            projection2.nearest_point()?
        } else {
            point
        };
        let shift = on1.distance(&on2);
        if shift <= BRepTool::vertex_tolerance(&shape)? {
            return Ok(None);
        }
        let mut transform = Transform3::default();
        let vector = if first_closed {
            Vector3::between(on1, on2)
        } else {
            Vector3::between(on2, on1)
        };
        transform.set_translation(vector);
        Ok(Some((transform, shift)))
    }
}
