//! BinTools probe cases, part 2b: the Rust twin of the set round trips of
//! tools/oracle/tkbrep_probe_bintools_geometry.cxx.
use super::bintools::{attempt, bytes_token, out_stream};
use super::bintools_geometry::geometry;
use super::common::{Reader, end, flag, int, real, word, written_bytes};
use rustycad_tkbrep::bintools::{BinCurve2dSet, BinCurveSet, BinSurfaceSet};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};

/// `bt_eval` of a curve, a 2D curve or a surface read back.
pub trait Evaluated {
    fn evaluate(&self) -> Result<(), Failure>;
}
impl Evaluated for Handle<dyn GeomCurve> {
    fn evaluate(&self) -> Result<(), Failure> {
        word(self.dynamic_type().map_or("null", |t| t.name()));
        let p = self.value(0.3)?;
        [p.x(), p.y(), p.z()].into_iter().for_each(real);
        Ok(())
    }
}
impl Evaluated for Handle<dyn Geom2dCurve> {
    fn evaluate(&self) -> Result<(), Failure> {
        word(self.dynamic_type().map_or("null", |t| t.name()));
        let p = self.value(0.3)?;
        [p.x(), p.y()].into_iter().for_each(real);
        Ok(())
    }
}
impl Evaluated for Handle<dyn GeomSurface> {
    fn evaluate(&self) -> Result<(), Failure> {
        word(self.dynamic_type().map_or("null", |t| t.name()));
        let p = self.value(0.3, 0.2)?;
        [p.x(), p.y(), p.z()].into_iter().for_each(real);
        Ok(())
    }
}

/// The geometry sets the probe drives.
pub trait ProbeSet: Default {
    type Member: Evaluated;
    fn write(&self, stream: &mut IoStream) -> Result<(), Failure>;
    fn read(&mut self, stream: &mut IoStream) -> Result<(), Failure>;
    fn member(&self, index: i32) -> Result<Self::Member, Failure>;
}
macro_rules! probe_set {
    ($set:ty, $member:ty, $get:ident) => {
        impl ProbeSet for $set {
            type Member = $member;
            fn write(&self, stream: &mut IoStream) -> Result<(), Failure> {
                <$set>::write(self, stream, &ProgressRange::default())
            }
            fn read(&mut self, stream: &mut IoStream) -> Result<(), Failure> {
                <$set>::read(self, stream, &ProgressRange::default())
            }
            fn member(&self, index: i32) -> Result<$member, Failure> {
                self.$get(index)
            }
        }
    };
}
probe_set!(BinCurveSet, Handle<dyn GeomCurve>, curve);
probe_set!(BinCurve2dSet, Handle<dyn Geom2dCurve>, curve2d);
probe_set!(BinSurfaceSet, Handle<dyn GeomSurface>, surface);

/// `bt_set_bytes`.
pub fn set_bytes<S: ProbeSet>(set: &S) -> Vec<u8> {
    let mut o = out_stream();
    attempt(|| set.write(&mut o));
    written_bytes(o)
}

/// `bt_eval` on a member, printing the class and D0 or the failure type.
fn eval_member<S: ProbeSet>(set: &S, index: i32) -> Result<(), Failure> {
    set.member(index)?.evaluate()
}

fn round_trip<S: ProbeSet>(set: &S, n: i32) {
    let bytes = set_bytes(set);
    bytes_token(&bytes);
    let mut back = S::default();
    let mut is = IoStream::from_bytes(bytes);
    attempt(|| back.read(&mut is));
    int(i32::from(is.rdstate()));
    for i in 1..=n {
        attempt(|| eval_member(&back, i));
    }
    bytes_token(&set_bytes(&back));
}

/// `bt_read_geometry_set`: a given byte string read into `S`, its members up to the
/// first out-of-range index, and the set written again.
pub fn read_geometry_set<S: ProbeSet>(bytes: Vec<u8>) {
    let mut set = S::default();
    let mut is = IoStream::from_bytes(bytes);
    attempt(|| set.read(&mut is));
    int(i32::from(is.rdstate()));
    for i in 1..=64 {
        let member = set.member(i);
        match member {
            Ok(m) => {
                if let Err(e) = m.evaluate() {
                    word(e.exception_type());
                    break;
                }
            }
            Err(e) => {
                word(e.exception_type());
                break;
            }
        }
    }
    bytes_token(&set_bytes(&set));
}

/// `br_bin_geometry`.
pub fn geometry_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let g = geometry(&mut r);
    let mut curves = BinCurveSet::new();
    let mut nc = 0;
    for c in g.curves.iter().filter(|c| !c.is_null()) {
        int(curves.add(c));
        nc += 1;
    }
    round_trip(&curves, nc);
    let mut pcurves = BinCurve2dSet::new();
    let mut np = 0;
    for c in g.pcurves.iter().filter(|c| !c.is_null()) {
        int(pcurves.add(c));
        np += 1;
    }
    round_trip(&pcurves, np);
    let mut surfaces = BinSurfaceSet::new();
    let mut ns = 0;
    for s in g.surfaces.iter().filter(|s| !s.is_null()) {
        int(surfaces.add(s));
        ns += 1;
    }
    round_trip(&surfaces, ns);
    for c in g.curves.iter().filter(|c| !c.is_null()) {
        int(curves.index(c));
    }
    int(curves.index(&Handle::default()));
    attempt(|| curves.curve(0).map(|c| flag(c.is_null())));
    attempt(|| curves.curve(nc + 1).map(|_| ()));
    attempt(|| pcurves.curve2d(0).map(|_| ()));
    attempt(|| surfaces.surface(ns + 1).map(|_| ()));
    end();
}
