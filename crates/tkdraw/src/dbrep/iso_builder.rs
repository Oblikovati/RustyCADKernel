// Rust translation of Draw/TKDraw/DBRep/DBRep_IsoBuilder.hxx/.cxx (constructor and
// LoadIsos) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DBRep_IsoBuilder`: the isoparametric curves of a face, trimmed by its boundary with
//! a `Geom2dHatch_Hatcher`.
use super::edge_face::{DBRepFace, FaceIso};
use crate::draw::DrawConsole;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, edge as to_edge};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::format_general;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom2d_hatch::{Hatcher, Intersector};
use rustycad_tkgeomalgo::hatch_gen::ErrorStatus;
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::{Direction2, Point2};

/// `HatcherConfusion2d` and `HatcherConfusion3d`.
const HATCHER_CONFUSION: f64 = 1.0e-8;

/// The p-curves of the face edges, in exploration order, keyed by the oriented edge
/// (`DataMapOfEdgePCurve`, whose default hasher compares with `IsEqual`).
pub type EdgePCurves = Vec<(Edge, Handle<dyn Geom2dCurve>)>;

/// One family of isos: their parameters and hatching numbers (0 when not added).
#[derive(Debug, Clone, Default)]
struct IsoFamily {
    parameters: Vec<f64>,
    hatchings: Vec<i32>,
}

/// `DBRep_IsoBuilder`: a hatcher loaded with the face boundary and `NbIsos` U and V
/// iso lines.
pub struct DBRepIsoBuilder {
    pub(crate) hatcher: Hatcher,
    infinite: f64,
    pub(crate) bounds: (f64, f64, f64, f64),
    u_isos: IsoFamily,
    v_isos: IsoFamily,
    nb_domains: i32,
}

impl DBRepIsoBuilder {
    /// `DBRep_IsoBuilder(TopologicalFace, Infinite, NbIsos)`: bounds infinite UV
    /// ranges by `infinite`, loads the (trimmed) p-curves and the gap segments,
    /// adds the isos and computes their domains. A face with a missing or degenerate
    /// p-curve keeps no iso (the C++ constructor returns early).
    pub fn new(face: &Face, infinite: f64, nb_isos: i32) -> Result<Self, Failure> {
        let count = usize::try_from(nb_isos).unwrap_or(0);
        let intersector = Intersector::new(precision::p_confusion(), precision::p_confusion());
        let family = IsoFamily {
            parameters: vec![0.0; count],
            hatchings: vec![0; count],
        };
        let mut builder = Self {
            hatcher: Hatcher::new(
                &intersector,
                HATCHER_CONFUSION,
                HATCHER_CONFUSION,
                true,
                false,
            ),
            infinite,
            bounds: bounded(uv_bounds(face)?, infinite),
            u_isos: family.clone(),
            v_isos: family,
            nb_domains: 0,
        };
        let Some(mut pcurves) = face_pcurves(face)? else {
            return Ok(builder);
        };
        builder.fill_gaps(face, &mut pcurves)?;
        for (edge, pcurve) in &pcurves {
            let adaptor = Geom2dAdaptorCurve::with_curve(pcurve)?;
            builder.hatcher.add_element(adaptor, edge.orientation());
        }
        builder.add_isos(nb_isos)?;
        builder.compute_domains()?;
        Ok(builder)
    }

    /// `NbDomains`: the total number of iso domains.
    pub fn nb_domains(&self) -> i32 {
        self.nb_domains
    }

    /// Adds `nb_isos` U isos (vertical lines) then V isos (horizontal lines) at the
    /// middles of equal steps, when a step exceeds the 3D confusion.
    fn add_isos(&mut self, nb_isos: i32) -> Result<(), Failure> {
        let (u_min, u_max, v_min, v_max) = self.bounds;
        let (delta_u, delta_v) = ((u_max - u_min).abs(), (v_max - v_min).abs());
        let confusion = delta_u.min(delta_v) * HATCHER_CONFUSION;
        self.hatcher.set_confusion3d(confusion);
        let step_u = delta_u / f64::from(nb_isos);
        if step_u > confusion {
            let direction = Direction2::new(0.0, 1.0)?;
            let lines = |u: f64| Point2::new(u, 0.0);
            self.u_isos = add_family(
                &mut self.hatcher,
                u_min,
                step_u,
                nb_isos,
                (lines, direction),
            )?;
        }
        let step_v = delta_v / f64::from(nb_isos);
        if step_v > confusion {
            let direction = Direction2::new(1.0, 0.0)?;
            let lines = |v: f64| Point2::new(0.0, v);
            self.v_isos = add_family(
                &mut self.hatcher,
                v_min,
                step_v,
                nb_isos,
                (lines, direction),
            )?;
        }
        Ok(())
    }

    /// `Trim()` then, iso by iso (U then V), the domains of the trimmed ones.
    fn compute_domains(&mut self) -> Result<(), Failure> {
        self.hatcher.trim()?;
        self.nb_domains = 0;
        for i in 0..self.u_isos.hatchings.len() {
            for index in [self.u_isos.hatchings[i], self.v_isos.hatchings[i]] {
                if index != 0 {
                    self.nb_domains += self.domains_of(index)?;
                }
            }
        }
        Ok(())
    }

    /// The domains of hatching `index` once trimmed, 0 when it failed.
    fn domains_of(&mut self, index: i32) -> Result<i32, Failure> {
        if !self.hatcher.trim_done(index)? || self.hatcher.trim_failed(index)? {
            return Ok(0);
        }
        self.hatcher.compute_domains_of(index)?;
        if !self.hatcher.is_done(index)? {
            return Ok(0);
        }
        self.hatcher.nb_domains(index)
    }

    /// `LoadIsos`: stores the iso domains in `face`, U isos first; an iso whose domains
    /// failed is reported on `out` and skipped. A domain open at one end extends past
    /// the face bounds by `Infinite`. Native quirk kept: the V isos also take their
    /// open ends from the V bounds.
    pub fn load_isos(&self, face: &mut DBRepFace, out: &dyn DrawConsole) -> Result<(), Failure> {
        let mut number = 0;
        for (family, kind) in [(&self.u_isos, IsoType::IsoU), (&self.v_isos, IsoType::IsoV)] {
            for (parameter, index) in family.parameters.iter().zip(&family.hatchings) {
                if *index == 0 {
                    continue;
                }
                if !self.hatcher.is_done(*index)? {
                    self.report_failure(kind, *parameter, *index, out)?;
                    continue;
                }
                for domain in 1..=self.hatcher.nb_domains(*index)? {
                    number += 1;
                    let (first, last) = self.domain_range(*index, domain)?;
                    face.set_iso(
                        number,
                        FaceIso {
                            kind,
                            parameter: *parameter,
                            first,
                            last,
                        },
                    )?;
                }
            }
        }
        Ok(())
    }

    /// The parameter range of domain `domain` of hatching `index`.
    fn domain_range(&self, index: i32, domain: i32) -> Result<(f64, f64), Failure> {
        let (_, _, v_min, v_max) = self.bounds;
        let domain = self.hatcher.domain(index, domain)?;
        let first = if domain.has_first_point() {
            domain.first_point()?.parameter()
        } else {
            v_min - self.infinite
        };
        let last = if domain.has_second_point() {
            domain.second_point()?.parameter()
        } else {
            v_max + self.infinite
        };
        Ok((first, last))
    }

    /// The `std::cout` report of an iso whose domains could not be computed.
    fn report_failure(
        &self,
        kind: IsoType,
        parameter: f64,
        index: i32,
        out: &dyn DrawConsole,
    ) -> Result<(), Failure> {
        let letter = if kind == IsoType::IsoU { "U" } else { "V" };
        let status = match self.hatcher.status(index)? {
            ErrorStatus::NoProblem => " No Problem",
            ErrorStatus::TrimFailure => " Trim Failure",
            ErrorStatus::TransitionFailure => " Transition Failure",
            ErrorStatus::IncoherentParity => " Incoherent Parity",
            ErrorStatus::IncompatibleStates => " Incompatible States",
        };
        let value = format_general(parameter, 6);
        out.write_out(&format!(
            "DBRep_IsoBuilder:: {letter} iso of parameter: {value}{status}\n"
        ));
        Ok(())
    }
}

/// The UV bounds with each infinite side replaced as the constructor does.
fn bounded(
    (mut u_min, mut u_max, mut v_min, mut v_max): (f64, f64, f64, f64),
    infinite: f64,
) -> (f64, f64, f64, f64) {
    (u_min, u_max) = bounded_range(u_min, u_max, infinite);
    (v_min, v_max) = bounded_range(v_min, v_max, infinite);
    (u_min, u_max, v_min, v_max)
}

/// One parameter range: `[-Infinite, Infinite]` when both ends are infinite, else the
/// infinite end at `Infinite` from the finite one.
fn bounded_range(min: f64, max: f64, infinite: f64) -> (f64, f64) {
    match (
        precision::is_negative_infinite(min),
        precision::is_positive_infinite(max),
    ) {
        (true, true) => (-infinite, infinite),
        (true, false) => (max - infinite, max),
        (false, true) => (min, min + infinite),
        (false, false) => (min, max),
    }
}

/// `nb` hatchings along `direction` through `origin(first + step / 2 + k * step)`.
fn add_family(
    hatcher: &mut Hatcher,
    first: f64,
    step: f64,
    nb: i32,
    (origin, direction): (impl Fn(f64) -> Point2, Direction2),
) -> Result<IsoFamily, Failure> {
    let mut family = IsoFamily::default();
    let mut parameter = first + step / 2.;
    for _ in 1..=nb {
        family.parameters.push(parameter);
        let line = Handle::new(Geom2dLine::from_point(origin(parameter), direction));
        let curve = Geom2dAdaptorCurve::with_curve(&line.cast::<dyn Geom2dCurve>())?;
        family.hatchings.push(hatcher.add_hatching(curve));
        parameter += step;
    }
    Ok(family)
}

/// The p-curves of the face edges, trimmed to the edge ranges as the constructor does;
/// `None` when an edge has no p-curve or an empty range (no iso is built then).
fn face_pcurves(face: &Face) -> Result<Option<EdgePCurves>, Failure> {
    let mut pcurves = EdgePCurves::new();
    let mut explorer = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        let edge = to_edge(explorer.current())?;
        explorer.next()?;
        let stored = BRepTool::curve_on_face(&edge, face)?;
        if stored.pcurve.is_null() || stored.first == stored.last {
            return Ok(None);
        }
        match trimmed_pcurve(stored.pcurve, stored.first, stored.last)? {
            PCurveRange::Whole(curve) => pcurves.push((edge, curve)),
            PCurveRange::Trimmed(curve) => pcurves.push((edge, curve)),
            PCurveRange::Empty => {}
            PCurveRange::OutOfRange => return Ok(None),
        }
    }
    Ok(Some(pcurves))
}

/// What the edge range does to a p-curve.
enum PCurveRange {
    /// The range is the whole curve.
    Whole(Handle<dyn Geom2dCurve>),
    /// A `Geom2d_TrimmedCurve` on the range.
    Trimmed(Handle<dyn Geom2dCurve>),
    /// The clamped range is empty: the edge is skipped.
    Empty,
    /// The range of a trimmed p-curve leaves its basis: no iso is built.
    OutOfRange,
}

/// The "Test if a TrimmedCurve is necessary" block: the p-curve itself on its own
/// range, else a trimmed curve on `[u1, u2]` clamped to a non-periodic curve.
fn trimmed_pcurve(
    pcurve: Handle<dyn Geom2dCurve>,
    mut u1: f64,
    mut u2: f64,
) -> Result<PCurveRange, Failure> {
    let tolerance = precision::p_confusion();
    let curve = pcurve
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "p-curve"))?;
    let (first, last) = (curve.first_parameter(), curve.last_parameter());
    if (first - u1).abs() <= tolerance && (last - u2).abs() <= tolerance {
        return Ok(PCurveRange::Whole(pcurve));
    }
    if !curve.is_periodic() {
        let trimmed = pcurve.cast::<Geom2dTrimmedCurve>();
        if let Some(trimmed) = trimmed.get() {
            let basis = trimmed.basis_curve();
            let (b_first, b_last) = (basis.first_parameter(), basis.last_parameter());
            if b_first - u1 > tolerance
                || b_first - u2 > tolerance
                || u1 - b_last > tolerance
                || u2 - b_last > tolerance
            {
                return Ok(PCurveRange::OutOfRange);
            }
        } else {
            u1 = clamp_to(u1, first, last, tolerance);
            u2 = clamp_to(u2, first, last, tolerance);
        }
    }
    if (u1 - u2).abs() <= tolerance {
        return Ok(PCurveRange::Empty);
    }
    let trimmed = Geom2dTrimmedCurve::new(&pcurve, u1, u2, true, true)?;
    Ok(PCurveRange::Trimmed(
        Handle::new(trimmed).cast::<dyn Geom2dCurve>(),
    ))
}

/// A parameter moved onto `[first, last]` when it is beyond by more than `tolerance`.
fn clamp_to(u: f64, first: f64, last: f64, tolerance: f64) -> f64 {
    let u = if first - u > tolerance { first } else { u };
    if u - last > tolerance { last } else { u }
}
