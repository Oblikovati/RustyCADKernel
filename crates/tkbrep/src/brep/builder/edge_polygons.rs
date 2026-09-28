// Created on: 1991-07-02
// Created by: Remi LEQUETTE
// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edge polygon members of `BRep_Builder`: the 3D polygon, the polygons on
//! triangulations and the 2D polygons on surfaces.
use super::edge::locked_checked_edge;
use super::update_curves::CurveList;
use super::{BRepBuilder, relative_to};
use crate::brep::{
    BRepCurveRepresentation, BRepPolygon3D, BRepPolygonOnClosedSurface,
    BRepPolygonOnClosedTriangulation, BRepPolygonOnSurface, BRepPolygonOnTriangulation, BRepTool,
};
use crate::topods::{Edge, Face, TShape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;

/// Two node arrays on one triangulation (the seam of a closed triangulation).
pub type NodesPair<'a> = (
    &'a Handle<PolygonOnTriangulation>,
    &'a Handle<PolygonOnTriangulation>,
);
/// Two 2D polygons on one surface (the seam of a closed surface).
pub type Polygon2DPair<'a> = (&'a Handle<Polygon2D>, &'a Handle<Polygon2D>);

/// Removes the first representation accepted by `matches`; true when one was removed.
fn remove_first(
    list: &mut CurveList,
    matches: impl Fn(&dyn BRepCurveRepresentation) -> bool,
) -> bool {
    let position = list
        .iter()
        .position(|representation| representation.get().is_some_and(&matches));
    position.map(|index| list.remove(index)).is_some()
}

impl BRepBuilder {
    /// `MakeEdge(E, P)`: an edge with a 3D polygon; `builder.make_edge_with_polygon3d(&mut edge, &polygon)?`.
    pub fn make_edge_with_polygon3d(
        &self,
        edge: &mut Edge,
        polygon: &Handle<Polygon3D>,
    ) -> Result<(), Failure> {
        self.make_edge(edge)?;
        self.update_edge_polygon3d(edge, polygon, &Location::default())
    }

    /// `MakeEdge(E, N, T)` and `MakeEdge(E, N, T, L)`: an edge with a polygon on a
    /// triangulation; `builder.make_edge_on_triangulation(&mut edge, &nodes, &triangulation, &location)?`.
    pub fn make_edge_on_triangulation(
        &self,
        edge: &mut Edge,
        nodes: &Handle<PolygonOnTriangulation>,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> Result<(), Failure> {
        self.make_edge(edge)?;
        self.update_edge_polygon_on_triangulation(edge, nodes, triangulation, location)
    }

    /// `UpdateEdge(E, P)` and `UpdateEdge(E, P, L)`: replaces the 3D polygon (a null one
    /// removes it). Without a 3D polygon a new one is appended, even for a null
    /// polygon, as natively; `builder.update_edge_polygon3d(&edge, &polygon, &location)?`.
    pub fn update_edge_polygon3d(
        &self,
        edge: &Edge,
        polygon: &Handle<Polygon3D>,
        location: &Location,
    ) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::UpdateEdge")?;
        let replaced = tedge.change_curves(|curves| replace_polygon3d(curves, polygon))?;
        if !replaced {
            let relative = relative_to(location, edge)?;
            let representation = BRepPolygon3D::new(polygon, &relative);
            tedge.change_curves(|curves| curves.push(Handle::new(representation).into_cast()));
        }
        tedge.set_modified(true);
        Ok(())
    }

    /// `UpdateEdge(E, N, T)` and `UpdateEdge(E, N, T, L)`: replaces the polygon on
    /// (`triangulation`, `location`); Modified is set only when the list changed;
    /// `builder.update_edge_polygon_on_triangulation(&edge, &nodes, &triangulation, &location)?`.
    pub fn update_edge_polygon_on_triangulation(
        &self,
        edge: &Edge,
        nodes: &Handle<PolygonOnTriangulation>,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::UpdateEdge")?;
        let relative = relative_to(location, edge)?;
        let modified = tedge.change_curves(|curves| {
            let removed = remove_first(curves, |r| {
                r.is_polygon_on_triangulation_at(triangulation, &relative)
            });
            if nodes.is_null() {
                return removed;
            }
            let representation = BRepPolygonOnTriangulation::new(nodes, triangulation, &relative);
            curves.push(Handle::new(representation).into_cast());
            true
        });
        if modified {
            tedge.set_modified(true);
        }
        Ok(())
    }

    /// `UpdateEdge(E, N1, N2, T)` and `UpdateEdge(E, N1, N2, T, L)`: replaces the polygon
    /// on (`triangulation`, `location`) by a seam polygon when both arrays are non-null;
    /// `builder.update_edge_polygons_on_triangulation(&edge, (&n1, &n2), &triangulation, &location)?`.
    pub fn update_edge_polygons_on_triangulation(
        &self,
        edge: &Edge,
        nodes: NodesPair<'_>,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::UpdateEdge")?;
        let relative = relative_to(location, edge)?;
        let modified = tedge.change_curves(|curves| {
            let removed = remove_first(curves, |r| {
                r.is_polygon_on_triangulation_at(triangulation, &relative)
            });
            if nodes.0.is_null() || nodes.1.is_null() {
                return removed;
            }
            let representation =
                BRepPolygonOnClosedTriangulation::new(nodes.0, nodes.1, triangulation, &relative);
            curves.push(Handle::new(representation).into_cast());
            true
        });
        if modified {
            tedge.set_modified(true);
        }
        Ok(())
    }

    /// `UpdateEdge(E, P, F)`: the 2D polygon on the face's located surface;
    /// `builder.update_edge_polygon_on_face(&edge, &polygon, &face)?`.
    pub fn update_edge_polygon_on_face(
        &self,
        edge: &Edge,
        polygon: &Handle<Polygon2D>,
        face: &Face,
    ) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.update_edge_polygon_on_surface(edge, polygon, &surface, &location)
    }

    /// `UpdateEdge(E, P, S, L)`: replaces the 2D polygon on (`surface`, `location`) (a
    /// null one removes it); `builder.update_edge_polygon_on_surface(&edge, &polygon, &surface, &location)?`.
    pub fn update_edge_polygon_on_surface(
        &self,
        edge: &Edge,
        polygon: &Handle<Polygon2D>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::UpdateEdge")?;
        let relative = relative_to(location, edge)?;
        tedge.change_curves(|curves| {
            remove_first(curves, |r| r.is_polygon_on_surface_at(surface, &relative));
            if polygon.is_some() {
                let representation = BRepPolygonOnSurface::new(polygon, surface, &relative);
                curves.push(Handle::new(representation).into_cast());
            }
        });
        tedge.set_modified(true);
        Ok(())
    }

    /// `UpdateEdge(E, P1, P2, F)`: the seam polygons on the face's located surface;
    /// `builder.update_edge_polygons_on_face(&edge, (&p1, &p2), &face)?`.
    pub fn update_edge_polygons_on_face(
        &self,
        edge: &Edge,
        polygons: Polygon2DPair<'_>,
        face: &Face,
    ) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.update_edge_polygons_on_surface(edge, polygons, &surface, &location)
    }

    /// `UpdateEdge(E, P1, P2, S, L)`: replaces the 2D polygon on (`surface`, `location`)
    /// by a seam polygon when both are non-null. The new representation is stored at
    /// the identity location, not at the edge-relative one, as natively;
    /// `builder.update_edge_polygons_on_surface(&edge, (&p1, &p2), &surface, &location)?`.
    pub fn update_edge_polygons_on_surface(
        &self,
        edge: &Edge,
        polygons: Polygon2DPair<'_>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::UpdateEdge")?;
        let relative = relative_to(location, edge)?;
        tedge.change_curves(|curves| {
            remove_first(curves, |r| r.is_polygon_on_surface_at(surface, &relative));
            if polygons.0.is_some() && polygons.1.is_some() {
                let representation = BRepPolygonOnClosedSurface::new(
                    polygons.0,
                    polygons.1,
                    surface,
                    &Location::default(),
                );
                curves.push(Handle::new(representation).into_cast());
            }
        });
        tedge.set_modified(true);
        Ok(())
    }
}

/// The search of `UpdateEdge(E, P, L)`: the first 3D polygon is removed (null
/// `polygon`) or updated; false when the edge has no 3D polygon.
fn replace_polygon3d(curves: &mut CurveList, polygon: &Handle<Polygon3D>) -> Result<bool, Failure> {
    let Some(index) = curves
        .iter()
        .position(|representation| representation.get().is_some_and(|r| r.is_polygon3d()))
    else {
        return Ok(false);
    };
    if polygon.is_null() {
        curves.remove(index);
    } else if let Some(representation) = curves[index].get() {
        representation.set_polygon3d(polygon)?;
    }
    Ok(true)
}
