// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntWalk_IWalking`: the lines of the zero set of an implicit function on a
//! parametric surface, walked from start points on the boundary (open lines) and from
//! interior points (closed lines).
mod cadre;
mod close_line;
mod close_step;
mod deflection;
mod holes;
mod open_line;
mod open_step;
mod passage;
mod perform;
mod points;
mod stop_tests;
mod tools;

pub use tools::{IWalkingTypes, TheIWFunction, TheIWLine};

use super::{IntWalkVectorOfInteger, IntWalkVectorOfWalkingData, IntWalkWalkingData};
use crate::int_surf::{IntSurfPathPoint, IntSurfPntOn2S};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::bnd::Range1;
use rustycad_tkmath::gp::{Direction2, Vector3};
use rustycad_tkmath::math::Vector;
use std::collections::BTreeMap;

/// `CosRef3D`: the squared cosine under which the 3d angle is too great (tests in U4,
/// about 11.5 degrees).
const COS_REF_3D: f64 = 0.98;
/// `CosRef2D`: the same in the parameters (about 25 degrees).
const COS_REF_2D: f64 = 0.88;
/// `MaxDivision`: the maximum number of step divisions.
const MAX_DIVISION: i32 = 60;

/// The walking of the zero set of the function of an instance `T`.
///
/// `IntPatch_TheIWalking` is `IntWalkIWalking<T>` with the IntPatch types;
/// `let mut walk = IntWalkIWalking::<T>::new(1e-6, 0.01, 0.1, false)?;`.
pub struct IntWalkIWalking<T: IWalkingTypes> {
    done: bool,
    seq_single: Vec<IntSurfPathPoint>,
    fleche: f64,
    pas: f64,
    tolerance: Vector<'static>,
    epsilon: f64,
    reversed: bool,
    wd1: IntWalkVectorOfWalkingData,
    wd2: IntWalkVectorOfWalkingData,
    nb_multiplicities: IntWalkVectorOfInteger,
    my_s_range_u: Range1,
    my_s_range_v: Range1,
    um: f64,
    um_last: f64,
    vm: f64,
    vm_last: f64,
    previous_point: IntSurfPntOn2S,
    previousd3d: Vector3,
    previousd2d: Direction2,
    seq_ajout: Vec<i32>,
    seq_alone: Vec<i32>,
    point_line_line: BTreeMap<i32, Vec<i32>>,
    lines: Vec<Handle<T::Line>>,
    to_fill_holes: bool,
}

/// The not-done error of the consultation members.
fn not_done(member: &str) -> Failure {
    Failure::with_message("StdFail_NotDone", &format!("IntWalk_IWalking::{member}"))
}

/// The 1-based item `index` of a native sequence, or its Standard_OutOfRange.
pub(super) fn sequence_value<'a, X>(
    items: &'a [X],
    index: i32,
    what: &str,
) -> Result<&'a X, Failure> {
    if index < 1 || index as usize > items.len() {
        return Err(errors::out_of_range(&format!(
            "IntWalk_IWalking: {what} index {index} outside 1..={}",
            items.len()
        )));
    }
    Ok(&items[index as usize - 1])
}

/// The index of `wd[index]`: the native `operator[]` does not check it (reading past
/// the end is undefined natively); the port raises Standard_OutOfRange.
pub(super) fn wd_index(wd: &[IntWalkWalkingData], index: i32) -> Result<usize, Failure> {
    if index < 0 || index as usize >= wd.len() {
        return Err(errors::out_of_range(&format!(
            "IntWalk_IWalking: walking data index {index} outside 0..{}",
            wd.len()
        )));
    }
    Ok(index as usize)
}

/// The dummy entry that keeps the walking data 1-based (`Clear`).
const DUMMY: IntWalkWalkingData = IntWalkWalkingData {
    ustart: 0.0,
    vstart: 0.0,
    etat: -10,
};

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// `IntWalk_IWalking(Epsilon, Deflection, Increment, theToFillHoles)`: the
    /// confusion tolerance (squared here), the deflection, the relative step and
    /// whether holes between lines are filled;
    /// `IntWalkIWalking::<T>::new(1e-6, 0.01, 0.1, false)?`.
    pub fn new(
        epsilon: f64,
        deflection: f64,
        increment: f64,
        to_fill_holes: bool,
    ) -> Result<Self, Failure> {
        Ok(Self {
            done: false,
            seq_single: Vec::new(),
            fleche: deflection,
            pas: increment,
            tolerance: Vector::with_bounds(1, 2)?,
            epsilon: epsilon * epsilon,
            reversed: false,
            wd1: Vec::new(),
            wd2: Vec::new(),
            nb_multiplicities: Vec::new(),
            my_s_range_u: Range1::default(),
            my_s_range_v: Range1::default(),
            um: 0.0,
            um_last: 0.0,
            vm: 0.0,
            vm_last: 0.0,
            previous_point: IntSurfPntOn2S::default(),
            previousd3d: Vector3::default(),
            previousd2d: Direction2::default(),
            seq_ajout: Vec::new(),
            seq_alone: Vec::new(),
            point_line_line: BTreeMap::new(),
            lines: Vec::new(),
            to_fill_holes,
        })
    }
    /// `SetTolerance(Epsilon, Deflection, Increment)`; `walk.set_tolerance(1e-6, 0.01, 0.1)`.
    pub fn set_tolerance(&mut self, epsilon: f64, deflection: f64, increment: f64) {
        self.fleche = deflection;
        self.pas = increment;
        self.epsilon = epsilon * epsilon;
    }
    /// `Clear()`: empties the walking data (keeping a dummy first entry, so they stay
    /// 1-based), the added points and the lines; the single points, the lonely ends
    /// and the estimated ranges stay, as natively. `walk.clear()`.
    pub fn clear(&mut self) {
        self.wd1.clear();
        self.wd2.clear();
        self.wd1.push(DUMMY);
        self.wd2.push(DUMMY);
        self.nb_multiplicities.clear();
        self.nb_multiplicities.push(-1);
        self.done = false;
        self.seq_ajout.clear();
        self.lines.clear();
    }
    /// `IsDone()`; `walk.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbLines()`: StdFail_NotDone before a Perform; `walk.nb_lines()?`.
    pub fn nb_lines(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("NbLines"));
        }
        Ok(self.lines.len() as i32)
    }
    /// `Value(Index)`: the line of rank `index`; `walk.value(1)?`.
    pub fn value(&self, index: i32) -> Result<&Handle<T::Line>, Failure> {
        if !self.done {
            return Err(not_done("Value"));
        }
        sequence_value(&self.lines, index, "line")
    }
    /// `NbSinglePnts()`: the start points on which no line starts or ends.
    pub fn nb_single_pnts(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("NbSinglePnts"));
        }
        Ok(self.seq_single.len() as i32)
    }
    /// `SinglePnt(Index)`; `walk.single_pnt(1)?`.
    pub fn single_pnt(&self, index: i32) -> Result<&IntSurfPathPoint, Failure> {
        if !self.done {
            return Err(not_done("SinglePnt"));
        }
        sequence_value(&self.seq_single, index, "single point")
    }
    /// The parameters of a line point on the walked surface: on S2 unless reversed.
    fn surface_parameters(&self, point: &IntSurfPntOn2S) -> (f64, f64) {
        if self.reversed {
            point.parameters_on_s1()
        } else {
            point.parameters_on_s2()
        }
    }
    /// The line of the handle; the walking creates every line, so none is null.
    fn line_of(handle: &Handle<T::Line>) -> Result<&T::Line, Failure> {
        handle
            .get()
            .ok_or_else(|| errors::null_object("IntWalk_IWalking: null line"))
    }
    /// The line of rank `index` in the computed lines (`lines.Value(Index)`).
    fn line_at(&self, index: i32) -> Result<&T::Line, Failure> {
        Self::line_of(sequence_value(&self.lines, index, "line")?)
    }
}
