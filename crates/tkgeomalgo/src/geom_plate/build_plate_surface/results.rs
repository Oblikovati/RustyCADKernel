// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of the result members of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx (IsDone, Surface,
// SurfInit, Sense, Curves2d, Order, G0Error, G1Error, G2Error) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The results of `GeomPlate_BuildPlateSurface`.
use super::GeomPlateBuildPlateSurface;
use crate::geom_plate::surface::GeomPlateSurface;
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, errors};
use rustycad_tkg3d::geom::GeomSurface;

/// `myInitOrder` or `mySense`, Standard_NullObject when never set (a null
/// dereference natively).
fn table<'a>(table: &'a Option<Vec<i32>>, name: &str) -> Result<&'a [i32], Failure> {
    table.as_deref().ok_or_else(|| {
        errors::null_object(&format!(
            "GeomPlate_BuildPlateSurface: {name} is null; expected a performed builder with curves"
        ))
    })
}

/// `table->Value(index)`, 1-based, Standard_OutOfRange outside it.
fn table_value(table: &[i32], index: i32) -> Result<i32, Failure> {
    let slot = usize::try_from(index - 1).ok().and_then(|i| table.get(i));
    slot.copied().ok_or_else(|| {
        errors::out_of_range(&format!(
            "GeomPlate_BuildPlateSurface: index {index} outside 1..={}",
            table.len()
        ))
    })
}

/// A 1-based `NCollection_HArray1` of `length` default values.
fn harray<T: Default + Clone + Send + Sync + 'static>(length: i32) -> Result<HArray1<T>, Failure> {
    HArray1::with_bounds(1, length).map_err(errors::array_bounds)
}

impl GeomPlateBuildPlateSurface {
    /// Whether the plate is solved (`IsDone`, `Plate_Plate::IsDone`): true after a
    /// Perform without constraint too, as the native test notes. `builder.is_done()`.
    pub fn is_done(&self) -> bool {
        self.plate.is_done()
    }

    /// The plate surface, null until a Perform succeeds (`Surface`);
    /// `builder.surface()`.
    pub fn surface(&self) -> Handle<GeomPlateSurface> {
        self.geom_plate_surface.clone()
    }

    /// The initial surface (`SurfInit`); `builder.surf_init()`.
    pub fn surf_init(&self) -> Handle<dyn GeomSurface> {
        self.surf_init.clone()
    }

    /// The sense of each curve constraint in the order they were added (`Sense`): 1
    /// when the curve is reversed in the contour. Standard_NullObject before Perform.
    /// `builder.sense()?`.
    pub fn sense(&self) -> Result<Handle<HArray1<i32>>, Failure> {
        let nt_curve: i32 = self.nt_lin_cont();
        let sens: HArray1<i32> = harray(nt_curve)?;
        for i in 1..=nt_curve {
            let order: i32 = table_value(table(&self.init_order, "myInitOrder")?, i)?;
            let value: i32 = table_value(table(&self.sense, "mySense")?, order)?;
            sens.change_array1()
                .set_value(i, value)
                .map_err(errors::array_bounds)?;
        }
        Ok(Handle::new(sens))
    }

    /// The 2d curves of the curve constraints on the initial surface, in the order
    /// they were added (`Curves2d`); Standard_NullObject before Perform.
    /// `builder.curves2d()?`.
    pub fn curves2d(&self) -> Result<Handle<HArray1<Handle<dyn Geom2dCurve>>>, Failure> {
        let nt_curve: i32 = self.nt_lin_cont();
        let c2dfin: HArray1<Handle<dyn Geom2dCurve>> = harray(nt_curve)?;
        for i in 1..=nt_curve {
            let order: i32 = table_value(table(&self.init_order, "myInitOrder")?, i)?;
            let curve = self.lin(order)?.curve2d_on_surf()?;
            c2dfin
                .change_array1()
                .set_value(i, curve)
                .map_err(errors::array_bounds)?;
        }
        Ok(Handle::new(c2dfin))
    }

    /// The position after Perform of each curve constraint added (`Order`);
    /// Standard_NullObject before Perform. `builder.order()?`.
    pub fn order(&self) -> Result<Handle<HArray1<i32>>, Failure> {
        let result: HArray1<i32> = harray(self.nt_lin_cont())?;
        for i in 1..=self.nt_lin_cont() {
            let position: i32 = table_value(table(&self.init_order, "myInitOrder")?, i)?;
            result
                .change_array1()
                .set_value(position, i)
                .map_err(errors::array_bounds)?;
        }
        Ok(Handle::new(result))
    }

    /// The greatest distance to the curve constraints at the last check
    /// (`G0Error()`); `builder.g0_error()`.
    pub fn g0_error(&self) -> f64 {
        self.g0_error
    }

    /// The greatest angle to the curve constraints at the last check (`G1Error()`);
    /// `builder.g1_error()`.
    pub fn g1_error(&self) -> f64 {
        self.g1_error
    }

    /// The greatest curvature gap to the curve constraints at the last check
    /// (`G2Error()`); `builder.g2_error()`.
    pub fn g2_error(&self) -> f64 {
        self.g2_error
    }
}
