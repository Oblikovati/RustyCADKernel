// Rust translation of HatchGen_Domain.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HatchGen_Domain`: a part of a hatching between two points, possibly infinite.
use super::text::index_title;
use super::{IntersectionPointDump, PointOnHatching};
use rustycad_tkernel::standard::Failure;
use std::fmt;

/// `HatchGen_Domain`: the part of a hatching curve between a first and a second
/// point; a missing point means the domain is infinite on that side. Clearing a
/// point keeps its stored value, as natively; only the flag changes.
///
/// ```
/// use rustycad_tkgeomalgo::hatch_gen::{Domain, PointOnHatching};
/// let domain = Domain::semi_infinite(&PointOnHatching::new(), true);
/// assert!(domain.has_first_point() && !domain.has_second_point());
/// assert!(domain.second_point().is_err());
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Domain {
    my_has_first_point: bool,
    my_first_point: PointOnHatching,
    my_has_second_point: bool,
    my_second_point: PointOnHatching,
}

// The Standard_DomainError of FirstPoint and SecondPoint on a missing point.
fn missing_point(member: &str, which: &str) -> Failure {
    Failure::with_type(
        "Standard_DomainError",
        &format!("{member}: the domain has no {which} point; expected Has{which}Point()"),
        "",
    )
}

impl Domain {
    /// `HatchGen_Domain()`: an infinite domain; `Domain::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `HatchGen_Domain(P1, P2)`: the domain from `p1` to `p2`;
    /// `Domain::bounded(&p1, &p2)`.
    pub fn bounded(p1: &PointOnHatching, p2: &PointOnHatching) -> Self {
        Self {
            my_has_first_point: true,
            my_first_point: p1.clone(),
            my_has_second_point: true,
            my_second_point: p2.clone(),
        }
    }

    /// `HatchGen_Domain(P, First)`: the semi-infinite domain starting at `p` when
    /// `first`, else ending at `p`; `Domain::semi_infinite(&p, true)`.
    pub fn semi_infinite(p: &PointOnHatching, first: bool) -> Self {
        let mut domain = Self::new();
        if first {
            domain.set_first_point(p);
        } else {
            domain.set_second_point(p);
        }
        domain
    }

    /// SetPoints(P1, P2): both points; `domain.set_points(&p1, &p2)`.
    pub fn set_points(&mut self, p1: &PointOnHatching, p2: &PointOnHatching) {
        self.set_first_point(p1);
        self.set_second_point(p2);
    }

    /// SetPoints(): both points at infinity; `domain.set_infinite_points()`.
    pub fn set_infinite_points(&mut self) {
        self.my_has_first_point = false;
        self.my_has_second_point = false;
    }

    /// SetFirstPoint(P); `domain.set_first_point(&p)`.
    pub fn set_first_point(&mut self, p: &PointOnHatching) {
        self.my_has_first_point = true;
        self.my_first_point = p.clone();
    }

    /// SetFirstPoint(): the first point at infinity; `domain.set_first_infinite()`.
    pub fn set_first_infinite(&mut self) {
        self.my_has_first_point = false;
    }

    /// SetSecondPoint(P); `domain.set_second_point(&p)`.
    pub fn set_second_point(&mut self, p: &PointOnHatching) {
        self.my_has_second_point = true;
        self.my_second_point = p.clone();
    }

    /// SetSecondPoint(): the second point at infinity; `domain.set_second_infinite()`.
    pub fn set_second_infinite(&mut self) {
        self.my_has_second_point = false;
    }

    /// HasFirstPoint; `domain.has_first_point()`.
    pub fn has_first_point(&self) -> bool {
        self.my_has_first_point
    }

    /// FirstPoint: Standard_DomainError without a first point; `domain.first_point()?`.
    pub fn first_point(&self) -> Result<&PointOnHatching, Failure> {
        if !self.my_has_first_point {
            return Err(missing_point("HatchGen_Domain::FirstPoint", "First"));
        }
        Ok(&self.my_first_point)
    }

    /// HasSecondPoint; `domain.has_second_point()`.
    pub fn has_second_point(&self) -> bool {
        self.my_has_second_point
    }

    /// SecondPoint: Standard_DomainError without a second point; `domain.second_point()?`.
    pub fn second_point(&self) -> Result<&PointOnHatching, Failure> {
        if !self.my_has_second_point {
            return Err(missing_point("HatchGen_Domain::SecondPoint", "Second"));
        }
        Ok(&self.my_second_point)
    }

    /// Dump(Index = 0): writes the native `std::cout` text of the domain to `out`;
    /// `domain.dump(1, &mut text)?`.
    pub fn dump(&self, index: i32, out: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(
            out,
            "=== Domain {}=============================",
            index_title(index, "======")
        )?;
        if self.my_has_first_point {
            self.my_first_point.dump(1, out)?;
        } else {
            writeln!(out, "    Has not a first point")?;
        }
        if self.my_has_second_point {
            self.my_second_point.dump(2, out)?;
        } else {
            writeln!(out, "    Has not a second point")?;
        }
        writeln!(out, "==============================================")
    }
}
