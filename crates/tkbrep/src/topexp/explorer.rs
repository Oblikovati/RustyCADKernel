// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/TopExp/TopExp_Explorer.hxx and
// TopExp_Explorer.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopExp_Explorer`: a depth-first search for the sub-shapes of one type
//! (docs/topology-design.md section 6).
use crate::topods::errors::null_shape;
use crate::topods::{Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Explores the sub-shapes of type `to_find` of a shape, skipping those under a shape
/// of type `to_avoid` (no effect when `to_avoid` is SHAPE or not more complex than
/// `to_find`).
///
/// ```
/// use rustycad_tkbrep::topexp::Explorer;
/// use rustycad_tkbrep::topods::{Builder, Compound, Wire};
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// let (mut compound, mut wire) = (Compound::default(), Wire::default());
/// Builder.make_compound(&mut compound);
/// Builder.make_wire(&mut wire);
/// Builder.add(&mut compound, &wire).unwrap();
/// let mut explorer = Explorer::new(&compound, ShapeEnum::Wire, ShapeEnum::Shape).unwrap();
/// assert!(explorer.more() && explorer.current().is_same(&wire));
/// explorer.next().unwrap();
/// assert!(!explorer.more());
/// ```
#[derive(Clone)]
pub struct Explorer {
    stack: Vec<ShapeIterator>,
    shape: Shape,
    to_find: ShapeEnum,
    to_avoid: ShapeEnum,
    has_more: bool,
}

impl Default for Explorer {
    /// `TopExp_Explorer()`: an empty explorer, useful after `init`.
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            shape: Shape::default(),
            to_find: ShapeEnum::Shape,
            to_avoid: ShapeEnum::Shape,
            has_more: false,
        }
    }
}

impl Explorer {
    /// `TopExp_Explorer(S, ToFind, ToAvoid)`; the native default `ToAvoid` is
    /// `ShapeEnum::Shape`; `Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)?`.
    pub fn new(shape: &Shape, to_find: ShapeEnum, to_avoid: ShapeEnum) -> Result<Self, Failure> {
        let mut explorer = Self::default();
        explorer.init(shape, to_find, to_avoid)?;
        Ok(explorer)
    }

    /// `Init(S, ToFind, ToAvoid)`: restarts the exploration of `shape`; the shape itself
    /// is found when it has the type `to_find`; `explorer.init(&shape, find, avoid)?`.
    pub fn init(
        &mut self,
        shape: &Shape,
        to_find: ShapeEnum,
        to_avoid: ShapeEnum,
    ) -> Result<(), Failure> {
        self.clear();
        self.shape = shape.clone();
        self.to_find = to_find;
        self.to_avoid = to_avoid;
        let Some(tshape) = shape.tshape().get() else {
            return Ok(());
        };
        if to_find == ShapeEnum::Shape {
            return Ok(());
        }
        let kind = tshape.shape_type();
        if kind > to_find {
            return Ok(());
        }
        self.has_more = true;
        if kind != to_find {
            self.next()?;
        }
        Ok(())
    }

    /// `More()`: true while a shape remains; `explorer.more()`.
    pub fn more(&self) -> bool {
        self.has_more
    }

    /// `Next()`: moves to the next shape of the searched type. After the end the stack
    /// is empty and a new call restarts the exploration from the root, as natively;
    /// Standard_NullObject when the explored shape is null (a crash natively);
    /// `explorer.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol; Iterator is implemented too.
    pub fn next(&mut self) -> Result<(), Failure> {
        if self.stack.is_empty() {
            let kind = self
                .shape
                .shape_type()
                .map_err(|_| null_shape("TopExp_Explorer::Next"))?;
            if kind == self.to_find || self.should_avoid(kind) {
                self.has_more = false;
                return Ok(());
            }
            self.stack.push(ShapeIterator::new(&self.shape, true, true));
        } else if let Some(top) = self.stack.last_mut() {
            top.next();
        }
        self.descend();
        Ok(())
    }

    /// The loop of `Next`: goes down into more complex shapes that are not avoided,
    /// stops on a shape of the searched type, pops exhausted levels.
    fn descend(&mut self) {
        let (to_find, to_avoid) = (self.to_find, self.to_avoid);
        while let Some(top) = self.stack.last_mut() {
            let step = match top.value() {
                Err(_) => Step::Pop,
                Ok(current) => classify(current, to_find, to_avoid),
            };
            match step {
                Step::Found => {
                    self.has_more = true;
                    return;
                }
                Step::Push(child) => self.stack.push(child),
                Step::Skip => top.next(),
                Step::Pop => self.pop_iterator(),
            }
        }
        self.has_more = false;
    }

    /// `popIterator` followed by `myStack[myStackTop].Next()` when a level remains.
    fn pop_iterator(&mut self) {
        self.stack.pop();
        if let Some(top) = self.stack.last_mut() {
            top.next();
        }
    }

    fn should_avoid(&self, kind: ShapeEnum) -> bool {
        should_avoid(kind, self.to_avoid)
    }

    /// `Current()` (and `Value()`): the current shape; at the end, and before any
    /// descent, the explored shape itself; `explorer.current()`.
    pub fn current(&self) -> &Shape {
        match self.stack.last() {
            // A level on the stack always has a current shape (exhausted levels are popped).
            Some(top) => top.value().unwrap_or(&self.shape),
            None => &self.shape,
        }
    }

    /// `Value()`: the same as [`Explorer::current`]; `explorer.value()`.
    pub fn value(&self) -> &Shape {
        self.current()
    }

    /// `ReInit()`: restarts with the original arguments; `explorer.re_init()?`.
    pub fn re_init(&mut self) -> Result<(), Failure> {
        let shape = self.shape.clone();
        self.init(&shape, self.to_find, self.to_avoid)
    }

    /// `ExploredShape()`: the shape given to `init`; `explorer.explored_shape()`.
    pub fn explored_shape(&self) -> &Shape {
        &self.shape
    }

    /// `Depth()`: the stack depth, 0 for the explored shape itself; `explorer.depth()`.
    pub fn depth(&self) -> i32 {
        self.stack.len() as i32
    }

    /// `Clear()`: empties the stack and ends the exploration; `explorer.clear()`.
    pub fn clear(&mut self) {
        self.stack.clear();
        self.has_more = false;
    }
}

/// What the loop of `Next` does with the top level of the stack.
enum Step {
    /// The current shape has the searched type.
    Found,
    /// Descend into the current shape.
    Push(ShapeIterator),
    /// Move the top iterator to its next shape.
    Skip,
    /// The top level is exhausted.
    Pop,
}

/// One turn of the native loop on the current shape of the top iterator. A child
/// always has a TShape (`Builder::add` refuses a null component).
fn classify(current: &Shape, to_find: ShapeEnum, to_avoid: ShapeEnum) -> Step {
    let Some(tshape) = current.tshape().get() else {
        return Step::Skip;
    };
    let kind = tshape.shape_type();
    if kind == to_find {
        return Step::Found;
    }
    // `isMoreComplex`: the enumeration runs from COMPOUND (0) to VERTEX (7).
    if to_find > kind && !should_avoid(kind, to_avoid) {
        return Step::Push(ShapeIterator::new(current, true, true));
    }
    Step::Skip
}

/// `shouldAvoid`: `toAvoid` is not SHAPE and equals the type.
fn should_avoid(kind: ShapeEnum, to_avoid: ShapeEnum) -> bool {
    to_avoid != ShapeEnum::Shape && to_avoid == kind
}

/// The native `begin()`/`end()` range: yields each found shape. An error of `next`
/// (a null explored shape) ends the iteration.
impl Iterator for Explorer {
    type Item = Shape;
    fn next(&mut self) -> Option<Shape> {
        if !self.more() {
            return None;
        }
        let current = self.current().clone();
        Explorer::next(self).ok()?;
        Some(current)
    }
}
