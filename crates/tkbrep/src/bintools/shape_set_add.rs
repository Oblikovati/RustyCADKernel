// Created on: 2004-05-11
// Created by: Sergey ZARITCHNY <szy@opencascade.com>
// Copyright (c) 2004-2014 OPEN CASCADE SAS
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
// Rust translation of BinTools_ShapeSet::AddShape (BinTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Collecting the geometry of one shape into the tables of the binary shape set.
use super::shape_set::BinShapeSet;
use super::shape_set_base::BinShapeSetBase;
use crate::brep::access::{edge_tshape, face_tshape, vertex_tshape};
use crate::brep::{BRepCurveRepresentation, BRepPointRepresentation};
use crate::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

impl BinShapeSet {
    /// `AddShape(S)`: adds the geometry of a vertex (the curves and surfaces of its
    /// point representations), an edge (its curves, p-curves, surfaces and, with
    /// triangles, its polygons and triangulations) or a face (its surface and, with
    /// triangles or without a surface, its triangulation), and their locations. The
    /// native code casts the TShape without a check; the port raises
    /// Standard_TypeMismatch for a TShape of another class; `set.add_shape(&shape)?`.
    pub fn add_shape(&mut self, shape: &Shape) -> Result<(), Failure> {
        match shape.shape_type()? {
            ShapeEnum::Vertex => self.add_vertex_geometry(shape),
            ShapeEnum::Edge => self.add_edge_geometry(shape),
            ShapeEnum::Face => self.add_face_geometry(shape),
            _ => Ok(()),
        }
    }

    fn add_vertex_geometry(&mut self, shape: &Shape) -> Result<(), Failure> {
        let tvertex = vertex_tshape(shape, "BinTools_ShapeSet::AddShape")?;
        for point in tvertex.points() {
            let Some(point) = point.get() else { continue };
            self.add_point_geometry(point)?;
            self.locations.add(&point.location());
        }
        Ok(())
    }

    fn add_point_geometry(&mut self, point: &dyn BRepPointRepresentation) -> Result<(), Failure> {
        if point.is_point_on_curve() {
            self.curves.add(&point.curve()?);
        } else if point.is_point_on_curve_on_surface() {
            self.curves2d.add(&point.pcurve()?);
            self.surfaces.add(&point.surface()?);
        } else if point.is_point_on_surface() {
            self.surfaces.add(&point.surface()?);
        }
        Ok(())
    }

    fn add_edge_geometry(&mut self, shape: &Shape) -> Result<(), Failure> {
        let tedge = edge_tshape(shape, "BinTools_ShapeSet::AddShape")?;
        for representation in tedge.curves() {
            let Some(representation) = representation.get() else {
                continue;
            };
            self.add_curve_geometry(representation)?;
        }
        Ok(())
    }

    fn add_curve_geometry(&mut self, curve: &dyn BRepCurveRepresentation) -> Result<(), Failure> {
        if curve.is_curve3d() {
            let curve3d = curve.curve3d()?;
            if !curve3d.is_null() {
                self.curves.add(&curve3d);
                self.locations.add(&curve.location());
            }
        } else if curve.is_curve_on_surface() {
            self.surfaces.add(&curve.surface()?);
            self.curves2d.add(&curve.pcurve()?);
            self.locations.add(&curve.location());
            if curve.is_curve_on_closed_surface() {
                self.curves2d.add(&curve.pcurve2()?);
            }
        } else if curve.is_regularity() {
            self.surfaces.add(&curve.surface()?);
            self.locations.add(&curve.location());
            self.surfaces.add(&curve.surface2()?);
            self.locations.add(&curve.location2()?);
        } else if self.is_with_triangles() {
            self.add_mesh_geometry(curve)?;
        }
        Ok(())
    }

    /// The polygons of an edge, collected only with triangles. A triangulation reached
    /// from an edge is added without normals; a face met earlier (`Add` handles the
    /// parents first) keeps the flag it set.
    fn add_mesh_geometry(&mut self, curve: &dyn BRepCurveRepresentation) -> Result<(), Failure> {
        if curve.is_polygon3d() {
            let polygon = curve.polygon3d()?;
            if !polygon.is_null() {
                self.polygons3d.add(&polygon);
                self.locations.add(&curve.location());
            }
        } else if curve.is_polygon_on_triangulation() {
            self.triangulations.add(&curve.triangulation()?, false);
            self.nodes.add(&curve.polygon_on_triangulation()?);
            self.locations.add(&curve.location());
            if curve.is_polygon_on_closed_triangulation() {
                self.nodes.add(&curve.polygon_on_triangulation2()?);
            }
        } else if curve.is_polygon_on_surface() {
            self.surfaces.add(&curve.surface()?);
            self.polygons2d.add(&curve.polygon()?);
            self.locations.add(&curve.location());
            if curve.is_polygon_on_closed_surface() {
                self.polygons2d.add(&curve.polygon2()?);
            }
        }
        Ok(())
    }

    /// The surface, the triangulation (with triangles, or always for a face without a
    /// surface, whose normals are then always kept) and the face location.
    fn add_face_geometry(&mut self, shape: &Shape) -> Result<(), Failure> {
        let tface = face_tshape(shape, "BinTools_ShapeSet::AddShape")?;
        let surface = tface.surface();
        let mut need_normals = self.is_with_normals();
        if surface.is_null() {
            need_normals = true;
        } else {
            self.surfaces.add(&surface);
        }
        if self.is_with_triangles() || surface.is_null() {
            let triangulation = tface.active_triangulation();
            if !triangulation.is_null() {
                self.triangulations.add(&triangulation, need_normals);
            }
        }
        self.locations.add(&tface.location());
        Ok(())
    }
}
