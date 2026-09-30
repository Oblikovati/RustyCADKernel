// Rust translation of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_SectionLine.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_SectionLine: a polyline of intersection between two polyhedra.
use super::section_point::SectionPoint;
use super::sequence_access::{first, item, last, write_text};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use std::fmt;

/// Intf_SectionLine: a sequence of section points.
///
/// The `closed` flag that `close` sets is never read: `is_closed` compares the first
/// and last points instead, and the copy constructor does not copy the flag, as
/// natively.
/// ```
/// use rustycad_tkgeomalgo::intf::{SectionLine, SectionPoint};
/// let mut line = SectionLine::new();
/// line.append(SectionPoint::new());
/// assert_eq!(line.number_of_points(), 1);
/// assert!(line.is_closed().unwrap());
/// ```
#[derive(Default)]
pub struct SectionLine {
    points: Sequence<SectionPoint>,
    // Stored as natively and never read (see the type documentation).
    #[allow(dead_code)]
    closed: bool,
}

impl Clone for SectionLine {
    /// Intf_SectionLine(const Intf_SectionLine&): copies the points, not the flag.
    fn clone(&self) -> Self {
        Self {
            points: self.points.clone(),
            closed: false,
        }
    }
}

impl PartialEq for SectionLine {
    /// operator==: `is_equal`.
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl SectionLine {
    /// Intf_SectionLine(): an empty line; `SectionLine::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// operator=: copies the points and keeps this line's flag, as natively;
    /// `line.assign(&other)`.
    pub fn assign(&mut self, other: &Self) {
        self.points.assign(&other.points);
    }

    /// NumberOfPoints; `line.number_of_points()`.
    pub fn number_of_points(&self) -> i32 {
        self.points.length()
    }

    /// GetPoint: the point of one-based `index`; raises Standard_OutOfRange outside
    /// the line; `line.get_point(1)?`.
    pub fn get_point(&self, index: i32) -> Result<&SectionPoint, Failure> {
        item(&self.points, index)
    }

    /// IsClosed: whether the first and last points are logically equal; raises
    /// Standard_NoSuchObject on an empty line; `line.is_closed()?`.
    pub fn is_closed(&self) -> Result<bool, Failure> {
        Ok(first(&self.points)?.is_equal(last(&self.points)?))
    }

    /// Contains: whether a point of the line is logically equal to `point`;
    /// `line.contains(&point)`.
    pub fn contains(&self, point: &SectionPoint) -> bool {
        self.points
            .iter()
            .any(|candidate| point.is_equal(candidate))
    }

    /// IsEnd: 1 when `point` is the first point, the number of points (not 2, despite
    /// the native documentation) when it is the last, else 0; raises
    /// Standard_NoSuchObject on an empty line; `line.is_end(&point)?`.
    pub fn is_end(&self, point: &SectionPoint) -> Result<i32, Failure> {
        if first(&self.points)?.is_equal(point) {
            return Ok(1);
        }
        if last(&self.points)?.is_equal(point) {
            return Ok(self.points.length());
        }
        Ok(0)
    }

    /// IsEqual: the same number of points, pairwise logically equal;
    /// `line.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.points.len() == other.points.len()
            && self
                .points
                .iter()
                .zip(other.points.iter())
                .all(|(a, b)| a.is_equal(b))
    }

    /// Append(Pi): adds a point at the end; `line.append(point)`.
    pub fn append(&mut self, point: SectionPoint) {
        self.points.append(point);
    }

    /// Append(LS): moves the points of `other` to the end; `other` becomes empty, as
    /// NCollection_Sequence::Append of a sequence does; `line.append_line(&mut other)`.
    pub fn append_line(&mut self, other: &mut Self) {
        self.points.append_sequence(&mut other.points);
    }

    /// Prepend(Pi): adds a point at the beginning; `line.prepend(point)`.
    pub fn prepend(&mut self, point: SectionPoint) {
        self.points.prepend(point);
    }

    /// Prepend(LS): moves the points of `other` to the beginning, emptying it;
    /// `line.prepend_line(&mut other)`.
    pub fn prepend_line(&mut self, other: &mut Self) {
        self.points.prepend_sequence(&mut other.points);
    }

    /// Reverse: reverses the order of the points; `line.reverse()`.
    pub fn reverse(&mut self) {
        self.points.reverse();
    }

    /// Close: sets the (never read) closed flag; `line.close()`.
    pub fn close(&mut self) {
        self.closed = true;
    }

    /// Dump: `indent` spaces, `LS `, then `Closed :` or `Open :` and a new line (the
    /// point dumps print nothing), written where native code uses `std::cout`. An
    /// empty line raises Standard_NoSuchObject after `LS ` is written, as natively;
    /// `line.dump(2, &mut text)?`.
    pub fn dump(&self, indent: i32, stream: &mut dyn fmt::Write) -> Result<(), Failure> {
        let prefix = format!("{}LS ", " ".repeat(usize::try_from(indent).unwrap_or(0)));
        write_text(stream, &prefix, "Intf_SectionLine::Dump")?;
        let state = if self.is_closed()? {
            "Closed :\n"
        } else {
            "Open :\n"
        };
        write_text(stream, state, "Intf_SectionLine::Dump")?;
        for point in self.points.iter() {
            point.dump(indent + 2);
        }
        Ok(())
    }
}
