//! HLRTopoBRep_FaceIsoLiner.cxx: the two loops of iso lines of Perform (U, then V),
//! which differ only by the direction and the origin of the lines.
use super::{FaceIsoLiner, IsoContext, IsoEnds};
use crate::hlr_topo_brep::Data;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeomalgo::geom2d_hatch::Hatcher;
use rustycad_tkgeomalgo::hatch_gen::{Domain, PointOnHatching};
use rustycad_tkmath::gp::{self, Point2, Point3};

impl IsoContext<'_> {
    // The `nb_isos` lines of one direction, from `min + step / 2` by `step`.
    pub(super) fn iso_lines(
        &self,
        hatcher: &mut Hatcher,
        ds: &mut Data,
        (min, step, nb_isos): (f64, f64, i32),
        iso_u: bool,
    ) -> Result<(), Failure> {
        let mut ends = IsoEnds::default();
        let mut prm = min + step / 2.;
        let dir = if iso_u { gp::dy2d() } else { gp::dx2d() };
        for _ in 1..=nb_isos {
            let ori = if iso_u {
                Point2::new(prm, 0.)
            } else {
                Point2::new(0., prm)
            };
            let iso_line = Handle::new(Geom2dLine::from_point(ori, dir));
            let curve: Handle<dyn Geom2dCurve> = iso_line.clone().cast();
            let ind_h = hatcher.add_hatching(Geom2dAdaptorCurve::with_curve(&curve)?);
            hatcher.trim_hatching(ind_h)?;
            if hatcher.trim_done(ind_h)? && !hatcher.trim_failed(ind_h)? {
                hatcher.compute_domains_of(ind_h)?;
            }
            if !hatcher.is_done(ind_h)? {
                hatcher.rem_hatching(ind_h)?;
                continue;
            }
            for i_dom in 1..=hatcher.nb_domains(ind_h)? {
                let dom = hatcher.domain(ind_h, i_dom)?.clone();
                self.domain_line(&iso_line, &dom, &mut ends, ds)?;
            }
            hatcher.rem_hatching(ind_h)?;
            prm += step;
        }
        Ok(())
    }

    // One domain of an iso line: the end vertices on the elements it meets, then the
    // iso edge when both end vertices exist (they may come from an earlier domain).
    fn domain_line(
        &self,
        iso_line: &Handle<Geom2dLine>,
        dom: &Domain,
        ends: &mut IsoEnds,
        ds: &mut Data,
    ) -> Result<(), Failure> {
        let (vmin, vmax) = self.v_bounds;
        let u1 = if dom.has_first_point() {
            dom.first_point()?.parameter()
        } else {
            vmin - super::INFINITE
        };
        let u2 = if dom.has_second_point() {
            dom.second_point()?.parameter()
        } else {
            vmax + super::INFINITE
        };
        let p1 = self.surface_point(iso_line, u1)?;
        let p2 = self.surface_point(iso_line, u2)?;
        if dom.has_first_point() {
            self.end_vertices(dom.first_point()?, p1, &mut ends.v1, ds)?;
        }
        if dom.has_second_point() {
            self.end_vertices(dom.second_point()?, p2, &mut ends.v2, ds)?;
        }
        if !ends.v1.is_null() && !ends.v2.is_null() {
            let (v1, v2) = (&mut ends.v1, &mut ends.v2);
            FaceIsoLiner::make_iso_line(self.face, iso_line, v1, v2, u1, u2, self.tolerance, ds)?;
        }
        Ok(())
    }

    fn surface_point(&self, iso_line: &Handle<Geom2dLine>, u: f64) -> Result<Point3, Failure> {
        let line = iso_line
            .get()
            .ok_or_else(|| Failure::new("HLRTopoBRep_FaceIsoLiner: null iso line"))?;
        let p = line.d0(u)?;
        self.surface.d0(p.x(), p.y())
    }

    // The vertex of each element point of a hatching point; the last one is kept.
    fn end_vertices(
        &self,
        pnt_h: &PointOnHatching,
        p: Point3,
        vertex: &mut rustycad_tkbrep::topods::Vertex,
        ds: &mut Data,
    ) -> Result<(), Failure> {
        for i_pnt_e in 1..=pnt_h.nb_points() {
            let pnt_e = pnt_h.point(i_pnt_e)?;
            let edge = self.elements.edge(pnt_e.index())?;
            *vertex = FaceIsoLiner::make_vertex(&edge, p, pnt_e.parameter(), self.tolerance, ds)?;
            if self.elements.is_outline(pnt_e.index()) {
                ds.add_out_v(vertex);
            }
        }
        Ok(())
    }
}
