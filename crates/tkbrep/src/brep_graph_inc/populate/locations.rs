// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of applyDefinitionLocation, applyDefinitionLocationToPolygon3D,
// invalidateTriangulationCachedMinMax and applyDefinitionLocationToTriangulation of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Populate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The definition-frame transforms of population: geometry is stored with the TShape
//! location baked in, while the use locations stay on the references.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{Polygon3D, Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;

/// `applyDefinitionLocation(theGeom, theLocation)` for a curve: a transformed copy, or
/// the same handle for a null curve or an identity location.
pub(super) fn located_curve(
    curve: &Handle<dyn GeomCurve>,
    location: &Location,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let Some(geometry) = curve.get() else {
        return Ok(curve.clone());
    };
    if location.is_identity() {
        return Ok(curve.clone());
    }
    Ok(geometry
        .transformed(location.transformation())?
        .cast::<dyn GeomCurve>())
}

/// `applyDefinitionLocation(theGeom, theLocation)` for a surface.
pub(super) fn located_surface(
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let Some(geometry) = surface.get() else {
        return Ok(surface.clone());
    };
    if location.is_identity() {
        return Ok(surface.clone());
    }
    Ok(geometry
        .transformed(location.transformation())?
        .cast::<dyn GeomSurface>())
}

/// `applyDefinitionLocation(thePoint, theLocation)`.
pub(super) fn located_point(point: Point3, location: &Location) -> Point3 {
    if location.is_identity() {
        point
    } else {
        point.transformed(location.transformation())
    }
}

/// `applyDefinitionLocationToPolygon3D`: a copy with transformed nodes, the same
/// parameters and deflection; the same handle for a null polygon or identity.
pub(super) fn located_polygon3d(
    polygon: &Handle<Polygon3D>,
    location: &Location,
) -> Result<Handle<Polygon3D>, Failure> {
    let Some(source) = polygon.get() else {
        return Ok(polygon.clone());
    };
    if location.is_identity() {
        return Ok(polygon.clone());
    }
    let transform = location.transformation();
    let nodes: Vec<Point3> = source
        .nodes()
        .iter()
        .map(|p| p.transformed(transform))
        .collect();
    let nodes = Array1::from_shared(&nodes, 0);
    let copy = if source.has_parameters() {
        let parameters: Vec<f64> = source.parameters()?.iter().copied().collect();
        Polygon3D::from_nodes_and_parameters(&nodes, &Array1::from_shared(&parameters, 0))?
    } else {
        Polygon3D::from_nodes(&nodes)
    };
    copy.set_deflection(source.deflection());
    Ok(Handle::new(copy))
}

/// `applyDefinitionLocationToTriangulation`: always a copy; a non-identity location
/// transforms the nodes and normals, scales the deflection, flips the triangles of a
/// mirroring transform and drops the cached range.
pub(super) fn located_triangulation(
    triangulation: &Handle<Triangulation>,
    location: &Location,
) -> Result<Handle<Triangulation>, Failure> {
    let Some(source) = triangulation.get() else {
        return Ok(triangulation.clone());
    };
    let copy = source.copy();
    if location.is_identity() {
        return Ok(copy);
    }
    let mesh = copy.get().expect("a copied triangulation is not null");
    let transform = location.transformation();
    mesh.set_deflection(mesh.deflection() * transform.scale_factor().abs());
    for index in 1..=mesh.nb_nodes() {
        mesh.set_node(index, mesh.node(index)?.transformed(transform))?;
    }
    if mesh.has_normals() {
        for index in 1..=mesh.nb_nodes() {
            mesh.set_normal(index, mesh.normal(index)?.transformed(transform))?;
        }
    }
    if transform.is_negative() {
        for index in 1..=mesh.nb_triangles() {
            let (n1, n2, n3) = mesh.triangle(index)?.get();
            mesh.set_triangle(index, Triangle::new(n1, n3, n2))?;
        }
    }
    // A void box is the public cache-invalidation signal of Poly_Triangulation.
    if mesh.has_cached_min_max() {
        mesh.set_cached_min_max(&BoundingBox3::default());
    }
    Ok(copy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::gp::{Transform3, Vector3};

    fn translation() -> Location {
        let mut transform = Transform3::default();
        transform.set_translation(Vector3::new(1.0, 2.0, 3.0));
        Location::new(transform)
    }

    #[test]
    fn points_and_polygons_move_to_the_definition_frame() {
        let point = located_point(Point3::new(0.0, 0.0, 0.0), &translation());
        assert_eq!((point.x(), point.y(), point.z()), (1.0, 2.0, 3.0));
        let nodes = [Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
        let polygon = Handle::new(Polygon3D::from_nodes(&Array1::from_shared(&nodes, 1)));
        let moved = located_polygon3d(&polygon, &translation()).unwrap();
        assert_ne!(moved.object_address(), polygon.object_address());
        let same = located_polygon3d(&polygon, &Location::default()).unwrap();
        assert_eq!(same.object_address(), polygon.object_address());
        assert!(
            located_curve(&Handle::default(), &translation())
                .unwrap()
                .is_null()
        );
    }
}
