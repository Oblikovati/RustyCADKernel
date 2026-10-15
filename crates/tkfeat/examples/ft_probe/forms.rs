//! The BRepFeat form probes (`ft_hole`, `ft_lform`, `ft_rform`, `ft_gluer`, `ft_split`);
//! the twin of tools/oracle/tkfeat_probe_forms.cxx.
use super::common::{Reader, attempt, end, flag, int};
use super::shapes::{basis, face_at, ft_box, polygon_wire, result, shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfeat::brep_feat::{
    BRepFeatGluer, BRepFeatMakeCylindricalHole, BRepFeatMakeLinearForm, BRepFeatMakeRevolutionForm,
    BRepFeatSplitShape, Status,
};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::gp::{Axis1, Direction3, Plane3, Point3, Vector3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};

/// `ft_hole basis x method r a b cont`.
pub fn hole_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, x, method, rad, a, b, cont) = (
        r.integer(),
        r.real(),
        r.integer(),
        r.real(),
        r.real(),
        r.real(),
        r.flag(),
    );
    attempt(|| {
        let mut h = BRepFeatMakeCylindricalHole::new();
        h.init(
            &basis(kind)?,
            Axis1::new(Point3::new(x, 5., 12.), Direction3::new(0., 0., -1.)?),
        );
        match method {
            0 => h.perform(rad)?,
            1 => h.perform_thru_next(rad, cont)?,
            2 => h.perform_until_end(rad, cont)?,
            3 => h.perform_between(rad, a, b, cont)?,
            _ => h.perform_blind(rad, a, cont)?,
        }
        int(h.status() as i32);
        h.build()?;
        int(h.status() as i32);
        if h.status() == Status::NoError {
            shape(h.shape())?;
        }
        Ok(())
    });
    end();
}

/// `ft_form`: done, the status error, then the shape and the end faces when done.
macro_rules! form {
    ($f:expr) => {{
        flag($f.is_done());
        int($f.rib().current_status_error() as i32);
        if $f.is_done() {
            let s = $f.shape()?.clone();
            shape(&s)?;
            int($f.rib().first_shape()?.len() as i32);
            int($f.rib().last_shape()?.len() as i32);
        }
    }};
}

/// The plane through `p` of normal `n`.
fn plane(p: Point3, n: Direction3) -> Result<Handle<GeomPlane>, Failure> {
    Ok(Handle::new(GeomPlane::from_pln(Plane3::new(p, n)?)))
}

/// `ft_lform mode slide back h`.
pub fn lform_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, slide, back, h) = (r.integer(), r.flag(), r.flag(), r.real());
    attempt(|| {
        let w = polygon_wire(&[
            (2., 5., 10.),
            (2., 5., 10. + h),
            (8., 5., 10. + h),
            (8., 5., 10.),
        ])?;
        let pl = plane(Point3::new(0., 5., 0.), Direction3::new(0., 1., 0.)?)?;
        let dir1 = if back {
            Vector3::new(0., -1., 0.)
        } else {
            Vector3::new(0., 0., 0.)
        };
        let mut f = BRepFeatMakeLinearForm::with_wire(
            &basis(0)?,
            &w,
            &pl,
            Vector3::new(0., 1., 0.),
            dir1,
            mode,
            slide,
        )?;
        if f.is_done() {
            f.perform()?;
        }
        form!(f);
        Ok(())
    });
    end();
}

/// `ft_rform basis mode slide h1 h2`.
pub fn rform_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, mode, slide, h1, h2) = (r.integer(), r.integer(), r.flag(), r.real(), r.real());
    attempt(|| {
        let b = if kind != 0 {
            ft_box(-5., -5., 0., 10., 10., 10.)?
        } else {
            BRepPrimAPIMakeCylinder::new(5., 10.).shape()?.clone()
        };
        let z1 = if mode != 0 { 12. } else { 8. };
        let w = polygon_wire(&[(2., 0., 10.), (2., 0., z1), (4., 0., z1), (4., 0., 10.)])?;
        let pl = plane(Point3::new(0., 0., 0.), Direction3::new(0., 1., 0.)?)?;
        let axis = Axis1::new(Point3::new(0., 0., 0.), Direction3::new(0., 0., 1.)?);
        let mut f = BRepFeatMakeRevolutionForm::with_wire(&b, &w, &pl, axis, h1, h2, mode, slide)?;
        if f.is_done() {
            f.perform()?;
        }
        form!(f);
        Ok(())
    });
    end();
}

/// `ft_gluer z h`.
pub fn gluer_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (z, h) = (r.real(), r.real());
    attempt(|| {
        let b = basis(0)?;
        let s = ft_box(2., 2., z, 2., 2., h)?;
        let top = face_at(&b, 10.)?;
        let mut g = BRepFeatGluer::with_shapes(&s, &b);
        g.bind_faces(&face_at(&s, 10.)?, &top)?;
        int(g.ope_type() as i32);
        g.build(&Default::default())?;
        result(&mut g)?;
        if g.is_done() {
            int(g.modified(&top)?.len() as i32);
            flag(g.is_deleted(&top)?);
        }
        Ok(())
    });
    end();
}

/// `ft_split x n`.
pub fn split_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (x, n) = (r.real(), r.integer());
    attempt(|| {
        let b = basis(0)?;
        let top = face_at(&b, 10.)?;
        let mut sp = BRepFeatSplitShape::with_shape(&b);
        for k in 0..n {
            let xk = x + k as f64;
            let e = BRepBuilderAPIMakeEdge::from_points(
                Point3::new(xk, 0., 10.),
                Point3::new(xk, 10., 10.),
            )?
            .edge()?;
            sp.add_edge(&e, &top)?;
        }
        sp.build(&Default::default())?;
        result(&mut sp)?;
        if sp.is_done() {
            int(sp.modified(&top)?.len() as i32);
            int(sp.direct_left()?.len() as i32);
            int(sp.left()?.len() as i32);
            int(sp.right()?.len() as i32);
        }
        Ok(())
    });
    end();
}
