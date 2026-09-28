//! `gp3d_dir` and `gp3d_pnt`: gp_Dir setters/products and gp_Pnt members, mirroring
//! `gpc_dir_case` and `gpc_pnt_case`.
use super::json;
use crate::elementary_probe::{Fields, attempt, dir, end, flag, pnt, real, reals, xyz};
use crate::gp_transform_probe::read_transform;
use rustycad_tkernel::ncollection::{HashKey, HashPolicy};
use rustycad_tkmath::gp::{Direction3, Point3, PointHashPolicy, Vector3, Xyz};

pub fn gp3_dir_case(values: &[f64]) {
    let mut f = Fields::new(values);
    let (v, w, u) = (f.xyz(), f.xyz(), f.xyz());
    let (idx, s) = (f.int(), f.real());
    let (mut d, mut e, mut g) = (
        Direction3::default(),
        Direction3::default(),
        Direction3::default(),
    );
    attempt(|| {
        d = Direction3::from_xyz(v)?;
        Ok(())
    });
    attempt(|| {
        e = Direction3::from_vector(Vector3::from(w))?;
        Ok(())
    });
    attempt(|| {
        g = Direction3::new(u.x(), u.y(), u.z())?;
        Ok(())
    });
    [d, e, g].into_iter().for_each(dir);
    setters(d, w, idx, s);
    products(d, e, g, s);
    json(
        d,
        |d, s| d.dump_json(s),
        |d, t, p| d.init_from_json(t, p),
        dir,
    );
    end();
}
fn setters(d: Direction3, w: Xyz, idx: i32, s: f64) {
    let mut e = d;
    attempt(|| e.set_coord(idx, s));
    dir(e);
    e = d;
    attempt(|| e.set_coordinates(w.x(), w.y(), w.z()));
    dir(e);
    let single: [fn(&mut Direction3, f64) -> _; 3] =
        [Direction3::set_x, Direction3::set_y, Direction3::set_z];
    for set in single {
        e = d;
        attempt(|| set(&mut e, s));
        dir(e);
    }
    e = d;
    attempt(|| e.set_xyz(w));
    dir(e);
    attempt(|| {
        real(d.coord(idx)?);
        Ok(())
    });
    reals(d.coordinates());
}
fn products(d: Direction3, e: Direction3, f: Direction3, s: f64) {
    dir(-d);
    reals(&[d * e, d.dot(e), d.dot_cross(e, f)]);
    flag(d.is_equal(&e, s));
    attempt(|| {
        dir(d.crossed(e)?);
        Ok(())
    });
    for _ in 0..2 {
        // `g ^= e` and `g.Cross(e)`: both keep the unnormalized product when they raise.
        let mut g = d;
        attempt(|| g.cross(e));
        dir(g);
    }
    attempt(|| {
        dir(d.cross_crossed(e, f)?);
        Ok(())
    });
    let mut g = d;
    attempt(|| g.cross_cross(e, f));
    dir(g);
    // The copy/move constructors and assignments are Rust copies.
    dir(d);
    dir(e);
}

pub fn gp3_pnt_case(values: &[f64]) {
    let mut f = Fields::new(values);
    let (p, q, v) = (f.point(), f.point(), f.xyz());
    let (alpha, beta, s, idx) = (f.real(), f.real(), f.real(), f.int());
    let transform = read_transform(&mut f);
    let mut b = p;
    b.barycenter(alpha, q, beta);
    pnt(b);
    access(p, v, s, idx);
    reals(&[p.distance(&q), p.square_distance(&q)]);
    flag(p.is_equal(&q, s));
    pnt(p.translated(Vector3::from(v)));
    pnt(p.translated_between(q, Point3::from(v)));
    b = p;
    b.translate_between(q, Point3::from(v));
    pnt(b);
    b.translate(Vector3::from(v));
    pnt(b);
    motions(p, q, s, transform);
    print!("{} {} ", p.occt_hash(), q.occt_hash());
    flag(PointHashPolicy.equal(&p, &q));
    flag(PointHashPolicy.equal(&q, &p));
    json(
        p,
        |p, s| p.dump_json(s),
        |p, t, c| p.init_from_json(t, c),
        pnt,
    );
    end();
}
fn access(p: Point3, v: Xyz, s: f64, idx: i32) {
    let mut b = p;
    attempt(|| b.set_coord(idx, s));
    pnt(b);
    attempt(|| {
        real(p.coord(idx)?);
        Ok(())
    });
    b.set_coordinates(s, v.x(), v.y());
    b.set_x(v.z());
    b.set_y(s);
    b.set_z(v.x());
    pnt(b);
    b.set_xyz(v);
    pnt(b);
    reals(p.coordinates());
    xyz(p.xyz());
    b = p;
    *b.change_coord() += v;
    *b.change_coord() *= s;
    pnt(b);
    pnt(Point3::default());
}
fn motions(p: Point3, q: Point3, s: f64, transform: rustycad_tkmath::gp::Transform3) {
    pnt(p.scaled(q, s));
    let mut b = p;
    b.scale(q, s);
    pnt(b);
    pnt(p.mirrored_point(q));
    b = p;
    b.mirror_point(q);
    pnt(b);
    pnt(p.transformed(transform));
    b = p;
    b.transform(transform);
    pnt(b);
}
