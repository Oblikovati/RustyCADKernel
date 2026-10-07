//! ShapeCustom_Surface probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_custom_surface.cxx (which documents the requests).
use super::common::{Reader, attempt, end, real};
use super::construct_package::{bspline_surface, probe_surface};
use super::sh_common::surface;
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkshhealing::shape_custom::ShapeCustomSurface;

fn analytical(r: &mut Reader<'_>, tool: &mut ShapeCustomSurface) {
    let (tol, substitute) = (r.real(), r.flag());
    attempt(|| {
        surface(&tool.convert_to_analytical(tol, substitute)?);
        Ok(())
    });
    real(tool.gap());
}

fn periodic(r: &mut Reader<'_>, tool: &mut ShapeCustomSurface) {
    let (substitute, preci) = (r.flag(), r.real());
    attempt(|| {
        let p = tool.convert_to_periodic(substitute, preci)?;
        surface(&p);
        bspline_surface(&p.cast::<GeomBSplineSurface>());
        Ok(())
    });
}

/// `sc_surface (surface) nops (op)...`.
pub fn custom_surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = probe_surface(&mut r).expect("probe surface record");
    let mut tool = ShapeCustomSurface::with_surface(&s);
    for _ in 0..r.integer() {
        if r.integer() == 0 {
            analytical(&mut r, &mut tool);
        } else {
            periodic(&mut r, &mut tool);
        }
    }
    surface(&s);
    bspline_surface(&s.cast::<GeomBSplineSurface>());
    end();
}
