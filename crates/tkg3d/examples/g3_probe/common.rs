//! Printers, reader extensions, transformation, curve and surface builders and the
//! summaries shared by the TKG3d probe; the counterpart of
//! tools/oracle/tkg3d_probe_common.cxx.
pub use super::print::{Reader, attempt, end, flag, int, real, show};
use super::shapes;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::geom::{GeomCurve, GeomGeometry, GeomSurface};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Direction3, GeneralTransform2, Point3, Transform3, Vector3, Xyz,
};

pub fn xyz(value: Xyz) {
    real(value.x());
    real(value.y());
    real(value.z());
}
pub fn pnt(value: Point3) {
    xyz(value.xyz());
}
pub fn vec(value: Vector3) {
    xyz(*value.xyz());
}
pub fn dir(value: Direction3) {
    xyz(value.xyz());
}
pub fn ax1(value: Axis1) {
    pnt(value.location());
    dir(value.direction());
}
pub fn ax2(value: Axis2) {
    pnt(value.location());
    dir(value.direction());
    dir(value.x_direction());
    dir(value.y_direction());
}
pub fn ax3(value: Axis3) {
    pnt(value.location());
    dir(value.direction());
    dir(value.x_direction());
    dir(value.y_direction());
}
/// The class name of a handle, `null` for a null one (g3_name).
pub fn name<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) {
    match handle.dynamic_type() {
        Some(kind) => print!("{} ", kind.name()),
        None => print!("null "),
    }
}
pub fn dump(geometry: &dyn GeomGeometry, depth: i32) {
    let mut stream = DumpStream::default();
    geometry.dump_json(&mut stream, depth).unwrap();
    print!("{{{}}} ", stream.text());
}
pub fn gtrsf2d(value: GeneralTransform2) {
    for i in 1..=2 {
        for j in 1..=3 {
            real(value.value(i, j).unwrap());
        }
    }
    int(value.form() as i32);
}

impl Reader<'_> {
    pub fn pnt(&mut self) -> Point3 {
        let (x, y) = (self.real(), self.real());
        Point3::new(x, y, self.real())
    }
    pub fn vec(&mut self) -> Vector3 {
        let (x, y) = (self.real(), self.real());
        Vector3::new(x, y, self.real())
    }
    pub fn dir(&mut self) -> Direction3 {
        let (x, y) = (self.real(), self.real());
        Direction3::new(x, y, self.real()).expect("probe direction")
    }
    pub fn ax1(&mut self) -> Axis1 {
        let p = self.pnt();
        Axis1::new(p, self.dir())
    }
    pub fn ax2(&mut self) -> Axis2 {
        let (p, n) = (self.pnt(), self.dir());
        Axis2::with_x_direction(p, n, self.dir()).expect("probe frame")
    }
    pub fn ax3(&mut self) -> Axis3 {
        let (p, n) = (self.pnt(), self.dir());
        Axis3::new(p, n, self.dir()).expect("probe frame")
    }
}

/// One transformation, as g3_trsf reads it.
pub fn trsf(r: &mut Reader) -> Transform3 {
    let kind = r.integer();
    let p: Vec<f64> = (0..9).map(|_| r.real()).collect();
    let mut t = Transform3::default();
    let a = Point3::new(p[0], p[1], p[2]);
    let d = || Direction3::new(p[3], p[4], p[5]).unwrap();
    match kind {
        0 => t.set_mirror_point(a),
        1 => t.set_mirror_axis(Axis1::new(a, d())),
        2 => {
            let x = Direction3::new(p[6], p[7], p[8]).unwrap();
            t.set_mirror_plane(Axis2::with_x_direction(a, d(), x).unwrap());
        }
        3 => t.set_rotation(Axis1::new(a, d()), p[6]).unwrap(),
        4 => t.set_scale(a, p[3]).unwrap(),
        5 => t.set_translation(Vector3::new(p[0], p[1], p[2])),
        6 => t.set_translation_between(a, Point3::new(p[3], p[4], p[5])),
        7 => {
            let mut s = Transform3::default();
            s.set_scale(a, p[7]).unwrap();
            t.set_rotation(Axis1::new(a, d()), p[6]).unwrap();
            t = t.multiplied(s);
        }
        _ => {}
    }
    t
}

/// Reads a curve spec, reporting a construction failure and ending the line (g3_built_curve).
pub fn built_curve(r: &mut Reader) -> Option<Handle<dyn GeomCurve>> {
    report(shapes::curve(r))
}

/// Reads a surface spec, reporting a construction failure and ending the line (g3_built_surface).
pub fn built_surface(r: &mut Reader) -> Option<Handle<dyn GeomSurface>> {
    report(shapes::surface(r))
}

fn report<T>(built: Result<T, Failure>) -> Option<T> {
    match built {
        Ok(value) => Some(value),
        Err(error) => {
            println!("{}", error.exception_type());
            None
        }
    }
}

/// The curve summary of g3_curve_summary.
pub fn curve_summary(c: &dyn GeomCurve, params: &[f64]) {
    real(c.first_parameter());
    real(c.last_parameter());
    show(c.is_closed(), flag);
    flag(c.is_periodic());
    show(c.period(), real);
    int(c.continuity() as i32);
    for n in -1..=4 {
        show(c.is_cn(n), flag);
    }
    for &u in params {
        curve_evaluations(c, u);
    }
}

fn curve_evaluations(c: &dyn GeomCurve, u: f64) {
    show(c.eval_d0(u), pnt);
    show(c.eval_d1(u), |r| {
        pnt(r.point);
        vec(r.d1);
    });
    show(c.eval_d2(u), |r| {
        pnt(r.point);
        [r.d1, r.d2].into_iter().for_each(vec);
    });
    show(c.eval_d3(u), |r| {
        pnt(r.point);
        [r.d1, r.d2, r.d3].into_iter().for_each(vec);
    });
    for n in 0..=5 {
        show(c.eval_dn(u, n), vec);
    }
    show(c.value(u), pnt);
    real(c.reversed_parameter(u));
}

/// The surface summary of g3_surface_summary.
pub fn surface_summary(s: &dyn GeomSurface, uv: &[f64]) {
    let (u1, u2, v1, v2) = s.bounds();
    [u1, u2, v1, v2].into_iter().for_each(real);
    show(s.is_u_closed(), flag);
    show(s.is_v_closed(), flag);
    flag(s.is_u_periodic());
    flag(s.is_v_periodic());
    show(s.u_period(), real);
    show(s.v_period(), real);
    int(s.continuity() as i32);
    for n in -1..=4 {
        show(s.is_cn_u(n), flag);
        show(s.is_cn_v(n), flag);
    }
    for pair in uv.as_chunks::<2>().0 {
        surface_evaluations(s, pair[0], pair[1]);
    }
}

fn surface_evaluations(s: &dyn GeomSurface, u: f64, v: f64) {
    show(s.eval_d0(u, v), pnt);
    show(s.eval_d1(u, v), |r| {
        pnt(r.point);
        [r.d1u, r.d1v].into_iter().for_each(vec);
    });
    show(s.eval_d2(u, v), |r| {
        pnt(r.point);
        [r.d1u, r.d1v, r.d2u, r.d2v, r.d2uv]
            .into_iter()
            .for_each(vec);
    });
    show(s.eval_d3(u, v), |r| {
        pnt(r.point);
        let d = [
            r.d1u, r.d1v, r.d2u, r.d2v, r.d2uv, r.d3u, r.d3v, r.d3uuv, r.d3uvv,
        ];
        d.into_iter().for_each(vec);
    });
    for nu in 0..=3 {
        for nv in 0..=3 {
            show(s.eval_dn(u, v, nu, nv), vec);
        }
    }
    show(s.eval_dn(u, v, -1, 2), vec);
    show(s.value(u, v), pnt);
    real(s.u_reversed_parameter(u));
    real(s.v_reversed_parameter(v));
}

/// The iso curves of g3_isos.
pub fn isos(s: &dyn GeomSurface, isos: &[f64], params: &[f64]) {
    for &p in isos {
        show(s.u_iso(p), |c| {
            name(&c);
            curve_summary(&*c, params);
        });
        show(s.v_iso(p), |c| {
            name(&c);
            curve_summary(&*c, params);
        });
    }
}
