// Created on: 1996-02-20
// Created by: Laurent PAINNOT
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
// Rust translation of FoundationClasses/TKMath/Poly/Poly_PolygonOnTriangulation.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::triangulation::dump_size;
use rustycad_tkernel::{
    ncollection::{Array1, HArray1OfReal},
    standard::{
        Failure, Handle, SharedState, StandardTransient,
        dump::{self, DumpOutput},
    },
};
use std::io;
struct PolygonOnTriangulationFields {
    deflection: f64,
    nodes: Array1<'static, i32>,
    parameters: Handle<HArray1OfReal>,
}
/// `Poly_PolygonOnTriangulation`: node indices into a triangulation with optional
/// parameters; `PolygonOnTriangulation::new(4, false)?`.
pub struct PolygonOnTriangulation {
    base: StandardTransient,
    fields: SharedState<PolygonOnTriangulationFields>,
}
rustycad_tkernel::impl_transient!(
    PolygonOnTriangulation,
    "Poly_PolygonOnTriangulation",
    StandardTransient,
    base
);
const NO_PARAMETERS: &str = "Poly_PolygonOnTriangulation::Parameter : parameters is NULL";
impl PolygonOnTriangulation {
    /// Allocates node indices and, if asked, parameters at `1..=nb_nodes`;
    /// `PolygonOnTriangulation::new(4, true)?`.
    pub fn new(nb_nodes: i32, has_parameters: bool) -> Result<Self, Failure> {
        let nodes = Array1::with_bounds(1, nb_nodes)
            .map_err(|e| e.into_failure("Poly_PolygonOnTriangulation"))?;
        let parameters = if has_parameters {
            Handle::new(HArray1OfReal::with_bounds(1, nb_nodes)?)
        } else {
            Handle::default()
        };
        Ok(Self::from_fields(nodes, parameters))
    }
    /// Copies node indices; the table keeps the input's bounds, because the native
    /// constructor assigns it with `NCollection_Array1::Assign`;
    /// `PolygonOnTriangulation::from_nodes(&indices)`.
    pub fn from_nodes(nodes: &Array1<'_, i32>) -> Self {
        let mut table = Array1::default();
        table.assign(nodes);
        Self::from_fields(table, Handle::default())
    }
    /// Copies node indices and parameters, each keeping its input's bounds;
    /// `PolygonOnTriangulation::from_nodes_and_parameters(&indices, &values)`.
    pub fn from_nodes_and_parameters(
        nodes: &Array1<'_, i32>,
        parameters: &Array1<'_, f64>,
    ) -> Self {
        let polygon = Self::from_nodes(nodes);
        let mut values = Array1::default();
        values.assign(parameters);
        polygon.fields.write().parameters = HArray1OfReal::shared(values);
        polygon
    }
    fn from_fields(nodes: Array1<'static, i32>, parameters: Handle<HArray1OfReal>) -> Self {
        Self {
            base: StandardTransient::default(),
            fields: SharedState::new(PolygonOnTriangulationFields {
                deflection: 0.0,
                nodes,
                parameters,
            }),
        }
    }
    /// A new handle with copied indices, parameters and deflection; tables of different
    /// lengths raise `Standard_DimensionMismatch`, as natively; `polygon.copy()?`.
    pub fn copy(&self) -> Result<Handle<Self>, Failure> {
        let fields = self.fields.read();
        let copy = Self::new(fields.nodes.len() as i32, fields.parameters.is_some())?;
        copy.change_node_array(|nodes| nodes.copy_values(&fields.nodes))
            .map_err(|e| e.into_failure("Poly_PolygonOnTriangulation::Copy"))?;
        if let Some(source) = fields.parameters.get() {
            let values = source.array1();
            copy.change_parameter_array(|target| target.copy_values(&values))?
                .map_err(|e| e.into_failure("Poly_PolygonOnTriangulation::Copy"))?;
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
    /// Number of node indices; `polygon.nb_nodes()`.
    pub fn nb_nodes(&self) -> i32 {
        self.fields.read().nodes.len() as i32
    }
    /// Node index at a table index; `polygon.node(1)?`.
    pub fn node(&self, index: i32) -> Result<i32, Failure> {
        let fields = self.fields.read();
        let node = fields
            .nodes
            .value(index)
            .map_err(|e| e.into_failure("Poly_PolygonOnTriangulation::Node"))?;
        Ok(*node)
    }
    /// Sets the node index at a table index; `polygon.set_node(1, 7)?`.
    pub fn set_node(&self, index: i32, node: i32) -> Result<(), Failure> {
        self.fields
            .write()
            .nodes
            .set_value(index, node)
            .map_err(|e| e.into_failure("Poly_PolygonOnTriangulation::SetNode"))
    }
    /// Lends the node table, as `ChangeNodeArray()` and deprecated `ChangeNodes()`;
    /// `polygon.change_node_array(|nodes| nodes.len())`.
    pub fn change_node_array<R>(&self, edit: impl FnOnce(&mut Array1<'static, i32>) -> R) -> R {
        edit(&mut self.fields.write().nodes)
    }
    /// Copy of the node table, as `Nodes()`; `polygon.nodes()`.
    pub fn nodes(&self) -> Array1<'static, i32> {
        self.fields.read().nodes.clone()
    }
    /// True when parameters are stored; `polygon.has_parameters()`.
    pub fn has_parameters(&self) -> bool {
        self.fields.read().parameters.is_some()
    }
    /// The shared parameter handle, null without parameters; `polygon.parameters()`.
    pub fn parameters(&self) -> Handle<HArray1OfReal> {
        self.fields.read().parameters.clone()
    }
    /// Parameter at a table index, `Standard_NullObject` without parameters; `polygon.parameter(1)?`.
    pub fn parameter(&self, index: i32) -> Result<f64, Failure> {
        self.shared_parameters()?.value(index)
    }
    /// Sets the parameter at a table index; `polygon.set_parameter(1, 0.5)?`.
    pub fn set_parameter(&self, index: i32, value: f64) -> Result<(), Failure> {
        self.shared_parameters()?.set_value(index, value)
    }
    /// Lends the parameter table, as `ChangeParameterArray()` and deprecated
    /// `ChangeParameters()`; `polygon.change_parameter_array(|p| p.len())?`.
    pub fn change_parameter_array<R>(
        &self,
        edit: impl FnOnce(&mut Array1<'static, f64>) -> R,
    ) -> Result<R, Failure> {
        Ok(self.shared_parameters()?.change_array1(edit))
    }
    /// Shares a parameter handle whose bounds must equal the node table's, else
    /// `Standard_OutOfRange`; a null handle removes them; `polygon.set_parameters(values)?`.
    pub fn set_parameters(&self, parameters: Handle<HArray1OfReal>) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        if let Some(values) = parameters.get() {
            let bounds = (values.lower(), values.upper());
            let expected = (fields.nodes.lower(), fields.nodes.upper());
            if bounds != expected {
                return Err(Failure::out_of_range(&format!(
                    "Poly_PolygonOnTriangulation::SetParameters() - invalid array size {bounds:?}; expected {expected:?}"
                )));
            }
        }
        fields.parameters = parameters;
        Ok(())
    }
    fn shared_parameters(&self) -> Result<Handle<HArray1OfReal>, Failure> {
        let parameters = self.fields.read().parameters.clone();
        if parameters.is_null() {
            return Err(Failure::with_type("Standard_NullObject", NO_PARAMETERS, ""));
        }
        Ok(parameters)
    }
    /// Writes the class name, deflection and table sizes; `polygon.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        let fields = self.fields.read();
        dump::dump_class_begin(output, "Poly_PolygonOnTriangulation")?;
        dump::dump_real(output, "Deflection", fields.deflection)?;
        dump_size(output, "Nodes.Size", fields.nodes.len())?;
        if let Some(parameters) = fields.parameters.get() {
            dump_size(output, "Parameters->Size", parameters.len())?;
        }
        Ok(())
    }
}
