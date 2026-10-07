// Rust translation of the p-curve members of TopOpeBRepDS_BuildTool (TranslateOnPeriodic, the
// two PCurve overloads and the function TopOpeBRepDS_SetThePCurve in
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_BuildTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The p-curves of the built edges: translated into the period of the face, then set
//! on the edge (overwriting, or as the second p-curve of a closed edge).
use super::TopOpeBRepDSBuildTool;
use super::curves::fun_uiso_line_on_sphe;
use crate::top_ope_brep_ds::TopOpeBRepDSCurve;
use crate::top_ope_brep_tool::TopOpeBRepToolShapeTool;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dGeometry, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::gp::{Direction2, Vector2};

/// Standard_NullObject of a null geometry met by the p-curve members.
fn null_object(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("TopOpeBRepDS_BuildTool: null {what}; expected a geometry"),
        "TopOpeBRepDS_BuildTool",
    )
}

/// `TopOpeBRepDS_SetThePCurve(B, E, F, O, C)`: on a non-plane face with an existing
/// p-curve, `c` set with it as the pair of a closed edge (the order given by `o`, not
/// for a degenerated edge); else `c` set as the p-curve (tolerance Confusion);
/// `topopebrepds_set_the_pcurve(&b, &e, &f, Orientation::Forward, &pc)?`.
pub fn topopebrepds_set_the_pcurve(
    b: &BRepBuilder,
    e: &Edge,
    f: &Face,
    o: Orientation,
    c: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    let (surface, _) = BRepTool::surface_with_location(f)?;
    let gp: Handle<GeomPlane> = surface.cast();
    let oc = if gp.is_null() {
        BRepTool::curve_on_face(e, f)?.pcurve
    } else {
        Handle::default()
    };
    if oc.is_null() {
        return b.update_edge_pcurve_on_face(e, c, f, precision::confusion());
    }
    if BRepTool::degenerated(e)? {
        return Ok(());
    }
    let pair = if o == Orientation::Reversed {
        (&oc, c)
    } else {
        (c, &oc)
    };
    b.update_edge_pcurves_on_face(e, pair, f, precision::confusion())
}

impl TopOpeBRepDSBuildTool {
    /// `TranslateOnPeriodic(F, E, PC)`: the p-curve replaced by a copy translated so
    /// that its point at the middle of the 3d curve range of the edge falls in the
    /// period of the face (the projection of the 3d point for an iso-U line on a
    /// sphere); `bt.translate_on_periodic(&face, &edge, &mut pc)?`.
    pub fn translate_on_periodic(
        &self,
        f: &Shape,
        e: &Shape,
        pc: &mut Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        let (c3d, c3df, c3dl) = BRepTool::curve(&topods::edge(e)?)?;
        let curve = c3d.get().ok_or_else(|| null_object("3d curve"))?;
        let (first, mut last) = (c3df, c3dl);
        if curve.is_periodic() && last < first {
            last += (first - last).abs();
        }
        let uiso_line_on_sphe = fun_uiso_line_on_sphe(f, pc)?;
        let t = (first + last) * 0.5;
        let ptest = pc.get().ok_or_else(|| null_object("p-curve"))?.value(t)?;
        let (u1, v1) = (ptest.x(), ptest.y());
        let (mut u2, mut v2) = (u1, v1);
        if uiso_line_on_sphe {
            let p3dtest = GeomAdaptorCurve::with_curve(&c3d)?.value(t)?;
            let surf = BRepTool::surface(&topods::face(f)?)?;
            let pons = ProjectPointOnSurf::with_surface(p3dtest, &surf, ExtAlgo::Grad)?;
            if pons.nb_points()? >= 1 {
                (u2, v2) = pons.lower_distance_parameters()?;
            }
        } else {
            TopOpeBRepToolShapeTool::adjust_on_periodic(f, &mut u2, &mut v2)?;
        }
        let (du, dv) = (u2 - u1, v2 - v1);
        if du != 0. || dv != 0. {
            let pct: Handle<dyn Geom2dCurve> = pc
                .get()
                .ok_or_else(|| null_object("p-curve"))?
                .copy()?
                .cast();
            pct.get()
                .ok_or_else(|| null_object("p-curve"))?
                .translate(Vector2::new(du, dv))?;
            *pc = pct;
        }
        Ok(())
    }

    /// The INTERNAL vertices of the edge given their parameters on the face.
    fn update_internal_vertices_on_face(&self, ee: &Edge, ff: &Face) -> Result<(), Failure> {
        for vi in Explorer::new(ee.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)? {
            if vi.orientation() != Orientation::Internal {
                continue;
            }
            let v = topods::vertex(&vi)?;
            let tolvi = TopOpeBRepToolShapeTool::tolerance(&vi)?;
            let newpar = BRepTool::parameter(&v, ee)?;
            self.my_builder
                .update_vertex_on_edge_face(&v, newpar, ee, ff, tolvi)?;
        }
        Ok(())
    }

    /// `PCurve(F, E, PC)`: the p-curve (translated into the period when Translate) set
    /// on the edge, overwriting unless OverWrite is off or it is an iso-U line on a
    /// sphere (then TopOpeBRepDS_SetThePCurve), and the INTERNAL vertices updated;
    /// `bt.pcurve(&mut face, &mut edge, &pc)?`.
    pub fn pcurve(
        &self,
        f: &mut Shape,
        e: &mut Shape,
        pc: &Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        if pc.is_null() {
            return Ok(());
        }
        let (ff, ee) = (topods::face(f)?, topods::edge(e)?);
        let mut pct = pc.clone();
        let uiso_line_on_sphe = fun_uiso_line_on_sphe(f, pc)?;
        let overwrite = if uiso_line_on_sphe {
            false
        } else {
            self.my_over_write
        };
        if self.my_translate {
            self.translate_on_periodic(f, e, &mut pct)?;
        }
        if overwrite {
            self.my_builder
                .update_edge_pcurve_on_face(&ee, &pct, &ff, 0.)?;
        } else {
            topopebrepds_set_the_pcurve(&self.my_builder, &ee, &ff, e.orientation(), &pct)?;
        }
        self.update_internal_vertices_on_face(&ee, &ff)
    }

    /// The p-curve trimmed to the range of the curve, reversed and reparametrized on
    /// the range of the edge curve.
    fn reversed_pcurve(
        pct: &Handle<dyn Geom2dCurve>,
        (cdsmin, cdsmax): (f64, f64),
        (cf, cl): (f64, f64),
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let reversed = Geom2dTrimmedCurve::new(pct, cdsmin, cdsmax, true, true)?;
        reversed.reverse()?;
        let (f, l) = (reversed.first_parameter(), reversed.last_parameter());
        let reversed: Handle<dyn Geom2dCurve> = Handle::new(reversed).cast();
        let mut out = pct.clone();
        geom_lib::same_range(
            precision::p_confusion(),
            &reversed,
            (f, l),
            (cf, cl),
            &mut out,
        )?;
        Ok(out)
    }

    /// A line p-curve translated in place along the periodic direction it follows by
    /// the shift of the curve range start.
    fn shift_periodic_line(
        line2d: &Geom2dLine,
        ff: &Face,
        (cdsmin, cf): (f64, f64),
    ) -> Result<(), Failure> {
        let (surf, _) = BRepTool::surface_with_location(ff)?;
        let s = surf.get().ok_or_else(|| null_object("surface"))?;
        let dir2d = line2d.direction();
        if s.is_u_periodic() && dir2d.is_parallel(Direction2::X, precision::angular()) {
            line2d.translate(Vector2::new((cdsmin - cf) * dir2d.x(), 0.))?;
        } else if s.is_v_periodic() && dir2d.is_parallel(Direction2::Y, precision::angular()) {
            line2d.translate(Vector2::new(0., (cdsmin - cf) * dir2d.y()))?;
        }
        Ok(())
    }

    /// `PCurve(F, E, CDS, PC)`: the p-curve set by TopOpeBRepDS_SetThePCurve; when the
    /// edge has a 3d curve and the curve has a range, reversed onto the edge range for
    /// a reversed existing or equivalent curve, else a line on a periodic edge curve
    /// translated (in place) by the shift of the range start;
    /// `bt.pcurve_of_curve(&mut face, &mut edge, &c, &pc)?`.
    pub fn pcurve_of_curve(
        &self,
        f: &mut Shape,
        e: &mut Shape,
        cds: &TopOpeBRepDSCurve,
        pc: &Handle<dyn Geom2dCurve>,
    ) -> Result<(), Failure> {
        if pc.is_null() {
            return Ok(());
        }
        let (ff, ee) = (topods::face(f)?, topods::edge(e)?);
        let mut pct = pc.clone();
        let range = cds.range();
        let found = BRepTool::curve_with_location(&ee)?;
        if let Some(c) = found.curve.get() {
            let (cf, cl) = (found.first, found.last);
            let (cdsmin, cdsmax) = range.unwrap_or((0., 0.));
            if range.is_some()
                && (cds.is_existing_edge_reversed() || cds.is_equivalent_curve_reversed())
            {
                pct = Self::reversed_pcurve(&pct, (cdsmin, cdsmax), (cf, cl))?;
            } else {
                let deca = (cf - cdsmin).abs() > precision::p_confusion();
                let line2d: Handle<Geom2dLine> = pct.clone().cast();
                if let Some(l) = line2d.get()
                    && range.is_some()
                    && deca
                    && c.is_periodic()
                {
                    Self::shift_periodic_line(l, &ff, (cdsmin, cf))?;
                }
            }
        }
        topopebrepds_set_the_pcurve(&self.my_builder, &ee, &ff, e.orientation(), &pct)
    }
}
