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
// Rust translation of the four BRepGraph::ShapesView::AddWithHistory overloads of
// ModelingData/TKBRep/BRepGraph/BRepGraph_ShapesView.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Shapes().AddWithHistory`: add an algorithm result and import its history into the
//! graph's history layer.
use super::add::ShapesViewMut;
use super::{AddResult, Options, ShapeNodeMap};
use crate::brep_graph::NodeId;
use crate::brep_graph::layer_history::LayerHistory;
use crate::brep_tools::History;
use crate::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

// The options of the overloads without options: no auto product.
fn without_auto_product() -> Options {
    Options {
        create_auto_product: false,
        ..Options::default()
    }
}

impl ShapesViewMut<'_> {
    /// `AddWithHistory(theResultShape, theInputs, theHistory, theOpLabel)`: the options
    /// overload without an auto product.
    pub fn add_with_history(
        &mut self,
        result_shape: &Shape,
        inputs: &ShapeNodeMap,
        history: &Handle<History>,
        label: &AsciiString,
    ) -> Result<AddResult, Failure> {
        self.add_with_history_options(
            result_shape,
            inputs,
            history,
            label,
            &without_auto_product(),
        )
    }

    /// `AddWithHistory(theResultShape, theInputs, theHistory, theOpLabel, theOptions)`:
    /// `Add` (tracking the added nodes when the history is absorbed), then, for a
    /// successful add, the ensured history layer absorbs `history` from the inputs to
    /// the added nodes, unless the layer is disabled or the history null.
    pub fn add_with_history_options(
        &mut self,
        result_shape: &Shape,
        inputs: &ShapeNodeMap,
        history: &Handle<History>,
        label: &AsciiString,
        options: &Options,
    ) -> Result<AddResult, Failure> {
        let layer = self.graph().layer_registry().ensure::<LayerHistory>()?;
        let Some(recorder) = layer.get() else {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "BRepGraph::ShapesView::AddWithHistory - the history layer GUID holds another \
                 layer class (expected BRepGraph_LayerHistory)",
                "",
            ));
        };
        let absorbs = recorder.is_enabled() && history.is_some();
        let build_options = Options {
            track_added_nodes: options.track_added_nodes || absorbs,
            ..*options
        };
        let result = self.add_with_options(result_shape, &build_options)?;
        if result.is_ok() && absorbs {
            recorder.absorb(self.graph(), inputs, &result.added_nodes, history, label)?;
        }
        Ok(result)
    }

    /// `AddWithHistory(theResultShape, theInputRoots, theHistory, theOpLabel)`: the
    /// roots overload without an auto product.
    pub fn add_with_history_roots(
        &mut self,
        result_shape: &Shape,
        input_roots: &[NodeId],
        history: &Handle<History>,
        label: &AsciiString,
    ) -> Result<AddResult, Failure> {
        let options = without_auto_product();
        self.add_with_history_roots_options(result_shape, input_roots, history, label, &options)
    }

    /// `AddWithHistory(theResultShape, theInputRoots, theHistory, theOpLabel,
    /// theOptions)`: the inputs are the sub-shapes of the roots' shapes
    /// (`CollectHistoryInputs`).
    pub fn add_with_history_roots_options(
        &mut self,
        result_shape: &Shape,
        input_roots: &[NodeId],
        history: &Handle<History>,
        label: &AsciiString,
        options: &Options,
    ) -> Result<AddResult, Failure> {
        let mut inputs = ShapeNodeMap::default();
        self.graph()
            .shapes()
            .collect_history_inputs(input_roots, &mut inputs)?;
        self.add_with_history_options(result_shape, &inputs, history, label, options)
    }
}
