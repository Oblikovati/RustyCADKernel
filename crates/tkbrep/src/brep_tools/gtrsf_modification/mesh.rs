// Rust translation of BRepTools_GTrsfModification::NewTriangulation, NewPolygon and
// NewPolygonOnTriangulation with the normal helpers
// (ModelingData/TKBRep/BRepTools/BRepTools_GTrsfModification.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The mesh members of `BRepTools_GTrsfModification`.
use super::{GTrsfModification, normalized_vectorial_part, representation_transformation};
use crate::brep::BRepTool;
use crate::brep_tools::trsf_modification::flip_triangles;
use crate::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Direction3, GeneralTransform3, Matrix3, Point3};
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation, mesh_purpose};

/// `normalTransformation`: the cofactor matrix of the normalised vectorial part (rows
/// the cross products of its columns), negated for a negative determinant.
fn normal_transformation(normalized: &Matrix3, is_negative: bool) -> Result<Matrix3, Failure> {
    let (c1, c2, c3) = (
        normalized.column(1)?,
        normalized.column(2)?,
        normalized.column(3)?,
    );
    let mut matrix = Matrix3::from_columns(c2.crossed(c3), c3.crossed(c1), c1.crossed(c2));
    if is_negative {
        matrix *= -1.0;
    }
    Ok(matrix)
}

/// `aGTrsf.Transforms(aP.ChangeCoord())`.
fn transformed(gtrsf: &GeneralTransform3, point: Point3) -> Point3 {
    let mut coordinates = point.xyz();
    gtrsf.transforms(&mut coordinates);
    Point3::new(coordinates.x(), coordinates.y(), coordinates.z())
}

impl GTrsfModification {
    /// `NewTriangulation(F, T)`: a transformed copy of the active triangulation (in the
    /// face frame): nodes transformed, triangles flipped for a negative normalised part,
    /// normals mapped by the normal matrix and rescaled (a collapsed normal keeps its
    /// value), deflection scaled, cached box cleared; false without triangulation.
    pub(super) fn transformed_triangulation(
        &self,
        face: &Face,
        triangulation: &mut Handle<Triangulation>,
    ) -> Result<bool, Failure> {
        let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
        *triangulation = mesh;
        let Some(source) = triangulation.get() else {
            return Ok(false);
        };
        let gtrsf = representation_transformation(&self.gtrsf(), &location, face.location())?;
        let normalized = normalized_vectorial_part(&gtrsf, self.scale());
        let is_negative = normalized.determinant() < 0.0;
        let copy = source.copy();
        let Some(mesh) = copy.get() else {
            return Ok(false);
        };
        mesh.set_cached_min_max(&BoundingBox3::default());
        mesh.set_deflection(mesh.deflection() * self.scale());
        for index in 1..=mesh.nb_nodes() {
            mesh.set_node(index, transformed(&gtrsf, mesh.node(index)?))?;
        }
        if is_negative {
            flip_triangles(mesh)?;
        }
        if mesh.has_normals() {
            transform_normals(mesh, &normal_transformation(&normalized, is_negative)?)?;
        }
        *triangulation = copy;
        Ok(true)
    }

    /// `NewPolygon(E, P)`: a transformed copy of the 3D polygon (in the edge frame) with
    /// a scaled deflection; the parameters are kept; false without polygon.
    pub(super) fn transformed_polygon(
        &self,
        edge: &Edge,
        polygon: &mut Handle<Polygon3D>,
    ) -> Result<bool, Failure> {
        let (found, location) = BRepTool::polygon3d(edge)?;
        *polygon = found;
        let Some(source) = polygon.get() else {
            return Ok(false);
        };
        let gtrsf = representation_transformation(&self.gtrsf(), &location, edge.location())?;
        let copy = source.copy()?;
        let Some(moved) = copy.get() else {
            return Ok(false);
        };
        moved.set_deflection(moved.deflection() * self.scale());
        moved.change_nodes(|nodes| {
            for node in nodes.iter_mut() {
                *node = transformed(&gtrsf, *node);
            }
        });
        *polygon = copy;
        Ok(true)
    }

    /// `NewPolygonOnTriangulation(E, F, P)`: a copy of the polygon on the face's active
    /// triangulation (the deflection kept); false without triangulation or polygon.
    pub(super) fn copied_polygon_on_triangulation(
        &self,
        edge: &Edge,
        face: &Face,
        polygon: &mut Handle<PolygonOnTriangulation>,
    ) -> Result<bool, Failure> {
        let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
        if mesh.is_null() {
            return Ok(false);
        }
        *polygon = BRepTool::polygon_on_triangulation(edge, &mesh, &location)?;
        if let Some(source) = polygon.get() {
            *polygon = source.copy()?;
        }
        Ok(!polygon.is_null())
    }
}

/// Each normal multiplied by the normal matrix and divided by its largest absolute
/// component; a null result keeps the source normal.
fn transform_normals(mesh: &Triangulation, matrix: &Matrix3) -> Result<(), Failure> {
    for index in 1..=mesh.nb_nodes() {
        let mut normal = mesh.normal(index)?.xyz();
        normal *= *matrix;
        let scale = normal.x().abs().max(normal.y().abs()).max(normal.z().abs());
        if scale == 0.0 {
            // A collapsed tangent plane has no transformed normal; the source value stays.
            continue;
        }
        normal /= scale;
        mesh.set_normal(index, Direction3::from_xyz(normal)?)?;
    }
    Ok(())
}
