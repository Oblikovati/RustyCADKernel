//! The GC package probe; the counterpart of tools/oracle/tkgeombase_probe_gc.cxx, split
//! like its curve and surface files.
mod curves;
mod surfaces;
pub use curves::*;
pub use surfaces::*;

use super::common::{self as p, Reader};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dLine, Geom2dParabola,
    Geom2dTransformation, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomEllipse, GeomHyperbola,
    GeomLine, GeomParabola, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
    GeomTransformation, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{
    Circle2, Circle3, Ellipse2, Ellipse3, Hyperbola2, Hyperbola3, Parabola2, Parabola3,
};

/// The class name of a handle, `null` for a null one (gb_name).
pub fn name<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) {
    match handle.dynamic_type() {
        Some(kind) => print!("{} ", kind.name()),
        None => print!("null "),
    }
}

/// gb_geom_curve: the class, then the trimmed bounds and basis or the gp element.
pub fn geom_curve<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) {
    let curve: Handle<dyn GeomCurve> = handle.cast();
    name(&curve);
    if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
        p::real(trimmed.first_parameter());
        p::real(trimmed.last_parameter());
        return geom_curve(&trimmed.basis_curve());
    }
    if let Some(line) = curve.cast::<GeomLine>().get() {
        return p::lin(line.lin());
    }
    geom_conic(&curve);
}

fn geom_conic(curve: &Handle<dyn GeomCurve>) {
    if let Some(x) = curve.cast::<GeomCircle>().get() {
        p::circ(x.circ().unwrap());
    } else if let Some(x) = curve.cast::<GeomEllipse>().get() {
        p::elips(x.elips().unwrap());
    } else if let Some(x) = curve.cast::<GeomHyperbola>().get() {
        p::hypr(x.hypr().unwrap());
    } else if let Some(x) = curve.cast::<GeomParabola>().get() {
        p::parab(x.parab().unwrap());
    }
}

/// gb_geom2d_curve: the class, then the trimmed bounds and basis or the gp element.
pub fn geom2d_curve<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) {
    let curve: Handle<dyn Geom2dCurve> = handle.cast();
    name(&curve);
    if let Some(trimmed) = curve.cast::<Geom2dTrimmedCurve>().get() {
        p::real(trimmed.first_parameter());
        p::real(trimmed.last_parameter());
        return geom2d_curve(&trimmed.basis_curve());
    }
    if let Some(line) = curve.cast::<Geom2dLine>().get() {
        return p::lin2d(line.lin2d());
    }
    geom2d_conic(&curve);
}

fn geom2d_conic(curve: &Handle<dyn Geom2dCurve>) {
    if let Some(x) = curve.cast::<Geom2dCircle>().get() {
        p::circ2d(x.circ2d().unwrap());
    } else if let Some(x) = curve.cast::<Geom2dEllipse>().get() {
        p::elips2d(x.elips2d().unwrap());
    } else if let Some(x) = curve.cast::<Geom2dHyperbola>().get() {
        p::hypr2d(x.hypr2d().unwrap());
    } else if let Some(x) = curve.cast::<Geom2dParabola>().get() {
        p::parab2d(x.parab2d().unwrap());
    }
}

/// gb_geom_surface: the class, then the trimmed bounds and basis or the gp element.
pub fn geom_surface<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) {
    let surface: Handle<dyn GeomSurface> = handle.cast();
    name(&surface);
    if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
        let (u1, u2, v1, v2) = trimmed.bounds();
        [u1, u2, v1, v2].into_iter().for_each(p::real);
        return geom_surface(&trimmed.basis_surface());
    }
    if let Some(x) = surface.cast::<GeomPlane>().get() {
        p::pln(x.pln());
    } else if let Some(x) = surface.cast::<GeomConicalSurface>().get() {
        p::cone(x.cone().unwrap());
    } else if let Some(x) = surface.cast::<GeomCylindricalSurface>().get() {
        p::cylinder(x.cylinder().unwrap());
    }
}

/// gb_geom_trsf and gb_geom2d_trsf.
pub fn geom_trsf(handle: &Handle<GeomTransformation>) {
    name(handle);
    if let Some(t) = handle.get() {
        p::trsf(t.trsf());
    }
}
pub fn geom2d_trsf(handle: &Handle<Geom2dTransformation>) {
    name(handle);
    if let Some(t) = handle.get() {
        p::trsf2d(t.trsf2d());
    }
}

/// The conversion operator of a GC_Root algorithm, typed like its `value()`.
pub fn converted<M, H>(maker: &M, _like: &Result<H, Failure>) -> Result<H, Failure>
where
    for<'a> H: TryFrom<&'a M, Error = Failure>,
{
    H::try_from(maker)
}

/// gb_gc_root inside GB_TRY: status, IsDone, IsError, Value() and the conversion.
macro_rules! gc_root {
    ($print:path, $maker:expr) => {
        crate::gb_probe::common::attempt(|| {
            use rustycad_tkgeombase::gc::Root;
            let maker = $maker?;
            crate::gb_probe::common::int(maker.status() as i32);
            crate::gb_probe::common::flag(maker.is_done());
            crate::gb_probe::common::flag(maker.is_error());
            let value = maker.value();
            let conversion = crate::gb_probe::gc::converted(&maker, &value);
            crate::gb_probe::common::show(value, |h| $print(&h));
            crate::gb_probe::common::show(conversion, |h| $print(&h));
            Ok(())
        })
    };
}

/// gb_gc_trsf inside GB_TRY: Value() and the conversion of a transformation maker.
macro_rules! gc_trsf {
    ($print:path, $maker:expr) => {
        crate::gb_probe::common::attempt(|| {
            let maker = $maker?;
            $print(&maker.value());
            $print(&rustycad_tkernel::standard::Handle::from(&maker));
            Ok(())
        })
    };
}
pub(super) use {gc_root, gc_trsf};

/// A maker that cannot raise, in the shape of the fallible ones.
pub(super) fn ok<T>(value: T) -> Result<T, Failure> {
    Ok(value)
}

/// The gp readers of tools/oracle/tkgeombase_probe_gc_curves.cxx (gb_read_*).
impl Reader<'_> {
    pub fn circ(&mut self) -> Circle3 {
        let f = self.ax2();
        Circle3::new(f, self.real()).unwrap()
    }
    pub fn elips(&mut self) -> Ellipse3 {
        let (f, a) = (self.ax2(), self.real());
        Ellipse3::new(f, a, self.real()).unwrap()
    }
    pub fn hypr(&mut self) -> Hyperbola3 {
        let (f, a) = (self.ax2(), self.real());
        Hyperbola3::new(f, a, self.real()).unwrap()
    }
    pub fn parab(&mut self) -> Parabola3 {
        let f = self.ax2();
        Parabola3::new(f, self.real()).unwrap()
    }
    pub fn circ2d(&mut self) -> Circle2 {
        let f = self.ax22d();
        Circle2::new(f, self.real()).unwrap()
    }
    pub fn elips2d(&mut self) -> Ellipse2 {
        let (f, a) = (self.ax22d(), self.real());
        Ellipse2::new(f, a, self.real()).unwrap()
    }
    pub fn hypr2d(&mut self) -> Hyperbola2 {
        let (f, a) = (self.ax22d(), self.real());
        Hyperbola2::new(f, a, self.real()).unwrap()
    }
    pub fn parab2d(&mut self) -> Parabola2 {
        let f = self.ax22d();
        Parabola2::new(f, self.real()).unwrap()
    }
}
