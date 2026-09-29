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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_Node.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_Node`: the constraints of the approximation at one (U, V) point.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Point3, Xy};

/// The locked fields of a node.
#[derive(Clone)]
struct NodeState {
    true_points: Array2<'static, Point3>,
    errors: Array2<'static, f64>,
    coord: Xy,
    ord_in_u: i32,
    ord_in_v: i32,
}

/// The values `F(U,V)` and derivatives up to `(UOrder, VOrder)` at the node's
/// coordinates, and the approximation errors on them (`AdvApp2Var_Node`).
///
/// Shared through handles; every member takes `&self`.
/// `let node = AdvApp2VarNode::with_orders(1, 2); node.point(0, 0)?`.
pub struct AdvApp2VarNode {
    parent: StandardTransient,
    state: SharedState<NodeState>,
}
impl_transient!(AdvApp2VarNode, "AdvApp2Var_Node", StandardTransient, parent);

impl Default for AdvApp2VarNode {
    fn default() -> Self {
        Self::new()
    }
}

fn zero_tables(
    iu: i32,
    iv: i32,
) -> Result<(Array2<'static, Point3>, Array2<'static, f64>), Failure> {
    let failure =
        |error: rustycad_tkernel::ncollection::ArrayError| error.into_failure("AdvApp2Var_Node");
    let mut points = Array2::with_bounds(0, iu, 0, iv).map_err(failure)?;
    let mut errors = Array2::with_bounds(0, iu, 0, iv).map_err(failure)?;
    points.init(Point3::new(0.0, 0.0, 0.0));
    errors.init(0.0);
    Ok((points, errors))
}

impl AdvApp2VarNode {
    /// A node of orders (2, 2) at (0, 0) (`AdvApp2Var_Node()`); `AdvApp2VarNode::new()`.
    pub fn new() -> Self {
        Self::with_coord(Xy::new(0.0, 0.0), 2, 2)
            .unwrap_or_else(|_| unreachable!("orders 2 are valid"))
    }

    /// A node of orders `(iu, iv)` at (0, 0); negative orders keep one value slot
    /// (`AdvApp2Var_Node(iu, iv)`, whose tables are `0..max(0, order)`).
    /// `AdvApp2VarNode::with_orders(1, 2)`.
    pub fn with_orders(iu: i32, iv: i32) -> Self {
        let (true_points, errors) = zero_tables(iu.max(0), iv.max(0))
            .unwrap_or_else(|_| unreachable!("non-negative orders"));
        Self::from_state(NodeState {
            true_points,
            errors,
            coord: Xy::new(0.0, 0.0),
            ord_in_u: iu,
            ord_in_v: iv,
        })
    }

    /// A node of orders `(iu, iv)` at `uv` (`AdvApp2Var_Node(UV, iu, iv)`);
    /// Standard_RangeError for an order below -1, whose table bounds are invalid.
    /// `AdvApp2VarNode::with_coord(Xy::new(0.5, 1.0), 1, 1)?`.
    pub fn with_coord(uv: Xy, iu: i32, iv: i32) -> Result<Self, Failure> {
        let (true_points, errors) = zero_tables(iu, iv)?;
        Ok(Self::from_state(NodeState {
            true_points,
            errors,
            coord: uv,
            ord_in_u: iu,
            ord_in_v: iv,
        }))
    }

    fn from_state(state: NodeState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }

    /// The (U, V) coordinates (`Coord`); `node.coord().x()`.
    pub fn coord(&self) -> Xy {
        self.state.read().coord
    }

    /// Moves the node to `(x1, x2)` (`SetCoord`); `node.set_coord(0.5, 1.0)`.
    pub fn set_coord(&self, x1: f64, x2: f64) {
        self.state.write().coord = Xy::new(x1, x2);
    }

    /// The continuity order in U (`UOrder`); `node.u_order()`.
    pub fn u_order(&self) -> i32 {
        self.state.read().ord_in_u
    }

    /// The continuity order in V (`VOrder`); `node.v_order()`.
    pub fn v_order(&self) -> i32 {
        self.state.read().ord_in_v
    }

    /// Stores the derivative `(iu, iv)` of F at the node (`SetPoint`);
    /// Standard_OutOfRange outside the table. `node.set_point(0, 1, p)?`.
    pub fn set_point(&self, iu: i32, iv: i32, pt: Point3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state
            .true_points
            .set_value(iu, iv, pt)
            .map_err(|e| e.into_failure("AdvApp2Var_Node::SetPoint"))
    }

    /// The derivative `(iu, iv)` of F at the node (`Point`); Standard_OutOfRange
    /// outside the table. `node.point(0, 0)?`.
    pub fn point(&self, iu: i32, iv: i32) -> Result<Point3, Failure> {
        let state = self.state.read();
        state
            .true_points
            .value(iu, iv)
            .copied()
            .map_err(|e| e.into_failure("AdvApp2Var_Node::Point"))
    }

    /// Stores the approximation error of the derivative `(iu, iv)` (`SetError`);
    /// `node.set_error(0, 0, 1e-7)?`.
    pub fn set_error(&self, iu: i32, iv: i32, error: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        state
            .errors
            .set_value(iu, iv, error)
            .map_err(|e| e.into_failure("AdvApp2Var_Node::SetError"))
    }

    /// The approximation error of the derivative `(iu, iv)` (`Error`); `node.error(0, 0)?`.
    pub fn error(&self, iu: i32, iv: i32) -> Result<f64, Failure> {
        let state = self.state.read();
        state
            .errors
            .value(iu, iv)
            .copied()
            .map_err(|e| e.into_failure("AdvApp2Var_Node::Error"))
    }

    /// Copies every field of `other` (`operator=`); `node.assign(&other)`.
    pub fn assign(&self, other: &AdvApp2VarNode) {
        let copy: NodeState = other.state.snapshot();
        *self.state.write() = copy;
    }
}
