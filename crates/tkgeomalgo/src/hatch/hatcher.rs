// Rust translation of Hatch_Hatcher.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hatch_Hatcher: cross hatching of straight lines trimmed by straight boundaries.
use super::{Line, LineForm, Parameter};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, StandardDirection2, Vector2};

/// Hatch_Hatcher: computes cross hatchings in a 2d plane, in three steps: add the
/// lines, trim them with boundaries (the inside is on the left), read back the
/// parameter intervals of each line. In oriented mode the orientation of the
/// trimming curves matters and intervals may be infinite (RealFirst/RealLast);
/// in unoriented mode they are always finite.
/// ```
/// use rustycad_tkgeomalgo::hatch::Hatcher;
/// use rustycad_tkmath::gp::Point2;
/// let mut hatcher = Hatcher::new(1.0e-7, true);
/// hatcher.add_x_line(0.5);
/// hatcher.trim_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0), 1)?;
/// hatcher.trim_segment(Point2::new(1.0, 1.0), Point2::new(0.0, 1.0), 2)?;
/// assert_eq!((hatcher.start(1, 1)?, hatcher.end(1, 1)?), (0.0, 1.0));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct Hatcher {
    my_toler: f64,
    my_lines: Vec<Line>,
    my_orient: bool,
}

// NCollection_Sequence::operator() raises Standard_OutOfRange outside [1, Length].
fn sequence_slot<'a, T>(items: &'a [T], index: i32, what: &str) -> Result<&'a T, Failure> {
    usize::try_from(index)
        .ok()
        .filter(|&one_based| one_based >= 1 && one_based <= items.len())
        .map(|one_based| &items[one_based - 1])
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "Hatch_Hatcher: {what} index {index} out of range; expected in [1, {}]",
                items.len()
            ))
        })
}

impl Hatcher {
    /// Hatch_Hatcher(Tol, Oriented = true): an empty hatcher with intersection
    /// tolerance `tolerance`; `Hatcher::new(1.0e-7, true)`.
    pub fn new(tolerance: f64, oriented: bool) -> Self {
        Self {
            my_toler: tolerance,
            my_lines: Vec::new(),
            my_orient: oriented,
        }
    }

    /// Tolerance(Tol): set the intersection tolerance; `hatcher.set_tolerance(1.0e-6)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.my_toler = tolerance;
    }

    /// Tolerance(): the intersection tolerance; `hatcher.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.my_toler
    }

    /// AddLine(L, T = Hatch_ANYLINE): add a line to trim; `form` is informative
    /// only; `hatcher.add_line(line, LineForm::AnyLine)`.
    pub fn add_line(&mut self, line: Line2, form: LineForm) {
        self.my_lines.push(Line::new(line, form));
    }

    /// AddLine(D, Dist): add the infinite line of direction `direction` at signed
    /// distance `dist` from the origin (the points P with OP.V = Dist, V normal to
    /// D in the direct sense); `hatcher.add_line_at_distance(direction, 2.0)`.
    pub fn add_line_at_distance(&mut self, direction: Direction2, dist: f64) {
        let (x, y) = (direction.x(), direction.y());
        let origin = Point2::new(-y * dist, x * dist);
        self.add_line(Line2::new(origin, direction), LineForm::AnyLine);
    }

    /// AddXLine(X): add the line parallel to the Y axis at abscissa `x`;
    /// `hatcher.add_x_line(1.0)`.
    pub fn add_x_line(&mut self, x: f64) {
        let direction = Direction2::from_standard(StandardDirection2::Y);
        self.add_line(Line2::new(Point2::new(x, 0.0), direction), LineForm::XLine);
    }

    /// AddYLine(Y): add the line parallel to the X axis at ordinate `y`;
    /// `hatcher.add_y_line(1.0)`.
    pub fn add_y_line(&mut self, y: f64) {
        let direction = Direction2::from_standard(StandardDirection2::X);
        self.add_line(Line2::new(Point2::new(0.0, y), direction), LineForm::YLine);
    }

    /// Trim(L, Index = 0): trim the lines at their intersections with the whole
    /// line `line`; `hatcher.trim(line, 0)?`.
    pub fn trim(&mut self, line: Line2, index: i32) -> Result<(), Failure> {
        self.trim_range(line, real_first(), real_last(), index)
    }

    /// Trim(L, Start, End, Index = 0): trim the lines at their intersections with
    /// `line` within its parameter range [start, end]. An intersection at a bound
    /// (within the tolerance) is kept only when the rest of the trimming segment is
    /// on the left of the hatching line; `hatcher.trim_range(line, 0.0, 1.0, 0)?`.
    pub fn trim_range(
        &mut self,
        line: Line2,
        start: f64,
        end: f64,
        index: i32,
    ) -> Result<(), Failure> {
        let tolerance: f64 = self.my_toler;
        for hatch in &mut self.my_lines {
            let inters = AnaIntersection::lines(hatch.lin, line);
            if !inters.is_done() || inters.identical_elements()? || inters.parallel_elements()? {
                continue;
            }
            let pinter = inters.point(1)?;
            let line_par: f64 = pinter.param_on_second()?;
            let norm: f64 = line.direction().crossed(hatch.lin.direction());
            if !keeps_intersection(line_par - start, line_par - end, norm, tolerance) {
                continue;
            }
            hatch.add_intersection(
                pinter.param_on_first(),
                norm > 0.0,
                index,
                line_par,
                tolerance,
            );
        }
        Ok(())
    }

    /// Trim(P1, P2, Index = 0): trim the lines at their intersections with the
    /// oriented segment `p1 p2`. A vector near RealLast is first scaled down by its
    /// X (or else Y) component; a segment not longer than the tolerance is ignored;
    /// `hatcher.trim_segment(p1, p2, 0)?`.
    pub fn trim_segment(&mut self, p1: Point2, p2: Point2, index: i32) -> Result<(), Failure> {
        let mut v = Vector2::between(p1, p2);
        if v.x().abs() > 0.9 * real_last() {
            v = scaled(v, 1.0 / v.x());
        } else if v.y().abs() > 0.9 * real_last() {
            v = scaled(v, 1.0 / v.y());
        }
        if v.magnitude() > self.my_toler {
            let direction = Direction2::from_vector(v)?;
            self.trim_range(Line2::new(p1, direction), 0.0, p1.distance(p2), index)?;
        }
        Ok(())
    }

    /// NbIntervals(): the total number of intervals on all lines;
    /// `hatcher.nb_intervals()`.
    pub fn nb_intervals(&self) -> i32 {
        self.my_lines
            .iter()
            .map(|line| self.intervals_of(line))
            .sum()
    }

    /// NbLines(): the number of lines; `hatcher.nb_lines()`.
    pub fn nb_lines(&self) -> i32 {
        self.my_lines.len() as i32
    }

    /// Line(I): the line of index `i`; Standard_OutOfRange outside
    /// [1, NbLines]; `hatcher.line(1)?`.
    pub fn line(&self, i: i32) -> Result<Line2, Failure> {
        Ok(self.hatch_line(i)?.lin)
    }

    /// LineForm(I): the form of the line of index `i`; `hatcher.line_form(1)?`.
    pub fn line_form(&self, i: i32) -> Result<LineForm, Failure> {
        Ok(self.hatch_line(i)?.form)
    }

    /// IsXLine(I): the line of index `i` has a constant X; `hatcher.is_x_line(1)?`.
    pub fn is_x_line(&self, i: i32) -> Result<bool, Failure> {
        Ok(self.line_form(i)? == LineForm::XLine)
    }

    /// IsYLine(I): the line of index `i` has a constant Y; `hatcher.is_y_line(1)?`.
    pub fn is_y_line(&self, i: i32) -> Result<bool, Failure> {
        Ok(self.line_form(i)? == LineForm::YLine)
    }

    /// Coordinate(I): the X of an X line or the Y of a Y line; Standard_OutOfRange
    /// ("Hatcher : not an X or Y line") for any other line; `hatcher.coordinate(1)?`.
    pub fn coordinate(&self, i: i32) -> Result<f64, Failure> {
        let hatch: &Line = self.hatch_line(i)?;
        match hatch.form {
            LineForm::XLine => Ok(hatch.lin.location().x()),
            LineForm::YLine => Ok(hatch.lin.location().y()),
            LineForm::AnyLine => Err(Failure::out_of_range(&format!(
                "Hatcher : not an X or Y line (line {i} is Hatch_ANYLINE)"
            ))),
        }
    }

    /// NbIntervals(I): the number of intervals on the line of index `i`;
    /// `hatcher.nb_intervals_on(1)?`.
    pub fn nb_intervals_on(&self, i: i32) -> Result<i32, Failure> {
        Ok(self.intervals_of(self.hatch_line(i)?))
    }

    /// Start(I, J): the first parameter of interval `j` on line `i` (RealFirst for
    /// an interval open at its start); Standard_OutOfRange for a missing interval;
    /// `hatcher.start(1, 1)?`.
    pub fn start(&self, i: i32, j: i32) -> Result<f64, Failure> {
        let hatch: &Line = self.hatch_line(i)?;
        if hatch.inters.is_empty() {
            return self.single_open_interval(j, real_first());
        }
        let jj: i32 = self.start_slot(hatch, j);
        if jj == 0 {
            return Ok(real_first());
        }
        Ok(sequence_slot(&hatch.inters, jj, "intersection")?.par1)
    }

    /// StartIndex(I, J, Index, Par2): the trimming-curve index and parameter of the
    /// start of interval `j` on line `i`, (0, 0) for an open start. Unlike Start,
    /// native accepts J = 1 on an empty line even in unoriented mode;
    /// `let (index, par2) = hatcher.start_index(1, 1)?;`.
    pub fn start_index(&self, i: i32, j: i32) -> Result<(i32, f64), Failure> {
        let hatch: &Line = self.hatch_line(i)?;
        if hatch.inters.is_empty() {
            return empty_line_index(j);
        }
        let jj: i32 = self.start_slot(hatch, j);
        if jj == 0 {
            return Ok((0, 0.0));
        }
        Ok(index_and_par2(sequence_slot(
            &hatch.inters,
            jj,
            "intersection",
        )?))
    }

    /// End(I, J): the last parameter of interval `j` on line `i` (RealLast for an
    /// interval open at its end); `hatcher.end(1, 1)?`.
    pub fn end(&self, i: i32, j: i32) -> Result<f64, Failure> {
        let hatch: &Line = self.hatch_line(i)?;
        if hatch.inters.is_empty() {
            return self.single_open_interval(j, real_last());
        }
        let jj: i32 = self.end_slot(hatch, j);
        if jj > hatch.inters.len() as i32 {
            return Ok(real_last());
        }
        Ok(sequence_slot(&hatch.inters, jj, "intersection")?.par1)
    }

    /// EndIndex(I, J, Index, Par2): the trimming-curve index and parameter of the
    /// end of interval `j` on line `i`, (0, 0) for an open end;
    /// `let (index, par2) = hatcher.end_index(1, 1)?;`.
    pub fn end_index(&self, i: i32, j: i32) -> Result<(i32, f64), Failure> {
        let hatch: &Line = self.hatch_line(i)?;
        if hatch.inters.is_empty() {
            return empty_line_index(j);
        }
        let jj: i32 = self.end_slot(hatch, j);
        if jj > hatch.inters.len() as i32 {
            return Ok((0, 0.0));
        }
        Ok(index_and_par2(sequence_slot(
            &hatch.inters,
            jj,
            "intersection",
        )?))
    }

    fn hatch_line(&self, i: i32) -> Result<&Line, Failure> {
        sequence_slot(&self.my_lines, i, "line")
    }

    // In oriented mode a line whose first intersection ends an interval starts
    // with an interval open at RealFirst, which shifts every slot by one.
    fn opens_before_first(&self, hatch: &Line) -> bool {
        !hatch.inters[0].start && self.my_orient
    }

    fn start_slot(&self, hatch: &Line, j: i32) -> i32 {
        2 * j - 1 - i32::from(self.opens_before_first(hatch))
    }

    fn end_slot(&self, hatch: &Line, j: i32) -> i32 {
        2 * j - i32::from(self.opens_before_first(hatch))
    }

    fn intervals_of(&self, hatch: &Line) -> i32 {
        let length = hatch.inters.len() as i32;
        if length == 0 {
            return i32::from(self.my_orient);
        }
        length / 2 + i32::from(self.opens_before_first(hatch))
    }

    // An oriented line without intersections is the single interval
    // [RealFirst, RealLast]; anything else is out of range.
    fn single_open_interval(&self, j: i32, bound: f64) -> Result<f64, Failure> {
        if j != 1 || !self.my_orient {
            return Err(Failure::out_of_range(&format!(
                "Hatch_Hatcher: interval {j} on a line without intersections (oriented = {}); expected 1 in oriented mode",
                self.my_orient
            )));
        }
        Ok(bound)
    }
}

// Whether an intersection at parameter offsets `from_start`/`from_end` on the
// trimming line is kept: inside the range, or on a bound with the rest of the
// trimming segment on the left of the hatching line.
fn keeps_intersection(from_start: f64, from_end: f64, norm: f64, tolerance: f64) -> bool {
    if from_start < -tolerance || from_end > tolerance {
        return false;
    }
    if from_start < tolerance && norm < 0.0 {
        return false;
    }
    !(from_end > -tolerance && norm > 0.0)
}

fn index_and_par2(parameter: &Parameter) -> (i32, f64) {
    (parameter.index, parameter.par2)
}

fn empty_line_index(j: i32) -> Result<(i32, f64), Failure> {
    if j != 1 {
        return Err(Failure::out_of_range(&format!(
            "Hatch_Hatcher: interval {j} on a line without intersections; expected 1"
        )));
    }
    Ok((0, 0.0))
}

// gp_Vec2d::Multiply: both coordinates times `factor`.
fn scaled(vector: Vector2, factor: f64) -> Vector2 {
    Vector2::new(vector.x() * factor, vector.y() * factor)
}
