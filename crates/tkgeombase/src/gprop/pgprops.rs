// Created on: 1992-02-14
// Created by: Jean Claude VAUTHIER
// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GProp/GProp_PGProps.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GProp_PGProps`: the global properties of a weighted set of points.
use super::GProps;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Matrix3, Point3, Xyz};
use std::ops::Deref;

/// `GProp_PGProps`: points with masses (unit by default), accumulated at the
/// origin; the inherited [`GProps`] members are reached through `Deref`.
///
/// ```
/// use rustycad_tkgeombase::gprop::PGProps;
/// use rustycad_tkmath::gp::Point3;
/// let mut points = PGProps::new();
/// points.add_point(Point3::new(0.0, 0.0, 0.0));
/// points.add_point(Point3::new(2.0, 4.0, 6.0));
/// assert_eq!(points.centre_of_mass(), Point3::new(1.0, 2.0, 3.0));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PGProps {
    base: GProps,
}

impl Deref for PGProps {
    type Target = GProps;
    fn deref(&self) -> &GProps {
        &self.base
    }
}

fn domain(message: &str) -> Failure {
    Failure::with_type("Standard_DomainError", message, "")
}

fn dimension(message: &str) -> Failure {
    Failure::with_type("Standard_DimensionError", message, "")
}

// The inertia of a unit mass at `point` with respect to the origin.
fn point_inertia(point: Point3) -> Matrix3 {
    let (x, y, z): (f64, f64, f64) = (point.x(), point.y(), point.z());
    Matrix3::from_columns(
        Xyz::new(y * y + z * z, -x * y, -x * z),
        Xyz::new(-x * y, x * x + z * z, -y * z),
        Xyz::new(-x * z, -y * z, x * x + y * y),
    )
}

// The column-major walk of an Array2 (columns outside, rows inside).
fn column_major<'a, T: Copy>(array: &'a Array2<'_, T>) -> impl Iterator<Item = T> + 'a {
    (array.lower_col()..=array.upper_col())
        .flat_map(move |j| (array.lower_row()..=array.upper_row()).map(move |i| array[(i, j)]))
}

// The first element, Standard_OutOfRange for an empty array.
fn first<'a, T>(array: &'a Array1<'_, T>) -> Result<&'a T, Failure> {
    array.value(array.lower()).map_err(|error| {
        Failure::out_of_range(&format!(
            "GProp_PGProps::Barycentre: empty array ({error}); expected at least one point"
        ))
    })
}

// NCollection_Array2::RowLength() (columns) and ColLength() (rows).
fn lengths<T>(array: &Array2<'_, T>) -> (usize, usize) {
    (array.nb_columns(), array.nb_rows())
}

impl PGProps {
    /// `GProp_PGProps()`: an empty set at the origin; `PGProps::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `GProp_PGProps(Pnts)`: unit masses at `points`; `PGProps::from_points(&points)`.
    pub fn from_points(points: &Array1<'_, Point3>) -> Self {
        let mut props: Self = Self::new();
        points.iter().for_each(|point| props.add_point(*point));
        props
    }

    /// `GProp_PGProps(Pnts)` on a 2D array, walked column by column;
    /// `PGProps::from_points_2d(&grid)`.
    pub fn from_points_2d(points: &Array2<'_, Point3>) -> Self {
        let mut props: Self = Self::new();
        column_major(points).for_each(|point| props.add_point(point));
        props
    }

    /// `GProp_PGProps(Pnts, Density)`: `points` with the masses `density`;
    /// Standard_DomainError for different lengths or a mass `<= gp::Resolution()`;
    /// `PGProps::from_weighted_points(&points, &masses)?`.
    pub fn from_weighted_points(
        points: &Array1<'_, Point3>,
        density: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        if points.len() != density.len() {
            return Err(domain(&format!(
                "GProp_PGProps: points and density arrays have different lengths: {} and {}",
                points.len(),
                density.len()
            )));
        }
        let mut props: Self = Self::new();
        for (point, mass) in points.iter().zip(density.iter()) {
            props.add_weighted_point(*point, *mass)?;
        }
        Ok(props)
    }

    /// `GProp_PGProps(Pnts, Density)` on 2D arrays, walked column by column;
    /// Standard_DomainError for different dimensions or a non-positive mass;
    /// `PGProps::from_weighted_points_2d(&grid, &masses)?`.
    pub fn from_weighted_points_2d(
        points: &Array2<'_, Point3>,
        density: &Array2<'_, f64>,
    ) -> Result<Self, Failure> {
        if lengths(points) != lengths(density) {
            return Err(domain(&format!(
                "GProp_PGProps: points and density arrays have different dimensions: {:?} and {:?} (columns, rows)",
                lengths(points),
                lengths(density)
            )));
        }
        let mut props: Self = Self::new();
        for (point, mass) in column_major(points).zip(column_major(density)) {
            props.add_weighted_point(point, mass)?;
        }
        Ok(props)
    }

    /// `AddPoint(P)`: add a unit mass at `point`; `props.add_point(p)`.
    pub fn add_point(&mut self, point: Point3) {
        let mp: Matrix3 = point_inertia(point);
        let base: &mut GProps = &mut self.base;
        if base.dim == 0.0 {
            base.dim = 1.0;
            base.g = point;
            base.inertia = mp;
            return;
        }
        let new_mass: f64 = base.dim + 1.0;
        let new_x: f64 = (base.g.x() * base.dim + point.x()) / new_mass;
        let new_y: f64 = (base.g.y() * base.dim + point.y()) / new_mass;
        let new_z: f64 = (base.g.z() * base.dim + point.z()) / new_mass;
        base.g.set_coordinates(new_x, new_y, new_z);
        base.dim = new_mass;
        base.inertia += mp;
    }

    /// `AddPoint(P, Density)`: add the mass `density` at `point`;
    /// Standard_DomainError when `density <= gp::Resolution()`;
    /// `props.add_weighted_point(p, 2.0)?`.
    pub fn add_weighted_point(&mut self, point: Point3, density: f64) -> Result<(), Failure> {
        if density <= gp::resolution() {
            return Err(domain(&format!(
                "GProp_PGProps::AddPoint: density = {density}; expected > gp::Resolution()"
            )));
        }
        let mp: Matrix3 = point_inertia(point);
        let base: &mut GProps = &mut self.base;
        if base.dim == 0.0 {
            base.dim = density;
            base.g = point;
            base.inertia = mp * density;
            return Ok(());
        }
        let new_mass: f64 = base.dim + density;
        let new_x: f64 = (base.g.x() * base.dim + point.x() * density) / new_mass;
        let new_y: f64 = (base.g.y() * base.dim + point.y() * density) / new_mass;
        let new_z: f64 = (base.g.z() * base.dim + point.z() * density) / new_mass;
        base.g.set_coordinates(new_x, new_y, new_z);
        base.dim = new_mass;
        base.inertia += mp * density;
        Ok(())
    }

    /// `Barycentre(Pnts)`: the mean of `points`, summed from the first one;
    /// Standard_OutOfRange for an empty array (its first element is read);
    /// `PGProps::barycentre(&points)?`.
    pub fn barycentre(points: &Array1<'_, Point3>) -> Result<Point3, Failure> {
        let mut sum: Xyz = first(points)?.xyz();
        for point in points.iter().skip(1) {
            sum += point.xyz();
        }
        sum /= points.len() as f64;
        Ok(Point3::from(sum))
    }

    /// `Barycentre(Pnts)` on a 2D array, summed from zero column by column;
    /// `PGProps::barycentre_2d(&grid)`.
    pub fn barycentre_2d(points: &Array2<'_, Point3>) -> Point3 {
        let mut sum: Xyz = Xyz::new(0.0, 0.0, 0.0);
        column_major(points).for_each(|point| sum += point.xyz());
        let (row_length, col_length) = lengths(points);
        sum /= (row_length * col_length) as f64;
        Point3::from(sum)
    }

    /// `Barycentre(Pnts, Density, Mass, G)`: the total mass and the weighted
    /// barycentre; Standard_DimensionError for different lengths;
    /// `let (mass, g) = PGProps::weighted_barycentre(&points, &masses)?;`.
    pub fn weighted_barycentre(
        points: &Array1<'_, Point3>,
        density: &Array1<'_, f64>,
    ) -> Result<(f64, Point3), Failure> {
        if points.len() != density.len() {
            return Err(dimension(&format!(
                "GProp_PGProps::Barycentre: points and density arrays have different lengths: {} and {}",
                points.len(),
                density.len()
            )));
        }
        let mut mass: f64 = *first(density)?;
        let mut sum: Xyz = first(points)?.xyz() * mass;
        for (point, weight) in points.iter().zip(density.iter()).skip(1) {
            mass += *weight;
            sum += point.xyz() * *weight;
        }
        sum /= mass;
        Ok((mass, Point3::from(sum)))
    }

    /// `Barycentre(Pnts, Density, Mass, G)` on 2D arrays, summed from zero column
    /// by column; Standard_DimensionError for different dimensions;
    /// `let (mass, g) = PGProps::weighted_barycentre_2d(&grid, &masses)?;`.
    pub fn weighted_barycentre_2d(
        points: &Array2<'_, Point3>,
        density: &Array2<'_, f64>,
    ) -> Result<(f64, Point3), Failure> {
        if lengths(points) != lengths(density) {
            return Err(dimension(&format!(
                "GProp_PGProps::Barycentre: points and density arrays have different dimensions: {:?} and {:?} (columns, rows)",
                lengths(points),
                lengths(density)
            )));
        }
        let mut mass: f64 = 0.0;
        let mut sum: Xyz = Xyz::new(0.0, 0.0, 0.0);
        for (point, weight) in column_major(points).zip(column_major(density)) {
            mass += weight;
            sum += point.xyz() * weight;
        }
        sum /= mass;
        Ok((mass, Point3::from(sum)))
    }
}
