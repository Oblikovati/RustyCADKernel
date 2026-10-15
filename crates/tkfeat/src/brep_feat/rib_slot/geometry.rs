// Rust translation of the geometric helpers of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_RibSlot.cxx (CheckPoint, Normal, IntPar,
// EdgeExtention, ChoiceOfFaces, HeightMax) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry of the ribs and slots: the point inside the material beside the wire,
//! the normal of a face, the parameter of a point on a conic, the extension of an edge,
//! the face first met around a point of the wire, and the bound of the basis box.
use super::RibSlot;
use crate::errors::null_object;
use crate::loc_ope::CSIntersector;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBoundedCurve, GeomCircle, GeomCurve, GeomElementarySurface, GeomEllipse, GeomGeometry,
    GeomHyperbola, GeomLine, GeomParabola, GeomPlane, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::cslib;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;
use std::f64::consts::PI;

/// The 3D curve of an edge and its range (Standard_NullObject without curve).
pub(super) fn edge_curve(e: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let (c, f, l) = BRepTool::curve(e)?;
    if c.is_null() {
        return Err(null_object("BRepFeat_RibSlot", "edge without 3D curve"));
    }
    Ok((c, f, l))
}

/// The curve behind a handle (Standard_NullObject when null).
pub(super) fn live(c: &Handle<dyn GeomCurve>) -> Result<&dyn GeomCurve, Failure> {
    c.get()
        .ok_or_else(|| null_object("BRepFeat_RibSlot", "null curve"))
}

/// The line through `p` along `v` (`new Geom_Line(P, V)`, the vector normalized).
pub(super) fn line(
    p: Point3,
    v: rustycad_tkmath::gp::Vector3,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(Handle::new(GeomLine::from_point(p, Direction3::from_vector(v)?)).cast())
}

/// `IntPar(C, P)`: the parameter of `p` on the line or conic `c` (0 for another curve
/// or a null one); `int_par(&line, p)?`.
pub fn int_par(c: &Handle<dyn GeomCurve>, p: Point3) -> Result<f64, Failure> {
    if c.is_null() {
        return Ok(0.);
    }
    let ac = GeomAdaptorCurve::with_curve(c)?;
    Ok(match ac.get_type()? {
        CurveType::Line => elclib::parameter(&ac.line()?, p)?,
        CurveType::Circle => elclib::parameter(&ac.circle()?, p)?,
        CurveType::Ellipse => elclib::parameter(&ac.ellipse()?, p)?,
        CurveType::Hyperbola => elclib::parameter(&ac.hyperbola()?, p)?,
        CurveType::Parabola => elclib::parameter(&ac.parabola()?, p)?,
        _ => 0.,
    })
}

/// `ChoiceOfFaces(faces, cc, par, bnd, Pln)`: among `faces`, the one met first by the
/// eight lines turned around the normal of the plane from the tangent of `cc` at `par`
/// (null when none is met); `choice_of_faces(&mut faces, &c, 0.5, 1., &pln)?`.
pub fn choice_of_faces(
    faces: &mut List<Shape>,
    cc: &Handle<dyn GeomCurve>,
    par: f64,
    _bnd: f64,
    pln: &Handle<GeomPlane>,
) -> Result<Face, Failure> {
    let mut fff = Face::default();
    let (pp, tgt) = live(cc)?.d1(par)?;
    let l1 = GeomLine::from_point(pp, Direction3::from_vector(tgt)?);
    let pln = pln
        .get()
        .ok_or_else(|| null_object("BRepFeat_RibSlot::ChoiceOfFaces", "null plane"))?;
    let axe = Axis1::new(pp, pln.position().direction());
    let mut scur = Sequence::default();
    let counter = 8;
    for i in 1..=counter {
        let l: Handle<dyn GeomCurve> = l1.rotated(axe, (i as f64) * PI / 9.)?.cast();
        scur.append(l);
    }
    let mut best = f64::MAX;
    for f in faces.iter() {
        let mut asi = CSIntersector::with_shape(f);
        asi.perform_curves(&scur)?;
        if !asi.is_done() {
            continue;
        }
        for jj in 1..=counter {
            if asi.nb_points(jj)? >= 1 {
                let app = asi.point(jj, 1)?.parameter();
                if app >= 0. && app < best {
                    best = app;
                    fff = topods::face(f)?;
                }
            }
        }
    }
    Ok(fff)
}

/// Whether the curve is a line or a conic (`Geom_Line`, `Geom_Circle`, ...).
fn is_conic(cu: &Handle<dyn GeomCurve>) -> bool {
    !cu.cast::<GeomLine>().is_null()
        || !cu.cast::<GeomCircle>().is_null()
        || !cu.cast::<GeomEllipse>().is_null()
        || !cu.cast::<GeomHyperbola>().is_null()
        || !cu.cast::<GeomParabola>().is_null()
}

impl RibSlot {
    /// `CheckPoint(e, bnd, Pln)`: the middle of the edge moved a tenth of its tangent
    /// across the plane normal, to the material side of the wire;
    /// `rib.check_point(&e, &pln)?`.
    pub(crate) fn check_point(&self, e: &Edge, pln: &Handle<GeomPlane>) -> Result<Point3, Failure> {
        let (cc, f, l) = edge_curve(e)?;
        let par = (f + l) / 2.;
        let (mut pp, mut tgt) = live(&cc)?.d1(par)?;
        if e.orientation() == Orientation::Reversed {
            tgt.reverse();
        }
        let pln = pln
            .get()
            .ok_or_else(|| null_object("BRepFeat_RibSlot::CheckPoint", "null plane"))?;
        let d = -tgt.crossed(rustycad_tkmath::gp::Vector3::from(
            pln.pln().position().direction(),
        )) / 10.;
        pp.translate(d);
        Ok(pp)
    }

    /// `Normal(F, P)`: the normal of the face at `p` on a plane, cylinder, cone or torus
    /// (reversed for a forward face), X otherwise; `rib.normal(&f, p)?`.
    pub(crate) fn normal(&self, f: &Face, p: Point3) -> Result<Direction3, Failure> {
        let as_ = BRepAdaptorSurface::with_face(f, true)?;
        let (u, v) = match as_.get_type()? {
            SurfaceType::Plane => elslib::parameters(&as_.plane()?, p),
            SurfaceType::Cylinder => elslib::parameters(&as_.cylinder()?, p),
            SurfaceType::Cone => elslib::parameters(&as_.cone()?, p),
            SurfaceType::Torus => elslib::parameters(&as_.torus()?, p),
            _ => return Direction3::new(1., 0., 0.),
        };
        let d1 = as_.d1(u, v)?;
        // the status is not checked natively: an undefined normal stays the default X
        let res = cslib::normal(d1.d1u, d1.d1v, precision::confusion())?;
        let mut n = res.normal.unwrap_or_default();
        if f.orientation() == Orientation::Forward {
            n.reverse();
        }
        Ok(n)
    }

    /// `EdgeExtention(e, bnd, FirstLast)`: the edge lengthened before its start
    /// (`first_last`) or after its end, by a tenth of `bnd` in parameter on a line or a
    /// conic, else to the point at a thousandth of `bnd` along its end tangent.
    pub(crate) fn edge_extention(
        &self,
        e: &mut Edge,
        bnd: f64,
        first_last: bool,
    ) -> Result<(), Failure> {
        let (cu, f, l) = edge_curve(e)?;
        let mut c: Handle<dyn GeomBoundedCurve> =
            Handle::new(GeomTrimmedCurve::new(&cu, f, l, true, true)?).cast();
        let ee = if is_conic(&cu) {
            let (a, b) = if first_last {
                (f - bnd / 10., l)
            } else {
                (f, l + bnd / 10.)
            };
            BRepLibMakeEdge::from_curve_params(&cu, a, b)?.edge()?
        } else if first_last {
            let (pnt, vct) = c
                .get()
                .ok_or_else(|| null_object("EdgeExtention", "null"))?
                .d1(f)?;
            let pt = live(&line(pnt, -vct)?)?.d0(bnd / 1000.)?;
            geom_lib::extend_curve_to_point(&mut c, pt, 1, false)?;
            let last = BRepTool::pnt(&topexp::last_vertex(e, true))?;
            BRepLibMakeEdge::from_curve_points(&c.cast(), pt, last)?.edge()?
        } else {
            let (pnt, vct) = c
                .get()
                .ok_or_else(|| null_object("EdgeExtention", "null"))?
                .d1(l)?;
            let pt = live(&line(pnt, vct)?)?.d0(bnd / 1000.)?;
            geom_lib::extend_curve_to_point(&mut c, pt, 1, true)?;
            let first = BRepTool::pnt(&topexp::first_vertex(e, true))?;
            BRepLibMakeEdge::from_curve_points(&c.cast(), first, pt)?.edge()?
        };
        *e = ee;
        Ok(())
    }

    /// `HeightMax(theSbase, theSUntil, p1, p2)`: the largest box coordinate of the basis
    /// and the shape Until, with the corners moved twice that away (natively from the
    /// coordinates in the order xmin, xmax, ymin and ymax, ymin... as stored);
    /// `rib.height_max(&sbase, &until)?`.
    pub(crate) fn height_max(
        &self,
        the_sbase: &Shape,
        the_s_until: &Shape,
    ) -> Result<(f64, Point3, Point3), Failure> {
        let mut b = BoundingBox3::default();
        BRepBndLib::add(the_sbase, &mut b, true)?;
        if !the_s_until.is_null() {
            BRepBndLib::add(the_s_until, &mut b, true)?;
        }
        let [xmin, ymin, zmin, xmax, ymax, zmax] = b.get()?;
        let c = [xmin, xmax, ymin, ymax, zmin, zmax];
        let mut bnd = c[0];
        for v in c {
            if v > bnd {
                bnd = v;
            }
        }
        let p1 = Point3::new(c[0] - 2. * bnd, c[1] - 2. * bnd, c[2] - 2. * bnd);
        let p2 = Point3::new(c[3] + 2. * bnd, c[4] + 2. * bnd, c[5] + 2. * bnd);
        Ok((bnd, p1, p2))
    }
}
