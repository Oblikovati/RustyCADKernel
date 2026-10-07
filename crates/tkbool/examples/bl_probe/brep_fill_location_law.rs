//! BRepFill_LocationLaw probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_brepfill_location_law.cxx (the records are documented there).
use super::brep_fill_generator::failed;
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_fill::{
    BRepFillACRLaw, BRepFillDraftLaw, BRepFillEdge3DLaw, BRepFillEdgeOnSurfLaw, BRepFillLocationLaw,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCompCurve;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillConstantBiNormal, GeomFillCorrectedFrenet, GeomFillCurveAndTrihedron, GeomFillFrenet,
    GeomFillLocationDraft, GeomFillLocationLaw, GeomFillTrihedronLaw,
};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillGuideTrihedronAC, GeomFillLocationGuide, GeomFillTrihedronWithGuide,
};
use rustycad_tkmath::gp::Direction3;

/// `bl_vertex`.
fn vertex(v: &Vertex) -> Result<(), Failure> {
    if v.is_null() {
        int(-1);
        return Ok(());
    }
    pnt(BRepTool::pnt(v)?);
    real(BRepTool::vertex_tolerance(v)?);
    Ok(())
}

/// The D0 of the section at an abscissa, printed.
fn d0_digest(law: &dyn BRepFillLocationLaw, abscissa: f64, section: &Shape) -> Result<(), Failure> {
    shape(&law.d0(abscissa, section)?)
}

/// The first vertex of the section (TopExp_Explorer order).
fn first_vertex(section: &Shape) -> Vertex {
    Explorer::new(section, ShapeEnum::Vertex, ShapeEnum::Shape)
        .ok()
        .filter(Explorer::more)
        .map(|e| Vertex::unchecked(e.current().clone()))
        .unwrap_or_default()
}

/// The queries before the lengths are known and the per-vertex queries.
fn first_queries(law: &dyn BRepFillLocationLaw, n: i32) {
    if n > 0 {
        attempt(|| {
            let (f, l) = law
                .law(n)?
                .get()
                .map(|g| g.get_domain())
                .transpose()?
                .unwrap_or((0., 0.));
            real(law.abscissa(n, 0.5 * (f + l))?);
            Ok(())
        });
    }
    attempt(|| {
        int(law.get_status()? as i32);
        Ok(())
    });
    attempt(|| {
        flag(law.is_closed()?);
        Ok(())
    });
    for i in 0..=n {
        attempt(|| {
            int(law.is_g1(i, 1.0e-7, 1.0e-4)?);
            Ok(())
        });
    }
    attempt(|| {
        let h = law.nb_holes(1.0e-7)?;
        int(h);
        let mut d = vec![0; h as usize];
        law.holes(&mut d);
        d.iter().for_each(|&k| int(k));
        Ok(())
    });
    for i in 1..=n + 1 {
        attempt(|| vertex(&law.vertex(i)?));
    }
}

/// `bl_location_law_queries`.
fn location_law_queries(law: &dyn BRepFillLocationLaw, section: &Shape) {
    let mut n: i32 = 0;
    attempt(|| {
        n = law.nb_law()?;
        int(n);
        Ok(())
    });
    first_queries(law, n);
    let mut total: f64 = 0.;
    for i in 1..=n {
        attempt(|| {
            let (f, l) = law.curvilinear_bounds(i)?;
            real(f);
            real(l);
            total = l;
            Ok(())
        });
    }
    for a in [0., 0.3 * total, 0.5 * total, total, 1.1 * total + 1.] {
        attempt(|| {
            let (ind, u) = law.parameter(a)?;
            int(ind);
            if ind != 0 {
                real(u);
            }
            Ok(())
        });
    }
    let sv = first_vertex(section);
    if !sv.is_null() {
        for i in 0..=n {
            for loc in -1..=1 {
                attempt(|| vertex(&law.perform_vertex(i, &sv, 1.0e-4, loc)?));
            }
        }
    }
    attempt(|| d0_digest(law, 0.25 * total, section));
    attempt(|| {
        law.transform_in_g0_law()?;
        d0_digest(law, 0.25 * total, section)
    });
    attempt(|| {
        law.transform_in_compatible_law(0.1)?;
        d0_digest(law, 0.25 * total, section)
    });
    attempt(|| {
        law.delete_transform()?;
        d0_digest(law, 0.25 * total, section)
    });
}

/// The shape of the program at a table index.
fn at(t: &TpShapes, index: i32) -> Result<Shape, Failure> {
    t.shapes
        .get(index as usize)
        .cloned()
        .ok_or_else(|| Failure::out_of_range("bl_probe: shape index"))
}

/// The law of a request; `None` (printed) for an EdgeOnSurfLaw without result.
fn law_of(
    r: &mut Reader<'_>,
    t: &TpShapes,
    path: &Shape,
) -> Result<Option<Handle<dyn BRepFillLocationLaw>>, Failure> {
    let path = topods::wire(path)?;
    match r.integer() {
        0 => {
            let tri: Handle<dyn GeomFillTrihedronLaw> = match r.integer() {
                0 => Handle::new(GeomFillCorrectedFrenet::new()).cast(),
                1 => Handle::new(GeomFillFrenet::new()).cast(),
                _ => Handle::new(GeomFillConstantBiNormal::new(Direction3::Z)).cast(),
            };
            let model: Handle<dyn GeomFillLocationLaw> =
                Handle::new(GeomFillCurveAndTrihedron::new(&tri)).cast();
            Ok(Some(
                Handle::new(BRepFillEdge3DLaw::new(&path, &model)?).cast(),
            ))
        }
        1 => {
            let (d, a) = (r.dir(), r.real());
            let draft =
                BRepFillDraftLaw::new(&path, &Handle::new(GeomFillLocationDraft::new(d, a)?))?;
            draft.clean_law(0.1)?;
            Ok(Some(Handle::new(draft).cast()))
        }
        3 => {
            let guide_wire = topods::wire(&at(t, r.integer())?)?;
            let guide: Handle<dyn Adaptor3dCurve> =
                Handle::new(BRepAdaptorCompCurve::with_wire(&guide_wire, false)?).cast();
            let tri: Handle<dyn GeomFillTrihedronWithGuide> =
                Handle::new(GeomFillGuideTrihedronAC::new(&guide)?).cast();
            let lg = Handle::new(GeomFillLocationGuide::new(&tri)?);
            Ok(Some(Handle::new(BRepFillACRLaw::new(&path, &lg)?).cast()))
        }
        _ => {
            let law = BRepFillEdgeOnSurfLaw::new(&path, &at(t, r.integer())?)?;
            flag(law.has_result());
            if !law.has_result() {
                return Ok(None);
            }
            Ok(Some(Handle::new(law).cast()))
        }
    }
}

/// `bf_location_law program path section kind ...`.
pub fn location_law_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = match tp_shapes(&mut r) {
        Ok(t) => t,
        Err(error) => return failed(error),
    };
    let (path, section) = match (at(&t, r.integer()), at(&t, r.integer())) {
        (Ok(p), Ok(s)) => (p, s),
        (Err(error), _) | (_, Err(error)) => return failed(error),
    };
    match law_of(&mut r, &t, &path) {
        Ok(Some(law)) => match law.get() {
            Some(law) => location_law_queries(law, &section),
            None => return failed(Failure::out_of_range("bl_probe: null law")),
        },
        Ok(None) => {}
        Err(error) => return failed(error),
    }
    end();
}
