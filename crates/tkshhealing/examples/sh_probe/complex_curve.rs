//! ShapeExtend_ComplexCurve probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_complexcurve.cxx (which documents the request and the
//! test subclass both probes define).
use super::common::{Reader, attempt, end, flag, int, pnt, real, show, xyz};
use super::shapes::tp_shapes;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, StandardTransient};
use rustycad_tkernel::{impl_transient, standard::SharedState};
use rustycad_tkg3d::geom::{GeomCurve, GeomGeometry, dump_curve};
use rustycad_tkmath::gp::{Axis1, Transform3, Vector3};
use rustycad_tkshhealing::shape_extend::ShapeExtendComplexCurve;
use std::io;

/// `ShProbeComplexCurve`: segment i of n covers `[(i - 1) / n, i / n]`.
pub struct ShProbeComplexCurve {
    parent: StandardTransient,
    curves: Vec<Handle<dyn GeomCurve>>,
    closed: SharedState<bool>,
}
impl_transient!(
    ShProbeComplexCurve,
    "ShProbeComplexCurve",
    dyn ShapeExtendComplexCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn ShapeExtendComplexCurve]
);

impl ShProbeComplexCurve {
    fn new(curves: Vec<Handle<dyn GeomCurve>>) -> Self {
        Self {
            parent: StandardTransient::default(),
            curves,
            closed: SharedState::new(false),
        }
    }
    fn range(&self, i: i32) -> Result<(f64, f64), Failure> {
        let c = self.curve(i)?;
        Ok((c.first_parameter(), c.last_parameter()))
    }
}

impl GeomGeometry for ShProbeComplexCurve {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        self.complex_transform(transform)
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(Self::new(self.curves.clone())).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_curve(stream, depth)
    }
}

impl GeomCurve for ShProbeComplexCurve {
    fn reverse(&self) -> Result<(), Failure> {
        Ok(())
    }
    rustycad_tkshhealing::shape_extend_complex_curve_members!();
}

impl ShapeExtendComplexCurve for ShProbeComplexCurve {
    fn nb_curves(&self) -> i32 {
        self.curves.len() as i32
    }
    fn curve(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.curves.get(i).cloned())
            .ok_or_else(|| Failure::with_message("std::out_of_range", "probe segment index"))
    }
    fn locate_parameter(&self, u: f64) -> Result<(i32, f64), Failure> {
        let n = self.nb_curves();
        let s = u * f64::from(n);
        let mut i = s.floor() as i32 + 1;
        if i < 1 {
            i = 1;
        }
        if i > n {
            i = n;
        }
        let (f, l) = self.range(i)?;
        Ok((i, f + (s - f64::from(i - 1)) * (l - f)))
    }
    fn local_to_global(&self, index: i32, u_local: f64) -> Result<f64, Failure> {
        let (f, l) = self.range(index)?;
        Ok((f64::from(index - 1) + (u_local - f) / (l - f)) / f64::from(self.nb_curves()))
    }
    fn get_scale_factor(&self, ind: i32) -> Result<f64, Failure> {
        let (f, l) = self.range(ind)?;
        Ok(f64::from(self.nb_curves()) * (l - f))
    }
    fn complex_closed(&self) -> bool {
        *self.closed.read()
    }
    fn set_complex_closed(&self, closed: bool) {
        *self.closed.write() = closed;
    }
}

fn vec(v: Vector3) {
    xyz(*v.xyz());
}

fn eval(c: &ShProbeComplexCurve, u: f64) {
    show(c.value(u), pnt);
    attempt(|| {
        let d = c.eval_d1(u)?;
        pnt(d.point);
        vec(d.d1);
        Ok(())
    });
    attempt(|| {
        let d = c.eval_d2(u)?;
        pnt(d.point);
        vec(d.d1);
        vec(d.d2);
        Ok(())
    });
    attempt(|| {
        let d = c.eval_d3(u)?;
        pnt(d.point);
        [d.d1, d.d2, d.d3].into_iter().for_each(vec);
        Ok(())
    });
    for n in 0..=4 {
        show(c.eval_dn(u, n), vec);
    }
    attempt(|| {
        let (i, local) = c.locate_parameter(u)?;
        int(i);
        real(local);
        Ok(())
    });
}

pub fn trsf(r: &mut Reader<'_>) -> Transform3 {
    let mut t = Transform3::default();
    match r.integer() {
        1 => {
            let (x, y) = (r.real(), r.real());
            t.set_translation(Vector3::new(x, y, r.real()));
        }
        2 => {
            let (p, d) = (r.pnt(), r.dir());
            t.set_rotation(Axis1::new(p, d), r.real())
                .expect("probe rotation");
        }
        _ => {}
    }
    t
}

fn constants(c: &ShProbeComplexCurve, preci: f64) -> Result<(), Failure> {
    int(c.nb_curves());
    real(c.first_parameter());
    real(c.last_parameter());
    flag(c.is_periodic());
    int(c.continuity() as i32);
    for n in -1..=2 {
        flag(c.is_cn(n)?);
    }
    real(c.reversed_parameter(0.3)?);
    flag(c.is_closed()?);
    show(c.check_connectivity(preci), flag);
    flag(c.is_closed()?);
    for i in 1..=c.nb_curves() {
        real(c.get_scale_factor(i)?);
        real(c.local_to_global(i, c.curve(i)?.first_parameter())?);
    }
    Ok(())
}

/// `sh_complex program n (curve)... preci nu (u)... trsf ...`.
pub fn complex_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shape program");
    let curves = (0..r.integer())
        .map(|_| t.curves[r.integer() as usize].clone())
        .collect();
    let preci = r.real();
    let us = r.reals();
    let transform = trsf(&mut r);
    let c = ShProbeComplexCurve::new(curves);
    attempt(|| constants(&c, preci));
    us.iter().for_each(|&u| eval(&c, u));
    show(c.period(), real);
    attempt(|| c.transform(transform));
    for &u in &us {
        show(c.value(u), pnt);
    }
    let handle: Handle<dyn GeomCurve> =
        Handle::new(ShProbeComplexCurve::new(c.curves.clone())).cast();
    flag(handle.is_kind(<dyn ShapeExtendComplexCurve as RuntimeTyped>::static_type()));
    flag(handle.is_kind(<dyn GeomCurve as RuntimeTyped>::static_type()));
    end();
}
