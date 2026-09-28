//! The set, progress, debug-dump and custom-handler cases, the Rust twin of
//! tools/oracle/tkgeombase_probe_geomtools_sets.cxx.
use super::super::common::{Reader, end, flag, int};
use super::console::{captured, install_default_handler, probe_console};
use super::kinds::{Curves, Curves2d, GtKind, Surfaces, summary};
use super::{bytes, state, text, texts, written, written_bytes};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{
    ProgressIndicator, ProgressIndicatorHooks, ProgressRange, ProgressScope,
};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_eval::Geom2dEvalSineWaveCurve;
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_eval::{GeomEvalCircularHelixCurve, GeomEvalEllipsoidSurface};
use rustycad_tkgeombase::geom_tools::{
    self, ConsoleOutput, Curve2dSet, CurveSet, SurfaceSet, UndefinedTypeHandler,
    UndefinedTypeHandlerBase,
};
use rustycad_tkmath::gp::{Axis2, Axis2d, Axis3};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

fn set_case<K: GtKind>(r: &mut Reader<'_>) {
    let mut set = K::new_set();
    let (n, mut count) = (r.integer(), 0);
    let mut items: Vec<Handle<K::Geometry>> = Vec::new();
    for _ in 0..n {
        let h = if r.flag() {
            items.last().unwrap().clone()
        } else {
            K::build(r).unwrap()
        };
        items.push(h.clone());
        let index = K::add(&mut set, &h);
        count = count.max(index);
        int(index);
    }
    for h in &items {
        int(K::index(&set, h));
    }
    flag(K::get(&set, 0).is_null());
    flag(K::get(&set, count + 1).is_null());
    written(6, |s| K::dump_set(&set, s));
    let mut stream = IoStream::from_bytes(Vec::new());
    stream.set_precision(3);
    text(&captured(|| {
        K::write_set(&set, &mut stream, &ProgressRange::default()).unwrap()
    }));
    let precision = stream.precision() as i32;
    let full = written_bytes(stream);
    text(&full);
    int(precision);
    let mut back = K::new_set();
    for _ in 0..2 {
        let mut input = IoStream::from_bytes(full.clone());
        text(&captured(|| {
            K::read_set(&mut back, &mut input, &ProgressRange::default())
        }));
        int(i32::from(input.rdstate()));
    }
    for i in 1..=2 * count + 1 {
        summary::<K>(&K::get(&back, i));
    }
    K::clear(&mut back);
    flag(K::get(&back, 1).is_null());
}

/// `gt_set kind (dup | 0 spec)...`.
pub fn set_case_dispatch(values: &[f64]) {
    install_default_handler();
    let mut r = Reader::new(values);
    match r.integer() {
        0 => set_case::<Curves>(&mut r),
        1 => set_case::<Curves2d>(&mut r),
        _ => set_case::<Surfaces>(&mut r),
    }
    end();
}

/// `gt_set_null`.
pub fn set_null_case(_: &[f64]) {
    let (c, c2, s): (
        Handle<dyn GeomCurve>,
        Handle<dyn Geom2dCurve>,
        Handle<dyn GeomSurface>,
    ) = (Handle::default(), Handle::default(), Handle::default());
    let mut curves = CurveSet::new();
    int(curves.index(&c));
    int(curves.add(&c));
    int(curves.index(&c));
    flag(curves.curve(1).is_null());
    let mut curves2d = Curve2dSet::new();
    int(curves2d.index(&c2));
    int(curves2d.add(&c2));
    int(curves2d.add(&c2));
    int(curves2d.index(&c2));
    let mut surfaces = SurfaceSet::new();
    int(surfaces.index(&s));
    int(surfaces.add(&s));
    int(surfaces.add(&s));
    int(surfaces.index(&s));
    end();
}

fn set_read<K: GtKind>(r: &mut Reader<'_>) {
    let mut input = IoStream::from_bytes(bytes(r));
    let mut set = K::new_set();
    text(&captured(|| {
        K::read_set(&mut set, &mut input, &ProgressRange::default())
    }));
    for i in 1..=6 {
        summary::<K>(&K::get(&set, i));
    }
    state(&mut input);
}

/// `gt_set_read kind bytes`.
pub fn set_read_case(values: &[f64]) {
    install_default_handler();
    let mut r = Reader::new(values);
    match r.integer() {
        0 => set_read::<Curves>(&mut r),
        1 => set_read::<Curves2d>(&mut r),
        _ => set_read::<Surfaces>(&mut r),
    }
    end();
}

/// GtBreaker: breaks once UserBreak has been asked more than `limit` times.
struct Breaker {
    calls: Arc<AtomicI32>,
    limit: i32,
}
impl ProgressIndicatorHooks for Breaker {
    fn show(&self, _: &ProgressScope<'_>, _: f64, _: bool) {}
    fn user_break(&self) -> bool {
        self.calls.fetch_add(1, Ordering::SeqCst) + 1 > self.limit
    }
}

fn breaker(limit: i32) -> (ProgressIndicator, Arc<AtomicI32>) {
    let calls = Arc::new(AtomicI32::new(0));
    let hooks = Breaker {
        calls: Arc::clone(&calls),
        limit,
    };
    (ProgressIndicator::new(Box::new(hooks)), calls)
}

fn progress<K: GtKind>(r: &mut Reader<'_>) {
    let (limit, n) = (r.integer(), r.integer());
    let mut set = K::new_set();
    for _ in 0..n {
        K::add(&mut set, &K::build(r).unwrap());
    }
    let (writer, calls) = breaker(limit);
    let mut partial = IoStream::from_bytes(Vec::new());
    K::write_set(&set, &mut partial, &writer.start()).unwrap();
    text(&written_bytes(partial));
    int(calls.load(Ordering::SeqCst));
    let mut full = IoStream::from_bytes(Vec::new());
    K::write_set(&set, &mut full, &ProgressRange::default()).unwrap();
    let (reader, calls) = breaker(limit);
    let mut back = K::new_set();
    let mut input = IoStream::from_bytes(written_bytes(full));
    K::read_set(&mut back, &mut input, &reader.start());
    int(calls.load(Ordering::SeqCst));
    for i in 1..=n + 1 {
        summary::<K>(&K::get(&back, i));
    }
    state(&mut input);
}

/// `gt_progress kind limit n spec...`.
pub fn progress_case(values: &[f64]) {
    install_default_handler();
    let mut r = Reader::new(values);
    match r.integer() {
        0 => progress::<Curves>(&mut r),
        1 => progress::<Curves2d>(&mut r),
        _ => progress::<Surfaces>(&mut r),
    }
    end();
}

/// `gt_debug_dump kind spec`.
pub fn debug_dump_case(values: &[f64]) {
    install_default_handler();
    let mut r = Reader::new(values);
    let object: Handle<StandardTransient> = match r.integer() {
        0 => Curves::build(&mut r).unwrap().cast(),
        1 => Curves2d::build(&mut r).unwrap().cast(),
        2 => Surfaces::build(&mut r).unwrap().cast(),
        3 => {
            let (x, y) = (r.real(), r.real());
            Handle::new(GeomCartesianPoint::new(x, y, r.real())).into_cast()
        }
        _ => Handle::default(),
    };
    let mut message = String::new();
    let console = captured(|| message = geom_tools::geom_tools_dump(&object, &*probe_console()));
    text(message.as_bytes());
    text(&console);
    end();
}

/// GtHandler: stores the helix (42), the sine wave (43) and the ellipsoid (44).
struct ProbeHandler {
    parent: StandardTransient,
    base: UndefinedTypeHandlerBase,
}
impl_transient!(
    ProbeHandler,
    "ProbeHandler",
    StandardTransient,
    parent,
    interfaces: [dyn UndefinedTypeHandler]
);

impl UndefinedTypeHandler for ProbeHandler {
    fn console(&self) -> Arc<dyn ConsoleOutput> {
        self.base.console()
    }
    fn print_curve(
        &self,
        c: &Handle<dyn GeomCurve>,
        os: &mut IoStream,
        compact: bool,
    ) -> Result<(), Failure> {
        let h: Handle<GeomEvalCircularHelixCurve> = c.cast();
        if h.is_null() {
            return self.base.print_curve(c, os, compact);
        }
        os.insert_bytes(if compact { b"42 " } else { b"Helix " });
        os.insert_real(h.radius())
            .insert_bytes(b" ")
            .insert_real(h.pitch())
            .insert_bytes(b"\n");
        Ok(())
    }
    fn read_curve(&self, ctype: i32, is: &mut IoStream) -> Result<Handle<dyn GeomCurve>, Failure> {
        if ctype != 42 {
            return self.base.read_curve(ctype, is);
        }
        let (radius, pitch) = (geom_tools::get_real(is), geom_tools::get_real(is));
        Ok(Handle::new(GeomEvalCircularHelixCurve::new(
            Axis2::default(),
            radius,
            pitch,
        )?)
        .into_cast())
    }
    fn print_curve2d(
        &self,
        c: &Handle<dyn Geom2dCurve>,
        os: &mut IoStream,
        compact: bool,
    ) -> Result<(), Failure> {
        let w: Handle<Geom2dEvalSineWaveCurve> = c.cast();
        if w.is_null() {
            return self.base.print_curve2d(c, os, compact);
        }
        os.insert_bytes(if compact { b"43 " } else { b"Wave " });
        os.insert_real(w.amplitude())
            .insert_bytes(b" ")
            .insert_real(w.omega());
        os.insert_bytes(b" ")
            .insert_real(w.phase())
            .insert_bytes(b"\n");
        Ok(())
    }
    fn read_curve2d(
        &self,
        ctype: i32,
        is: &mut IoStream,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        if ctype != 43 {
            return self.base.read_curve2d(ctype, is);
        }
        let (a, o, p) = (
            geom_tools::get_real(is),
            geom_tools::get_real(is),
            geom_tools::get_real(is),
        );
        Ok(Handle::new(Geom2dEvalSineWaveCurve::new(Axis2d::default(), a, o, p)?).into_cast())
    }
    fn print_surface(
        &self,
        s: &Handle<dyn GeomSurface>,
        os: &mut IoStream,
        compact: bool,
    ) -> Result<(), Failure> {
        let e: Handle<GeomEvalEllipsoidSurface> = s.cast();
        if e.is_null() {
            return self.base.print_surface(s, os, compact);
        }
        os.insert_bytes(if compact { b"44 " } else { b"Ellipsoid " });
        os.insert_real(e.semi_axis_a())
            .insert_bytes(b" ")
            .insert_real(e.semi_axis_b());
        os.insert_bytes(b" ")
            .insert_real(e.semi_axis_c())
            .insert_bytes(b"\n");
        Ok(())
    }
    fn read_surface(
        &self,
        ctype: i32,
        is: &mut IoStream,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        if ctype != 44 {
            return self.base.read_surface(ctype, is);
        }
        let (a, b, c) = (
            geom_tools::get_real(is),
            geom_tools::get_real(is),
            geom_tools::get_real(is),
        );
        Ok(Handle::new(GeomEvalEllipsoidSurface::new(Axis3::default(), a, b, c)?).into_cast())
    }
}

fn handler_texts<K: GtKind>(r: &mut Reader<'_>) {
    let g = K::build(r).unwrap();
    texts::<K>(&g);
    let mut input = IoStream::from_bytes(bytes(r));
    summary::<K>(&K::read(&mut input));
    state(&mut input);
}

/// `gt_handler kind spec bytes`.
pub fn handler_case(values: &[f64]) {
    install_default_handler();
    let original = geom_tools::get_undefined_type_handler();
    let mine: Handle<dyn UndefinedTypeHandler> = Handle::new(ProbeHandler {
        parent: StandardTransient::default(),
        base: UndefinedTypeHandlerBase::with_console(probe_console()),
    })
    .into_cast();
    geom_tools::set_undefined_type_handler(&mine);
    geom_tools::set_undefined_type_handler(&Handle::default());
    flag(geom_tools::get_undefined_type_handler() == mine);
    let mut r = Reader::new(values);
    match r.integer() {
        0 => handler_texts::<Curves>(&mut r),
        1 => handler_texts::<Curves2d>(&mut r),
        _ => handler_texts::<Surfaces>(&mut r),
    }
    geom_tools::set_undefined_type_handler(&original);
    flag(geom_tools::get_undefined_type_handler() == original);
    end();
}
