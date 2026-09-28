// Created on: 1995-03-06
// Created by: Laurent PAINNOT
// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/Poly/Poly_Polygon3D.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{
    HArray1OfReal, errors::array_failure, owned::owned_array, shared_state::SharedState,
    triangulation::dump_size,
};
use crate::gp::Point3;
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{
        Failure, Handle, StandardTransient,
        dump::{self, DumpOutput},
    },
};
use std::io;
struct Polygon3DFields {
    deflection: f64,
    nodes: Array1<'static, Point3>,
    parameters: Option<Handle<HArray1OfReal>>,
}
/// `Poly_Polygon3D`: one-based 3D nodes with optional curve parameters; `Polygon3D::new(4, true)?`.
pub struct Polygon3D {
    base: StandardTransient,
    fields: SharedState<Polygon3DFields>,
}
rustycad_tkernel::impl_transient!(Polygon3D, "Poly_Polygon3D", StandardTransient, base);
impl Polygon3D {
    /// Allocates nodes and, if asked, parameters at `1..=nb_nodes`; `Polygon3D::new(4, true)?`.
    pub fn new(nb_nodes: i32, has_parameters: bool) -> Result<Self, Failure> {
        const MEMBER: &str = "Poly_Polygon3D::Poly_Polygon3D";
        let nodes = Array1::with_bounds(1, nb_nodes).map_err(|e| array_failure(MEMBER, e))?;
        let parameters = if has_parameters {
            Some(Handle::new(HArray1OfReal::with_bounds(1, nb_nodes)?))
        } else {
            None
        };
        Ok(Self::from_fields(nodes, parameters))
    }
    /// Copies nodes of any bounds to `1..=len`; `Polygon3D::from_nodes(&nodes)`.
    pub fn from_nodes(nodes: &Array1<'_, Point3>) -> Self {
        Self::from_fields(owned_array(nodes.as_slice(), 1), None)
    }
    /// Copies nodes and parameters. As natively, the parameter read for a node uses the
    /// node's own index, so both tables must share bounds; `Polygon3D::from_nodes_and_parameters(&n, &p)?`.
    pub fn from_nodes_and_parameters(
        nodes: &Array1<'_, Point3>,
        parameters: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        let values = HArray1OfReal::with_bounds(1, parameters.len() as i32)?;
        for (offset, index) in (nodes.lower()..=nodes.upper()).enumerate() {
            let parameter = parameters
                .value(index)
                .map_err(|e| array_failure("Poly_Polygon3D::Poly_Polygon3D", e))?;
            values.set_value(offset as i32 + 1, *parameter)?;
        }
        Ok(Self::from_fields(
            owned_array(nodes.as_slice(), 1),
            Some(Handle::new(values)),
        ))
    }
    fn from_fields(
        nodes: Array1<'static, Point3>,
        parameters: Option<Handle<HArray1OfReal>>,
    ) -> Self {
        Self {
            base: StandardTransient::default(),
            fields: SharedState::new(Polygon3DFields {
                deflection: 0.0,
                nodes,
                parameters,
            }),
        }
    }
    /// A new handle with copied nodes, parameters and deflection. The copy allocates one
    /// parameter per node, so a parameter table of another length raises
    /// `Standard_DimensionMismatch`, as natively; `polygon.copy()?`.
    pub fn copy(&self) -> Result<Handle<Self>, Failure> {
        let fields = self.fields.read();
        let copy = Self::new(fields.nodes.len() as i32, fields.parameters.is_some())?;
        copy.change_nodes(|nodes| nodes.copy_values(&fields.nodes))
            .map_err(|e| array_failure("Poly_Polygon3D::Copy", e))?;
        if let Some(source) = &fields.parameters {
            let values = source.array1();
            copy.change_parameters(|target| target.copy_values(&values))?
                .map_err(|e| array_failure("Poly_Polygon3D::Copy", e))?;
        }
        copy.set_deflection(fields.deflection);
        Ok(Handle::new(copy))
    }
    /// Returns the deflection; `polygon.deflection()`.
    pub fn deflection(&self) -> f64 {
        self.fields.read().deflection
    }
    /// Sets the deflection; `polygon.set_deflection(0.01)`.
    pub fn set_deflection(&self, deflection: f64) {
        self.fields.write().deflection = deflection;
    }
    /// Number of nodes, closing node included; `polygon.nb_nodes()`.
    pub fn nb_nodes(&self) -> i32 {
        self.fields.read().nodes.len() as i32
    }
    /// Copy of the node table, as `Nodes()`; `polygon.nodes()`.
    pub fn nodes(&self) -> Array1<'static, Point3> {
        self.fields.read().nodes.clone()
    }
    /// Lends the node table for editing, as `ChangeNodes()`; `polygon.change_nodes(|n| n.len())`.
    pub fn change_nodes<R>(&self, edit: impl FnOnce(&mut Array1<'static, Point3>) -> R) -> R {
        edit(&mut self.fields.write().nodes)
    }
    /// True when parameters are stored; `polygon.has_parameters()`.
    pub fn has_parameters(&self) -> bool {
        self.fields.read().parameters.is_some()
    }
    /// Copy of the parameters; natively a null dereference without them, here
    /// `Standard_NullObject`; `polygon.parameters()?`.
    pub fn parameters(&self) -> Result<Array1<'static, f64>, Failure> {
        Ok(self.shared_parameters()?.array1())
    }
    /// Lends the shared parameter table for editing, as `ChangeParameters()`;
    /// `polygon.change_parameters(|p| p.init(0.0))?`.
    pub fn change_parameters<R>(
        &self,
        edit: impl FnOnce(&mut Array1<'static, f64>) -> R,
    ) -> Result<R, Failure> {
        Ok(self.shared_parameters()?.change_array1(edit))
    }
    fn shared_parameters(&self) -> Result<Handle<HArray1OfReal>, Failure> {
        self.fields.read().parameters.clone().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "Poly_Polygon3D::Parameters: no parameters; expected a polygon built with parameters",
                "",
            )
        })
    }
    /// Writes the class name, deflection and table sizes; `polygon.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        let fields = self.fields.read();
        dump::dump_class_begin(output, "Poly_Polygon3D")?;
        dump::dump_real(output, "Deflection", fields.deflection)?;
        dump_size(output, "Nodes.Size", fields.nodes.len())?;
        if let Some(parameters) = &fields.parameters {
            dump_size(output, "Parameters->Size", parameters.len())?;
        }
        Ok(())
    }
}
