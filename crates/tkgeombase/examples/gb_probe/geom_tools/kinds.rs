//! The three geometry families of GeomTools behind one interface, so the probe cases are
//! written once, as the C++ templates over GtCurves, GtCurves2d and GtSurfaces are.
use super::super::common::{Reader, attempt, pnt, pnt2d};
use super::build;
use super::console::attempt_captured;
use super::console::captured;
use super::text;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkgeombase::geom_tools::{self, Curve2dSet, CurveSet, SurfaceSet};

/// One geometry family: its handle, its set class and the GeomTools members on them.
pub trait GtKind {
    type Geometry: ?Sized + rustycad_tkernel::standard::RuntimeTyped;
    type Set;
    fn build(r: &mut Reader<'_>) -> Result<Handle<Self::Geometry>, Failure>;
    fn dump(g: &Handle<Self::Geometry>, s: &mut IoStream) -> Result<(), Failure>;
    fn write(g: &Handle<Self::Geometry>, s: &mut IoStream) -> Result<(), Failure>;
    fn read(s: &mut IoStream) -> Handle<Self::Geometry>;
    /// The bits of the point at (0.3) or (0.3, 0.2).
    fn value(g: &Handle<Self::Geometry>) -> Result<(), Failure>;
    fn new_set() -> Self::Set;
    fn add(set: &mut Self::Set, g: &Handle<Self::Geometry>) -> i32;
    fn index(set: &Self::Set, g: &Handle<Self::Geometry>) -> i32;
    fn get(set: &Self::Set, index: i32) -> Handle<Self::Geometry>;
    fn dump_set(set: &Self::Set, s: &mut IoStream) -> Result<(), Failure>;
    fn write_set(set: &Self::Set, s: &mut IoStream, p: &ProgressRange<'_>) -> Result<(), Failure>;
    fn read_set(set: &mut Self::Set, s: &mut IoStream, p: &ProgressRange<'_>);
    fn clear(set: &mut Self::Set);
}

/// `gt_summary`: type name (or `null`), compact text at precision 17 with its console
/// output, and a point.
pub fn summary<K: GtKind>(g: &Handle<K::Geometry>) {
    let Some(kind) = g.dynamic_type() else {
        print!("null ");
        return;
    };
    print!("{} ", kind.name());
    let mut stream = IoStream::from_bytes(Vec::new());
    stream.set_precision(17);
    let console = captured(|| attempt_captured(|| K::write(g, &mut stream)));
    text(&super::written_bytes(stream));
    text(&console);
    attempt(|| K::value(g));
}

macro_rules! set_members {
    ($set:ty, $geometry:ty, $get:ident) => {
        type Set = $set;
        fn new_set() -> $set {
            <$set>::with_console(super::console::probe_console())
        }
        fn add(set: &mut $set, g: &Handle<$geometry>) -> i32 {
            set.add(g)
        }
        fn index(set: &$set, g: &Handle<$geometry>) -> i32 {
            set.index(g)
        }
        fn get(set: &$set, index: i32) -> Handle<$geometry> {
            set.$get(index)
        }
        fn dump_set(set: &$set, s: &mut IoStream) -> Result<(), Failure> {
            set.dump(s)
        }
        fn write_set(set: &$set, s: &mut IoStream, p: &ProgressRange<'_>) -> Result<(), Failure> {
            set.write(s, p)
        }
        fn read_set(set: &mut $set, s: &mut IoStream, p: &ProgressRange<'_>) {
            set.read(s, p)
        }
        fn clear(set: &mut $set) {
            set.clear()
        }
    };
}

pub struct Curves;
impl GtKind for Curves {
    type Geometry = dyn GeomCurve;
    set_members!(CurveSet, dyn GeomCurve, curve);
    fn build(r: &mut Reader<'_>) -> Result<Handle<dyn GeomCurve>, Failure> {
        build::curve(r)
    }
    fn dump(g: &Handle<dyn GeomCurve>, s: &mut IoStream) -> Result<(), Failure> {
        geom_tools::dump_curve(g, s)
    }
    fn write(g: &Handle<dyn GeomCurve>, s: &mut IoStream) -> Result<(), Failure> {
        geom_tools::write_curve(g, s)
    }
    fn read(s: &mut IoStream) -> Handle<dyn GeomCurve> {
        geom_tools::read_curve(s)
    }
    fn value(g: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        pnt(g.value(0.3)?);
        Ok(())
    }
}

pub struct Curves2d;
impl GtKind for Curves2d {
    type Geometry = dyn Geom2dCurve;
    set_members!(Curve2dSet, dyn Geom2dCurve, curve2d);
    fn build(r: &mut Reader<'_>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        build::curve2d(r)
    }
    fn dump(g: &Handle<dyn Geom2dCurve>, s: &mut IoStream) -> Result<(), Failure> {
        geom_tools::dump_curve2d(g, s)
    }
    fn write(g: &Handle<dyn Geom2dCurve>, s: &mut IoStream) -> Result<(), Failure> {
        geom_tools::write_curve2d(g, s)
    }
    fn read(s: &mut IoStream) -> Handle<dyn Geom2dCurve> {
        geom_tools::read_curve2d(s)
    }
    fn value(g: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        pnt2d(g.value(0.3)?);
        Ok(())
    }
}

pub struct Surfaces;
impl GtKind for Surfaces {
    type Geometry = dyn GeomSurface;
    set_members!(SurfaceSet, dyn GeomSurface, surface);
    fn build(r: &mut Reader<'_>) -> Result<Handle<dyn GeomSurface>, Failure> {
        build::surface(r)
    }
    fn dump(g: &Handle<dyn GeomSurface>, s: &mut IoStream) -> Result<(), Failure> {
        geom_tools::dump_surface(g, s)
    }
    fn write(g: &Handle<dyn GeomSurface>, s: &mut IoStream) -> Result<(), Failure> {
        geom_tools::write_surface(g, s)
    }
    fn read(s: &mut IoStream) -> Handle<dyn GeomSurface> {
        geom_tools::read_surface(s)
    }
    fn value(g: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        pnt(g.value(0.3, 0.2)?);
        Ok(())
    }
}
