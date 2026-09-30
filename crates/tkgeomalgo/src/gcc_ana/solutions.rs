// Rust translation of the solution storage shared by the GccAna solvers (their
// NCollection_Array1 fields and consultation checks) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Storage and consultation checks shared by the GccAna solvers.
//!
//! Every native solver keeps its solutions in fixed-size `NCollection_Array1` fields
//! (1-based) and checks `WellDone` and the index in the same way before answering;
//! these helpers hold that code once.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Xy};

/// A 1-based fixed-size array (a native `NCollection_Array1` field). Reads and writes
/// outside [1, size] raise Standard_OutOfRange, as the native bounds check does. Native
/// leaves the slots of trivial types uninitialized; here they start at `T::default()`.
#[derive(Clone, Debug)]
pub(crate) struct Slots<T> {
    values: Vec<T>,
}

impl<T: Copy + Default> Slots<T> {
    /// `NCollection_Array1<T>(1, size)`.
    pub(crate) fn new(size: usize) -> Self {
        Self {
            values: vec![T::default(); size],
        }
    }

    /// The upper bound (`Upper()`), the size of a 1-based array.
    pub(crate) fn upper(&self) -> i32 {
        self.values.len() as i32
    }

    fn offset(&self, index: i32) -> Result<usize, Failure> {
        usize::try_from(index)
            .ok()
            .filter(|&one_based| one_based >= 1 && one_based <= self.values.len())
            .map(|one_based| one_based - 1)
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "NCollection_Array1::Value: index {index} outside [1, {}]",
                    self.values.len()
                ))
            })
    }

    /// `array(index)` for reading.
    pub(crate) fn get(&self, index: i32) -> Result<T, Failure> {
        Ok(self.values[self.offset(index)?])
    }

    /// `array(index) = value`.
    pub(crate) fn set(&mut self, index: i32, value: T) -> Result<(), Failure> {
        let offset: usize = self.offset(index)?;
        self.values[offset] = value;
        Ok(())
    }
}

/// `StdFail_NotDone` raised by a consultation of a solver that did not succeed, or
/// by a tangency query on a solution identical to its argument.
pub(crate) fn not_done(member: &str) -> Failure {
    Failure::with_type(
        "StdFail_NotDone",
        &format!("{member}: the construction algorithm did not succeed"),
        "",
    )
}

/// `Standard_OutOfRange` raised for a solution index outside [1, NbSolutions].
pub(crate) fn solution_out_of_range(member: &str, index: i32, count: i32) -> Failure {
    Failure::out_of_range(&format!(
        "{member}: solution index {index} outside [1, {count}]"
    ))
}

/// `Standard_NegativeValue` raised by the solvers for a negative radius.
pub(crate) fn negative_value(member: &str, radius: f64) -> Failure {
    Failure::with_type(
        "Standard_NegativeValue",
        &format!("{member}: Radius = {radius}; expected >= 0"),
        "",
    )
}

/// The first consultation check of most solvers: StdFail_NotDone when the
/// algorithm failed, then Standard_OutOfRange for a bad index.
pub(crate) fn check_solution(
    well_done: bool,
    count: i32,
    index: i32,
    member: &str,
) -> Result<(), Failure> {
    if !well_done {
        return Err(not_done(member));
    }
    check_index(count, index, member)
}

/// The index check alone, for the members that skip the `WellDone` test natively.
pub(crate) fn check_index(count: i32, index: i32, member: &str) -> Result<(), Failure> {
    if index <= 0 || index > count {
        return Err(solution_out_of_range(member, index, count));
    }
    Ok(())
}

/// The tangency data of the solutions on one argument: the `TheSame<i>`,
/// `pnttg<i>sol`, `par<i>sol` and `pararg<i>` arrays of a native solver.
#[derive(Clone, Debug)]
pub(crate) struct Tangencies {
    pub(crate) same: Slots<i32>,
    pub(crate) point: Slots<Point2>,
    pub(crate) par_sol: Slots<f64>,
    pub(crate) par_arg: Slots<f64>,
}

impl Tangencies {
    /// Arrays of `size` slots each.
    pub(crate) fn new(size: usize) -> Self {
        Self {
            same: Slots::new(size),
            point: Slots::new(size),
            par_sol: Slots::new(size),
            par_arg: Slots::new(size),
        }
    }

    /// (ParSol, ParArg, PntSol) of solution `index`, without the TheSame test.
    pub(crate) fn raw(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        Ok((
            self.par_sol.get(index)?,
            self.par_arg.get(index)?,
            self.point.get(index)?,
        ))
    }

    /// (ParSol, ParArg, PntSol) of solution `index`; StdFail_NotDone when the
    /// solution is the argument itself (TheSame != 0).
    pub(crate) fn checked(&self, index: i32, member: &str) -> Result<(f64, f64, Point2), Failure> {
        if self.same.get(index)? != 0 {
            return Err(not_done(member));
        }
        self.raw(index)
    }

    /// Whether solution `index` is the argument itself (TheSame != 0).
    pub(crate) fn is_the_same(&self, index: i32) -> Result<bool, Failure> {
        Ok(self.same.get(index)? != 0)
    }

    /// Record a tangency: its point and parameters, TheSame = 0.
    pub(crate) fn record(
        &mut self,
        index: i32,
        point: Point2,
        par_sol: f64,
        par_arg: f64,
    ) -> Result<(), Failure> {
        self.same.set(index, 0)?;
        self.point.set(index, point)?;
        self.par_sol.set(index, par_sol)?;
        self.par_arg.set(index, par_arg)
    }
}

/// `gp_Circ2d(gp_Ax2d(center, gp_Dir2d::D::X), radius)`, the frame of every circle
/// solution; Standard_ConstructionError for a negative radius.
pub(crate) fn circle_at(
    center: Point2,
    radius: f64,
) -> Result<rustycad_tkmath::gp::Circle2, Failure> {
    use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, StandardDirection2};
    let dirx = Direction2::from_standard(StandardDirection2::X);
    Circle2::from_x_axis(Axis2d::new(center, dirx), radius, true)
}

/// The position of a solution circle (centre at `distcc` from the argument
/// centre, radius `radius`) relative to an argument circle of radius `r1`: the
/// given qualifier unless it is unqualified, else enclosed, outside or enclosing by
/// the native tolerance tests (`inclusive` selects `<=` over `<`, as each solver
/// writes it).
pub(crate) fn qualify(
    given: crate::gcc_ent::Position,
    distcc: f64,
    radius: f64,
    r1: f64,
    tol: f64,
    inclusive: bool,
) -> crate::gcc_ent::Position {
    use crate::gcc_ent::Position;
    if given != Position::Unqualified {
        return given;
    }
    let near = |value: f64| {
        if inclusive {
            value.abs() <= tol
        } else {
            value.abs() < tol
        }
    };
    if near(distcc + radius - r1) {
        return Position::Enclosed;
    }
    if near(distcc - r1 - radius) {
        return Position::Outside;
    }
    Position::Enclosing
}

// The tangency point of the circle (center, radius) on `line`:
// dc = Dir(origin - center); sign = dc . Dir(-dy, dx); dc = Dir(sign * (-dy, dx)).
pub(crate) fn line_tangency(line: Line2, center: Point2, radius: f64) -> Result<Point2, Failure> {
    let (dx, dy) = (line.direction().x(), line.direction().y());
    let dc = Direction2::from_xy(line.location().xy() - center.xy())?;
    let sign: f64 = dc.dot(Direction2::new(-dy, dx)?);
    let normal = Xy::new(-dy, dx);
    let dc = Direction2::from_xy(sign * normal)?;
    Ok(Point2::from(center.xy() + radius * dc.xy()))
}

// The qualifier of a solution relative to a qualified line: the given one, else
// outside when the line origin is on the side of the normal, else enclosed. Native
// builds the direction from the centre to the origin first, so a centre at the
// origin raises Standard_ConstructionError whatever the qualifier.
pub(crate) fn line_qualifier(
    qualified: crate::gcc_ent::QualifiedLin,
    center: Point2,
    normal: Direction2,
) -> Result<crate::gcc_ent::Position, Failure> {
    let dc = Direction2::from_xy(qualified.qualified().location().xy() - center.xy())?;
    if !qualified.is_unqualified() {
        return Ok(qualified.qualifier());
    }
    if dc.dot(normal) > 0.0 {
        return Ok(crate::gcc_ent::Position::Outside);
    }
    Ok(crate::gcc_ent::Position::Enclosed)
}
