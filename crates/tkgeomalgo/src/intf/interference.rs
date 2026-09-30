// Rust translation of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_Interference.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_Interference: the result shared by every interference algorithm (section
//! points, section lines, tangent zones and the tolerance).
use super::section_line::SectionLine;
use super::section_point::SectionPoint;
use super::sequence_access::{item, write_text};
use super::tangent_zone::TangentZone;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use std::fmt;

/// Intf_Interference: the base of the interference algorithms. The native class has a
/// protected constructor and protected members; the algorithms (InterferencePolygon2d,
/// the generic polygon-polyhedron interference, and derived classes of other packages)
/// own one, reach it through `Deref`/`DerefMut`, and fill it through the `*_mut`
/// accessors that stand for the protected members.
///
/// ```
/// use rustycad_tkgeomalgo::intf::{Interference, SectionPoint};
/// let mut interference = Interference::new(false);
/// interference.insert_points(&SectionPoint::new(), &SectionPoint::new()).unwrap();
/// assert_eq!(interference.nb_section_lines(), 1);
/// ```
#[derive(Clone, Default)]
pub struct Interference {
    pub(crate) section_points: Sequence<SectionPoint>,
    pub(crate) section_lines: Sequence<SectionLine>,
    pub(crate) tangent_zones: Sequence<TangentZone>,
    pub(crate) self_interference: bool,
    pub(crate) tolerance: f64,
}

impl Interference {
    /// Intf_Interference(Self) (protected natively): an empty result for a self
    /// interference or not, with a null tolerance; `Interference::new(false)`.
    pub fn new(self_interference: bool) -> Self {
        Self {
            self_interference,
            ..Self::default()
        }
    }

    /// SelfInterference (protected natively): sets the kind and clears the points,
    /// lines and zones; the tolerance is kept; `interference.self_interference(true)`.
    pub fn self_interference(&mut self, self_interference: bool) {
        self.self_interference = self_interference;
        self.section_points.clear(None);
        self.section_lines.clear(None);
        self.tangent_zones.clear(None);
    }

    /// mySPoins (protected natively): the section points, for the interference
    /// algorithms that fill them; `interference.section_points_mut().append(point)`.
    pub fn section_points_mut(&mut self) -> &mut Sequence<SectionPoint> {
        &mut self.section_points
    }

    /// mySLines (protected natively): the section lines;
    /// `interference.section_lines_mut().clear(None)`.
    pub fn section_lines_mut(&mut self) -> &mut Sequence<SectionLine> {
        &mut self.section_lines
    }

    /// myTZones (protected natively): the tangent zones;
    /// `interference.tangent_zones_mut().append(zone)`.
    pub fn tangent_zones_mut(&mut self) -> &mut Sequence<TangentZone> {
        &mut self.tangent_zones
    }

    /// Tolerance (protected natively): sets the tolerance an algorithm computed;
    /// `interference.set_tolerance(1e-7)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }

    /// NbSectionPoints; `interference.nb_section_points()`.
    pub fn nb_section_points(&self) -> i32 {
        self.section_points.length()
    }

    /// PntValue: the section point of one-based `index`; raises Standard_OutOfRange
    /// outside the points; `interference.pnt_value(1)?`.
    pub fn pnt_value(&self, index: i32) -> Result<&SectionPoint, Failure> {
        item(&self.section_points, index)
    }

    /// NbSectionLines; `interference.nb_section_lines()`.
    pub fn nb_section_lines(&self) -> i32 {
        self.section_lines.length()
    }

    /// LineValue: the section line of one-based `index`; raises Standard_OutOfRange;
    /// `interference.line_value(1)?`.
    pub fn line_value(&self, index: i32) -> Result<&SectionLine, Failure> {
        item(&self.section_lines, index)
    }

    /// NbTangentZones; `interference.nb_tangent_zones()`.
    pub fn nb_tangent_zones(&self) -> i32 {
        self.tangent_zones.length()
    }

    /// ZoneValue: the tangent zone of one-based `index`; raises Standard_OutOfRange;
    /// `interference.zone_value(1)?`.
    pub fn zone_value(&self, index: i32) -> Result<&TangentZone, Failure> {
        item(&self.tangent_zones, index)
    }

    /// GetTolerance; `interference.get_tolerance()`.
    pub fn get_tolerance(&self) -> f64 {
        self.tolerance
    }

    /// Contains: whether a section line or a tangent zone holds a point logically equal
    /// to `point` (the isolated section points are not searched, as natively);
    /// `interference.contains(&point)`.
    pub fn contains(&self, point: &SectionPoint) -> bool {
        self.section_lines.iter().any(|line| line.contains(point))
            || self.tangent_zones.iter().any(|zone| zone.contains(point))
    }

    /// Dump: the three headers, each followed by its elements' dumps (the section
    /// point and tangent zone dumps print nothing), written where native code uses
    /// `std::cout`; an empty section line raises Standard_NoSuchObject;
    /// `interference.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut dyn fmt::Write) -> Result<(), Failure> {
        write_text(stream, "Mes SectionPoint :\n", "Intf_Interference::Dump")?;
        for point in self.section_points.iter() {
            point.dump(2);
        }
        write_text(stream, "Mes SectionLine :\n", "Intf_Interference::Dump")?;
        for line in self.section_lines.iter() {
            line.dump(2, stream)?;
        }
        write_text(stream, "Mes TangentZone :\n", "Intf_Interference::Dump")?;
        for zone in self.tangent_zones.iter() {
            zone.dump(2);
        }
        Ok(())
    }
}
