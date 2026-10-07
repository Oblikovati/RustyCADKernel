//! The ChFi3d operations that run without the TKBool data structure
//! (`fl_hatching_domains`); the twin of tools/oracle/tkfillet_probe_chfi3d.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi3d::compute_hatching_domains;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_hatch::{Hatcher, Intersector};
use rustycad_tkgeombase::gc::MakeSegment2d;
use rustycad_tkmath::gp::{Axis2d, Point2};

/// The hatcher with the closed polygon of the points as elements.
fn polygon_hatcher(points: &[Point2], tol: f64, keep: bool) -> Result<Hatcher, Failure> {
    let mut hatcher = Hatcher::new(&Intersector::new(tol, tol), tol, tol, false, keep);
    for (i, p) in points.iter().enumerate() {
        let next = points[(i + 1) % points.len()];
        let segment: Handle<dyn Geom2dCurve> =
            MakeSegment2d::from_points(*p, next)?.value()?.cast();
        hatcher.add_element_curve(&segment, Orientation::Forward)?;
    }
    Ok(hatcher)
}

/// The points of the hatching (parameter, segment flags), then its domains.
fn hatching_state(hatcher: &Hatcher, ih: i32) -> Result<(), Failure> {
    int(hatcher.nb_points(ih)?);
    for i in 1..=hatcher.nb_points(ih)? {
        let point = hatcher.point(ih, i)?;
        real(point.parameter());
        flag(point.segment_beginning());
        flag(point.segment_end());
    }
    attempt(|| {
        int(hatcher.nb_domains(ih)?);
        for i in 1..=hatcher.nb_domains(ih)? {
            let domain = hatcher.domain(ih, i)?;
            flag(domain.has_first_point());
            if domain.has_first_point() {
                real(domain.first_point()?.parameter());
            }
            flag(domain.has_second_point());
            if domain.has_second_point() {
                real(domain.second_point()?.parameter());
            }
        }
        Ok(())
    });
    Ok(())
}

/// `fl_hatching_domains`: see tools/oracle/tkfillet_probe_chfi3d.cxx.
pub fn hatching_domains_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let n = r.integer();
    let points: Vec<Point2> = (0..n).map(|_| r.pnt2d()).collect();
    let (p, d, tol, keep) = (r.pnt2d(), r.dir2d(), r.real(), r.flag());
    let mut hatcher = polygon_hatcher(&points, tol, keep).expect("polygon elements");
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(p, d))).cast();
    let adaptor = Geom2dAdaptorCurve::with_curve(&line).expect("hatching adaptor");
    let ih = hatcher.trim_curve(adaptor).expect("trimmed hatching");
    attempt(|| {
        flag(compute_hatching_domains(&mut hatcher, ih)?);
        Ok(())
    });
    hatching_state(&hatcher, ih).expect("hatching state");
    end();
}
