//! GeomPlate_BuildPlateSurface and GeomPlate_MakeApprox probe operations, the Rust twin
//! of tools/oracle/tkgeomalgo_probe_geomplate_build.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, text, xy, xyz};
use super::geom_api_surface::bsurface_out;
use super::geom_plate::{
    bounds, curve_constraint, curve2d, eval, plate, point_constraint, surface,
};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{HArray1, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeomalgo::geom_plate::{
    GeomPlateBuildPlateSurface, GeomPlateCurveConstraint, GeomPlateMakeApprox,
    GeomPlatePlateG0Criterion, GeomPlatePlateG1Criterion, GeomPlatePointConstraint,
    GeomPlateSurface,
};
use rustycad_tkgeombase::adv_app2var::{AdvApp2VarCriterionRepartition, AdvApp2VarCriterionType};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Xy, Xyz};

fn live(c: &Handle<GeomPlateCurveConstraint>) -> &GeomPlateCurveConstraint {
    c.get().expect("probe constraint")
}

const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

fn harray<T: Default + Clone + Send + Sync + 'static>(values: Vec<T>) -> Handle<HArray1<T>> {
    let array: HArray1<T> = HArray1::with_bounds(1, values.len() as i32).expect("probe array");
    for (i, v) in (1..).zip(values) {
        array.change_array1()[i] = v;
    }
    Handle::new(array)
}

/// `ga_gp_builder`: the builder of one of the three constructors with its constraints.
fn builder(r: &mut Reader) -> Result<GeomPlateBuildPlateSurface, Failure> {
    let (ctor, degree, nbpts, nbiter) = (r.integer(), r.integer(), r.integer(), r.integer());
    let (tol2d, tol3d, tolang) = (r.real(), r.real(), r.real());
    let (aniso, nbbounds) = (r.flag(), r.integer());
    let init = surface(r)?;
    let nc = r.integer();
    let curves: Vec<Handle<GeomPlateCurveConstraint>> = (0..nc)
        .map(|_| curve_constraint(r))
        .collect::<Result<_, _>>()?;
    let np = r.integer();
    let mut points: Vec<Handle<GeomPlatePointConstraint>> = Vec::new();
    for _ in 0..np {
        points.push(point_constraint(r)?.1);
    }
    let mut b = match ctor {
        1 => GeomPlateBuildPlateSurface::with_surface(
            &init, degree, nbpts, nbiter, tol2d, tol3d, tolang, 0.1, aniso,
        )?,
        2 => {
            let npt = harray(curves.iter().map(|c| live(c).nb_points()).collect());
            let tang = harray(curves.iter().map(|c| live(c).order()).collect());
            let tab: Handle<HArray1<Handle<dyn Adaptor3dCurve>>> =
                harray(curves.iter().map(|c| live(c).curve3d()).collect());
            GeomPlateBuildPlateSurface::with_curves(
                &npt, &tab, &tang, degree, nbiter, tol2d, tol3d, tolang, 0.1, aniso,
            )?
        }
        _ => GeomPlateBuildPlateSurface::new(
            degree, nbpts, nbiter, tol2d, tol3d, tolang, 0.1, aniso,
        )?,
    };
    if ctor != 1 && init.is_some() {
        b.load_init_surface(&init);
    }
    if ctor != 2 {
        curves.iter().for_each(|c| b.add_curve(c));
    }
    points.iter().for_each(|p| b.add_point(p));
    if nbbounds != 0 {
        b.set_nb_bounds(nbbounds);
    }
    Ok(b)
}

fn type_name(s: &Handle<dyn GeomSurface>) {
    match s.dynamic_type() {
        Some(t) => text(t.name().as_bytes()),
        None => text(b"null"),
    }
}

fn int_array(a: Result<Handle<HArray1<i32>>, Failure>) {
    attempt(|| {
        let a = a?;
        let a = a.get().expect("array");
        let array = a.array1();
        int(array.length());
        array.iter().for_each(|v| int(*v));
        Ok(())
    });
}

fn build_curves(b: &mut GeomPlateBuildPlateSurface, nc: i32, gaps: bool) -> Result<(), Failure> {
    for i in 1..=nc {
        let cc = b.curve_constraint(i)?;
        let cc = cc.get().expect("constraint");
        int(cc.nb_points());
        attempt(|| {
            curve2d(
                &cc.curve2d_on_surf()?,
                cc.first_parameter()?,
                cc.last_parameter()?,
            );
            Ok(())
        });
        if !gaps {
            continue;
        }
        for value in [b.g0_error_of(i), b.g1_error_of(i), b.g2_error_of(i)] {
            attempt(|| {
                real(value?);
                Ok(())
            });
        }
    }
    int_array(b.sense());
    int_array(b.order());
    attempt(|| {
        let c2 = b.curves2d()?;
        let c2 = c2.get().expect("array");
        let array = c2.array1();
        int(array.length());
        for c in array.iter() {
            flag(c.is_null());
            if let Some(curve) = c.get() {
                attempt(|| {
                    curve2d(c, curve.first_parameter(), curve.last_parameter());
                    Ok(())
                });
            }
        }
        Ok(())
    });
    Ok(())
}

fn build_contours(b: &mut GeomPlateBuildPlateSurface) {
    attempt(|| {
        let mut s2: Sequence<Xy> = Sequence::default();
        b.disc2d_contour(4, &mut s2)?;
        int(s2.length());
        s2.iter().for_each(|p| xy(*p));
        Ok(())
    });
    for order in 0..2 {
        attempt(|| {
            let mut s3: Sequence<Xyz> = Sequence::default();
            b.disc3d_contour(4, order, &mut s3)?;
            int(s3.length());
            s3.iter().for_each(|p| xyz(*p));
            Ok(())
        });
    }
}

fn surface_results(b: &GeomPlateBuildPlateSurface, uv: &[f64]) {
    flag(b.is_done());
    let s = b.surface();
    flag(s.is_null());
    let init = b.surf_init();
    type_name(&init);
    if let Some(init) = init.get() {
        bounds(init);
    }
    if let Some(s) = s.get() {
        bounds(s);
        attempt(|| {
            let (a, bb, c, d) = s.real_bounds()?;
            [a, bb, c, d].into_iter().for_each(real);
            Ok(())
        });
        uv.chunks(2).for_each(|p| eval(s, p[0], p[1]));
    }
}

/// The global errors, once a check ran (see the native probe).
fn global_errors(b: &GeomPlateBuildPlateSurface, nc: i32) {
    if nc > 0 && b.is_done() && b.surface().is_some() {
        real(b.g0_error());
        real(b.g1_error());
        real(b.g2_error());
    }
}

fn build_body(r: &mut Reader) -> Result<(), Failure> {
    let mut b = builder(r)?;
    let ns = r.integer();
    let uv: Vec<f64> = (0..2 * ns).map(|_| r.real()).collect();
    let gaps = r.flag();
    attempt(|| b.perform(&ProgressRange::default()));
    surface_results(&b, &uv);
    let mut nc = 0;
    loop {
        match b.curve_constraint(nc + 1) {
            Ok(_) => nc += 1,
            Err(e) => {
                text(e.exception_type().as_bytes());
                break;
            }
        }
    }
    global_errors(&b, nc);
    attempt(|| b.curve_constraint(0).map(|_| ()));
    build_curves(&mut b, nc, gaps)?;
    let mut np = 0;
    loop {
        match b.point_constraint(np + 1) {
            Ok(p) => {
                np += 1;
                let p = p.get().expect("constraint");
                flag(p.has_pnt2d_on_surf());
                pnt2d(p.pnt2d_on_surf());
            }
            Err(e) => {
                text(e.exception_type().as_bytes());
                break;
            }
        }
    }
    attempt(|| b.point_constraint(0).map(|_| ()));
    if b.surface().is_some() {
        build_contours(&mut b);
    }
    Ok(())
}

/// `gplate_build`: Perform, then every result of the builder.
pub fn build_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| build_body(&mut r));
    end();
}

/// The targets of an explicit criterion: the plate points (mode 1) or normals (mode 2)
/// at its constraints, moved by `offset`.
fn criterion_targets(
    gs: &GeomPlateSurface,
    mode: i32,
    offset: Xyz,
) -> Result<(Sequence<Xy>, Sequence<Xyz>), Failure> {
    let mut s2: Sequence<Xy> = Sequence::default();
    let mut s3: Sequence<Xyz> = Sequence::default();
    gs.constraints(&mut s2);
    for p in s2.iter() {
        let d1 = gs.d1(p.x(), p.y())?;
        let target = if mode == 1 {
            d1.point.xyz()
        } else {
            *(d1.d1u ^ d1.d1v).xyz()
        };
        s3.append(target + offset);
    }
    Ok((s2, s3))
}

fn approx_body(r: &mut Reader) -> Result<(), Failure> {
    let init = surface(r)?;
    let plate = plate(r)?;
    let gs = Handle::new(GeomPlateSurface::new(&init, &plate));
    let (mode, tol3d) = (r.integer(), r.real());
    let (nbmax, dgmax) = (r.integer(), r.integer());
    let cont = SHAPES[r.integer() as usize];
    let enlarge = r.real();
    let mut errors = true;
    let app = if mode == 0 {
        let (dmax, crit) = (r.real(), r.integer());
        errors = (-1..=1).contains(&crit);
        GeomPlateMakeApprox::with_order(&gs, tol3d, nbmax, dgmax, dmax, crit, cont, enlarge)?
    } else {
        let maximum = r.real();
        let kind = [
            AdvApp2VarCriterionType::Absolute,
            AdvApp2VarCriterionType::Relative,
        ][r.integer() as usize];
        let rep = [
            AdvApp2VarCriterionRepartition::Regular,
            AdvApp2VarCriterionRepartition::Incremental,
        ][r.integer() as usize];
        let offset = r.xyz();
        let (s2, s3) = criterion_targets(gs.get().expect("plate surface"), mode, offset)?;
        if mode == 1 {
            let crit = GeomPlatePlateG0Criterion::new(&s2, &s3, maximum, kind, rep);
            GeomPlateMakeApprox::with_criterion(&gs, &crit, tol3d, nbmax, dgmax, cont, enlarge)?
        } else {
            let crit = GeomPlatePlateG1Criterion::new(&s2, &s3, maximum, kind, rep);
            GeomPlateMakeApprox::with_criterion(&gs, &crit, tol3d, nbmax, dgmax, cont, enlarge)?
        }
    };
    bsurface_out(&app.surface())?;
    if errors {
        real(app.approx_error());
        real(app.criterion_error());
    }
    Ok(())
}

/// `gplate_approx`: GeomPlate_MakeApprox of a plate surface.
pub fn approx_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| approx_body(&mut r));
    end();
}
