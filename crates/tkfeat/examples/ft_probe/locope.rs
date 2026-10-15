//! The LocOpe probes (`ft_lo_*`); the twin of tools/oracle/tkfeat_probe_locope.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::shapes::{basis, count, face_at, ft_box, polygon_face, rect, shape};
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfeat::loc_ope::{
    CurveShapeIntersector, DPrism, FindEdges, Gluer, LinearForm, Prism, Revol, RevolutionForm,
    Spliter, WiresOnShape,
};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

/// `ft_lo_shapes`: the swept shape and the face counts of its first and last shapes.
fn shapes(s: &Shape, first: &Shape, last: &Shape) -> Result<(), Failure> {
    shape(s)?;
    int(count(first, ShapeEnum::Face)?);
    int(count(last, ShapeEnum::Face)?);
    Ok(())
}

/// `ft_lo_curves`: the number of curves, the range of the first one and of the
/// barycentre curve.
fn curves(scur: &Sequence<Handle<dyn GeomCurve>>, b: &Handle<dyn GeomCurve>) {
    int(scur.len() as i32);
    if let Some(c) = scur.iter().next().and_then(|c| c.get()) {
        real(c.first_parameter());
        real(c.last_parameter());
    }
    if let Some(c) = b.get() {
        real(c.first_parameter());
        real(c.last_parameter());
    }
}

/// The square [0, 10]^2 and the rectangle of the revolutions.
fn bases() -> Result<(Face, Face), Failure> {
    let rc = polygon_face(&[(2., 0., 0.), (4., 0., 0.), (4., 0., 2.), (2., 0., 2.)])?;
    Ok((rect(0., 0., 10., 10., 0.)?, rc))
}

/// The prisms of `ft_lo_sweep` (kinds 0 to 3).
fn prisms(kind: i32, a: f64, b: f64, c: f64, sq: &Face) -> Result<(), Failure> {
    let mut scur = Sequence::default();
    if kind <= 1 {
        let p = if kind == 0 {
            Prism::with_vector(sq, Vector3::new(a, b, c))?
        } else {
            Prism::with_translation(sq, Vector3::new(0., 0., a), Vector3::new(0., 0., b))?
        };
        shapes(p.shape()?, p.first_shape(), p.last_shape())?;
        p.curves(&mut scur)?;
        curves(&scur, &p.baryc_curve()?);
        return Ok(());
    }
    let p = if kind == 2 {
        DPrism::with_height(sq, a, b)?
    } else {
        DPrism::with_heights(sq, a, b, c)?
    };
    shapes(p.shape()?, p.first_shape(), p.last_shape())?;
    p.curves(&mut scur)?;
    curves(&scur, &p.baryc_curve()?);
    Ok(())
}

/// The revolutions and forms of `ft_lo_sweep` (kinds 4 to 7).
fn others(kind: i32, a: f64, b: f64, sq: &Face, rc: &Face) -> Result<(), Failure> {
    let z = Axis1::new(Point3::new(0., 0., 0.), Direction3::new(0., 0., 1.)?);
    if kind == 4 || kind == 5 {
        let mut p = Revol::new();
        if kind == 4 {
            p.perform(rc, z, a)?;
        } else {
            p.perform_turned(rc, z, a, b)?;
        }
        shapes(p.shape()?, p.first_shape(), p.last_shape())?;
        let mut scur = Sequence::default();
        p.curves(&mut scur)?;
        curves(&scur, &p.baryc_curve()?);
        return Ok(());
    }
    if kind == 6 {
        let p = LinearForm::with_vector(
            sq,
            Vector3::new(0., 0., a),
            Point3::new(0., 0., 0.),
            Point3::new(10., 0., 0.),
        )?;
        return shapes(p.shape()?, p.first_shape(), p.last_shape());
    }
    let mut p = RevolutionForm::new();
    p.perform(rc, z, a)?;
    shapes(p.shape()?, p.first_shape(), p.last_shape())
}

/// `ft_lo_sweep kind a b c`.
pub fn sweep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let (a, b, c) = (r.real(), r.real(), r.real());
    attempt(|| {
        let (sq, rc) = bases()?;
        if kind <= 3 {
            return prisms(kind, a, b, c, &sq);
        }
        others(kind, a, b, &sq, &rc)
    });
    end();
}

/// `ft_lo_csinter x y z dx dy dz`.
pub fn csinter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (p, d) = (r.pnt(), r.dir());
    attempt(|| {
        let asi = CurveShapeIntersector::with_axis(&Axis1::new(p, d), &basis(0)?)?;
        flag(asi.is_done());
        let n = asi.nb_points()?;
        int(n);
        for i in 1..=n {
            real(asi.point(i)?.parameter());
            int(asi.point(i)?.orientation() as i32);
        }
        let (mut o, mut i1, mut i2) = (Orientation::Forward, 0, 0);
        let ok = asi.localize_after(0., &mut o, &mut i1, &mut i2)?;
        flag(ok);
        if ok {
            int(o as i32);
            int(i1);
            int(i2);
        }
        let ok = asi.localize_before_index(n + 1, &mut o, &mut i1, &mut i2)?;
        flag(ok);
        if ok {
            int(o as i32);
            int(i1);
            int(i2);
        }
        Ok(())
    });
    end();
}

/// `ft_lo_find_edges ox dz`.
pub fn find_edges_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (ox, dz) = (r.real(), r.real());
    attempt(|| {
        let up = ft_box(ox, 0., 10., 10., 10., dz)?;
        let (f1, f2) = (face_at(&basis(0)?, 10.)?, face_at(&up, 10.)?);
        let mut fe = FindEdges::with_shapes(&f1, &f2)?;
        let mut n = 0;
        fe.init_iterator();
        while fe.more() {
            n += 1;
            fe.next();
        }
        int(n);
        Ok(())
    });
    end();
}

/// `ft_lo_spliter x`.
pub fn spliter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let x = r.real();
    attempt(|| {
        let b = basis(0)?;
        let top = face_at(&b, 10.)?;
        let mut pw = WiresOnShape::new(&b);
        let e =
            BRepBuilderAPIMakeEdge::from_points(Point3::new(x, 0., 10.), Point3::new(x, 10., 10.))?
                .edge()?;
        pw.bind_edge(&e, &top)?;
        let mut sp = Spliter::with_shape(&b);
        sp.perform(&mut pw)?;
        flag(sp.is_done());
        if sp.is_done() {
            shape(sp.resulting_shape()?)?;
            int(sp.descendant_shapes(&top)?.len() as i32);
            int(sp.direct_left()?.len() as i32);
            int(sp.left()?.len() as i32);
        }
        Ok(())
    });
    end();
}

/// `ft_lo_gluer z h`.
pub fn gluer_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (z, h) = (r.real(), r.real());
    attempt(|| {
        let b = basis(0)?;
        let s = ft_box(2., 2., z, 2., 2., h)?;
        let mut g = Gluer::with_shapes(&b, &s);
        g.bind_faces(&face_at(&s, 10.)?, &face_at(&b, 10.)?)?;
        int(g.ope_type() as i32);
        g.perform()?;
        flag(g.is_done());
        if g.is_done() {
            shape(g.resulting_shape()?)?;
            int(g.edges().len() as i32);
        }
        Ok(())
    });
    end();
}
