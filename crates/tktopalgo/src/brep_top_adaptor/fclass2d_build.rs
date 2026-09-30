// Rust translation of BRepTopAdaptor_FClass2d.cxx (constructor) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constructor of `BRepTopAdaptor_FClass2d`: the polygon of each wire sampled
//! from its pcurves, and the role of each wire from the sign of its area.
use super::fclass2d::{BRepTopAdaptorFClass2d, WireRole};
use super::fclass2d_polygon::WirePolygon;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Wire};
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::cslib::Class2d;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

/// `eps`: the deflection and area below which a single-edge wire is an outer wire.
const EPS: f64 = 1.0e-10;

/// How the sampling of the wires ended.
enum SamplingEnd {
    /// Every wire was sampled, or a bad wire stopped the loop (`anIsBadWire`).
    Completed { bad_wire: bool },
    /// An edge had no pcurve: the constructor returns at once.
    Aborted,
}

impl BRepTopAdaptorFClass2d {
    /// `BRepTopAdaptor_FClass2d(F, Tol)`: samples each wire of the face (oriented
    /// FORWARD) into a polygon classifier with the UV tolerance `tol_uv`. A wire whose
    /// edge count differs from its WireExplorer walk, that has at most three points or
    /// whose area/perimeter thickness is not representable makes every
    /// classification exact; an edge without pcurve ends the construction at once;
    /// `BRepTopAdaptorFClass2d::new(&face, 1e-9)?`.
    pub fn new(face: &Face, tol_uv: f64) -> Result<Self, Failure> {
        //-- dead end on surfaces defined on more than one period
        let forward = Face::unchecked(face.oriented(Orientation::Forward));
        let mut classifier = Self::unsampled(&forward, tol_uv);
        let surface = BRepAdaptorSurface::with_face(face, false)?;
        classifier.is_u_periodic = surface.is_u_periodic()?;
        classifier.is_v_periodic = surface.is_v_periodic()?;
        classifier.u_period = if classifier.is_u_periodic {
            surface.u_period()?
        } else {
            0.0
        };
        classifier.v_period = if classifier.is_v_periodic {
            surface.v_period()?
        } else {
            0.0
        };
        let SamplingEnd::Completed { bad_wire } = classifier.sample_wires()? else {
            return Ok(classifier);
        };
        if !classifier.tab_class.is_empty() {
            classifier.set_periodic_ranges(surface.get_type()?, bad_wire);
        }
        Ok(classifier)
    }

    /// The wire loop: one polygon (and role) per wire until a bad wire.
    fn sample_wires(&mut self) -> Result<SamplingEnd, Failure> {
        // Umin = Vmin = 0.0 (RealLast() once); Umax = Vmax = -Umin.
        (self.umin, self.vmin) = (0.0, 0.0);
        (self.umax, self.vmax) = (-self.umin, -self.umin);
        let mut wires = Explorer::new(&self.face, ShapeEnum::Wire, ShapeEnum::Shape)?;
        let mut bad_wire: bool = false;
        while wires.more() && !bad_wire {
            let wire: Wire = topods::wire(wires.current())?;
            let Some(polygon) = WirePolygon::sample(self, &wire)? else {
                return Ok(SamplingEnd::Aborted);
            };
            bad_wire = self.add_wire(&wire, polygon)?;
            wires.next()?;
        }
        Ok(SamplingEnd::Completed { bad_wire })
    }

    /// The classifier and the role of one sampled wire; true for a bad wire.
    fn add_wire(&mut self, wire: &Wire, mut polygon: WirePolygon) -> Result<bool, Failure> {
        if polygon.nb_edges_left != 0 {
            //-- on compte ++ with a normal explorer and with the Wire Explorer
            self.push_fallback(&polygon);
            self.tab_orien.push(WireRole::Invalid);
            return Ok(true);
        }
        if !polygon.wire_is_not_empty {
            return Ok(false);
        }
        if polygon.nbpnts <= 3 {
            self.tab_orien.push(WireRole::Invalid);
            self.push_polygon(&polygon);
            return Ok(true);
        }
        let Some(signed_area) = polygon.refine(self, wire)? else {
            self.tab_orien.push(WireRole::Invalid);
            self.push_fallback(&polygon);
            return Ok(true);
        };
        self.push_role(&polygon, signed_area);
        polygon.fleche_u = polygon.fleche_u.max_native(self.toluv);
        polygon.fleche_v = polygon.fleche_v.max_native(self.toluv);
        self.push_polygon(&polygon);
        Ok(false)
    }

    /// The role of a valid wire: outer when its signed area is negative, or for a
    /// single-edge wire without deflection nor area.
    fn push_role(&mut self, polygon: &WirePolygon, signed_area: f64) {
        let flat_single_edge: bool = polygon.nb_edges == 1
            && polygon.fleche_u < EPS
            && polygon.fleche_v < EPS
            && signed_area.abs() < EPS;
        let role = if flat_single_edge || signed_area < 0.0 {
            WireRole::Outer
        } else {
            WireRole::Inner
        };
        self.tab_orien.push(role);
    }

    /// `CSLib_Class2d(SeqPnt2d, FlecheU, FlecheV, Umin, Vmin, Umax, Vmax)` with the
    /// running face bounds.
    fn push_polygon(&mut self, polygon: &WirePolygon) {
        let points = polygon.points.to_array1_view();
        self.tab_class.push(Class2d::from_array(
            &points,
            polygon.fleche_u,
            polygon.fleche_v,
            self.umin,
            self.vmin,
            self.umax,
            self.vmax,
        ));
    }

    /// The classifier of a bad wire: two points at the origin (an uncertain polygon).
    fn push_fallback(&mut self, polygon: &WirePolygon) {
        let points = LinearVector::with_value(2, Point2::new(0.0, 0.0));
        let fallback = WirePolygon::fallback(points, polygon.fleche_u, polygon.fleche_v);
        self.push_polygon(&fallback);
    }

    /// The first wire's role becomes invalid when a wire was bad, then the periodic
    /// ranges [0, 2 PI] of the revolved surfaces (U) and of the torus (V).
    fn set_periodic_ranges(&mut self, kind: SurfaceType, bad_wire: bool) {
        //-- If an error was detected on a wire: set all TabOrien to -1
        if bad_wire {
            self.tab_orien[0] = WireRole::Invalid;
        }
        let revolved: bool = matches!(
            kind,
            SurfaceType::Cone
                | SurfaceType::Cylinder
                | SurfaceType::Torus
                | SurfaceType::Sphere
                | SurfaceType::SurfaceOfRevolution
        );
        // modified by NIZHNY-OFV  Thu May 31 14:24:10 2001: U1 = 0 and U2 = 2 PI
        // (the unused uuu = 2 PI - (Umax - Umin) was the old offset).
        (self.u1, self.u2) = if revolved {
            (0.0, 2.0 * PI)
        } else {
            (0.0, 0.0)
        };
        (self.v1, self.v2) = if kind == SurfaceType::Torus {
            (0.0, 2.0 * PI)
        } else {
            (0.0, 0.0)
        };
    }
}

/// `if (Fleche < Toluv) Fleche = Toluv;` as a method on the deflections.
trait RaisedTo {
    fn max_native(self, floor: f64) -> f64;
}

impl RaisedTo for f64 {
    fn max_native(self, floor: f64) -> f64 {
        if self < floor { floor } else { self }
    }
}
