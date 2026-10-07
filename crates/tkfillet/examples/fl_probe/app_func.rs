//! The sweep functions and approximations of BRepBlend (`fl_app_func`); the twin of the
//! approximation case of tools/oracle/tkfillet_probe_brepblend.cxx.
use super::blend_func::{guide, surface, vec3};
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::blend::BlendPoint;
use rustycad_tkfillet::blend_func::{ConstRad, SectionShape};
use rustycad_tkfillet::brep_blend::{
    AppSurfaceOptions, BRepBlendAppFunc, BRepBlendAppSurf, BRepBlendAppSurface, BRepBlendLine,
    BlendSectionGenerator, SharedAppFunction,
};
use rustycad_tkfillet::chfi_ds::{lock, shared};
use rustycad_tkgeomalgo::app_blend::AppBlendApprox;
use rustycad_tkgeombase::approx::{SweepD1, SweepFunction};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

fn filled<T: Clone + Default>(n: i32) -> Array1<'static, T> {
    let mut array = Array1::with_bounds(1, n).expect("array");
    array.init(T::default());
    array
}

fn vec2(v: Vector2) {
    real(v.x());
    real(v.y());
}

fn reals(a: &Array1<'_, f64>) {
    (a.lower()..=a.upper()).for_each(|i| real(*a.value(i).expect("entry")));
}

fn ints(a: &Array1<'_, i32>) {
    (a.lower()..=a.upper()).for_each(|i| real(f64::from(*a.value(i).expect("entry"))));
}

/// `fl_approx`: the approximated surface.
fn approx(a: &dyn AppBlendApprox) {
    flag(a.is_done());
    if !a.is_done() {
        return;
    }
    attempt(|| a.surf_shape().map(|s| s.into_iter().for_each(int)));
    attempt(|| {
        a.surf_poles().map(|p| {
            for i in p.lower_row()..=p.upper_row() {
                (p.lower_col()..=p.upper_col()).for_each(|j| pnt(*p.value(i, j).expect("pole")));
            }
        })
    });
    attempt(|| {
        a.surf_weights().map(|w| {
            for i in w.lower_row()..=w.upper_row() {
                (w.lower_col()..=w.upper_col()).for_each(|j| real(*w.value(i, j).expect("weight")));
            }
        })
    });
    attempt(|| a.surf_u_knots().map(reals));
    attempt(|| a.surf_v_knots().map(reals));
    attempt(|| a.surf_u_mults().map(ints));
    attempt(|| a.surf_v_mults().map(ints));
    attempt(|| a.nb_curves2d().map(int));
    attempt(|| {
        for k in 1..=a.nb_curves2d()? {
            let poles = a.curve2d_poles(k)?;
            (poles.lower()..=poles.upper()).for_each(|i| pnt2d(*poles.value(i).expect("pole")));
        }
        Ok(())
    });
    attempt(|| {
        a.tol_reached().map(|(t3, t2)| {
            real(t3);
            real(t2);
        })
    });
}

fn right_angle() -> (
    Handle<dyn rustycad_tkg3d::adaptor3d::Adaptor3dSurface>,
    Handle<dyn rustycad_tkg3d::adaptor3d::Adaptor3dSurface>,
    Handle<dyn rustycad_tkg3d::adaptor3d::Adaptor3dCurve>,
) {
    let s1 = surface(&mut Reader::new(&[
        0., 0., 0., 0., 0., 0., 1., 1., 0., 0., 0., 0.,
    ]));
    let s2 = surface(&mut Reader::new(&[
        0., 0., 0., 0., 0., 1., 0., 1., 0., 0., 0., 0.,
    ]));
    let c = guide(&mut Reader::new(&[
        0., 0., 0., 0., 1., 0., 0., 0., 1., 0., 0.,
    ]));
    (s1, s2, c)
}

/// The sweep function members of `fl_app_func`.
fn sweep_members(
    af: &BRepBlendAppFunc,
    line: &rustycad_tkfillet::brep_blend::BRepBlendLineHandle,
    (tparam, t0): (f64, f64),
) -> Result<(), Failure> {
    let (np, nk, deg) = af.section_shape()?;
    let n2 = af.nb_2d_curves()?;
    [np, nk, deg, n2].into_iter().for_each(int);
    flag(af.is_rational()?);
    pnt(af.barycentre_of_surf()?);
    attempt(|| af.maximal_section().map(real));
    attempt(|| af.nb_intervals(Shape::C1).map(int));
    let (mut poles, mut p2, mut w, mut dw) = (
        filled::<Point3>(np),
        filled::<Point2>(n2),
        filled::<f64>(np),
        filled::<f64>(np),
    );
    let (mut dp, mut dp2) = (filled::<Vector3>(np), filled::<Vector2>(n2));
    attempt(|| {
        flag(af.d0(tparam, t0, t0, &mut poles, &mut p2, &mut w)?);
        print_section(&poles, &p2, &w);
        Ok(())
    });
    attempt(|| {
        let values = SweepD1 {
            poles: &mut poles,
            d_poles: &mut dp,
            poles2d: &mut p2,
            d_poles2d: &mut dp2,
            weights: &mut w,
            d_weights: &mut dw,
        };
        flag(af.d1(tparam, t0, t0, values)?);
        (poles.lower()..=poles.upper()).for_each(|i| pnt(*poles.value(i).expect("pole")));
        (dp.lower()..=dp.upper()).for_each(|i| vec3(*dp.value(i).expect("derivative")));
        (p2.lower()..=p2.upper()).for_each(|i| pnt2d(*p2.value(i).expect("pole")));
        (dp2.lower()..=dp2.upper()).for_each(|i| vec2(*dp2.value(i).expect("derivative")));
        reals(&w);
        reals(&dw);
        Ok(())
    });
    int(lock(line).nb_points());
    let mut tol = filled::<f64>(np);
    attempt(|| {
        af.get_tolerance(1.0e-4, 1.0e-5, 1.0e-3, &mut tol)
            .map(|()| reals(&tol))
    });
    Ok(())
}

fn print_section(poles: &Array1<'_, Point3>, p2: &Array1<'_, Point2>, w: &Array1<'_, f64>) {
    (poles.lower()..=poles.upper()).for_each(|i| pnt(*poles.value(i).expect("pole")));
    (p2.lower()..=p2.upper()).for_each(|i| pnt2d(*p2.value(i).expect("pole")));
    reals(w);
}

/// `fl_app_func radius choix shape npts t0 dt tparam tol3d tol2d continuity`.
pub fn app_func_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (radius, choix, shape, npts) = (r.real(), r.integer(), r.integer(), r.integer());
    let (t0, dt, tparam, tol3d, tol2d, cont) = (
        r.real(),
        r.real(),
        r.real(),
        r.real(),
        r.real(),
        r.integer(),
    );
    let (s1, s2, c) = right_angle();
    let mut f = ConstRad::new(&s1, &s2, &c).expect("function");
    f.set_radius(radius, choix);
    f.set_shape(
        [
            SectionShape::Rational,
            SectionShape::QuasiAngular,
            SectionShape::Polynomial,
            SectionShape::Linear,
        ][shape as usize],
    );
    let mut line = BRepBlendLine::new();
    for i in 0..npts {
        let t = t0 + f64::from(i) * dt;
        line.append(BlendPoint::on_surfaces(
            Point3::new(t, radius, 0.),
            Point3::new(t, 0., radius),
            t,
            [t, radius, t, -radius],
        ));
    }
    let line = shared(line);
    let func: SharedAppFunction = shared(f);
    attempt(|| {
        let af = BRepBlendAppFunc::new(&line, &func, tol3d, tol2d)?;
        sweep_members(&af, &line, (tparam, t0))?;
        let (first, last) = (t0, t0 + f64::from(npts - 1) * dt);
        let handle: Handle<dyn SweepFunction> = Handle::new(af).cast();
        let continuity = [Shape::C0, Shape::C1, Shape::C2][cont as usize];
        let options = AppSurfaceOptions {
            tol3d,
            tol2d,
            tol_angular: 1.0e-2,
            continuity,
            degmax: 11,
            segmax: 50,
        };
        attempt(|| BRepBlendAppSurface::new(&handle, first, last, options).map(|app| approx(&app)));
        Ok(())
    });
    attempt(|| {
        let mut app = BRepBlendAppSurf::new(3, 8, tol3d, tol2d, 5, false);
        let mut guard = lock(&func);
        let mut generator = BlendSectionGenerator::new(&mut *guard);
        app.perform(Some(&*lock(&line)), &mut generator, true)?;
        approx(&app);
        Ok(())
    });
    end();
}
