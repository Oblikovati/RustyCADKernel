//! Real Contap boundary roots and segments, including all result getters.
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkhlr::contap::{
    ContapArcFunction, ContapThePathPointOfTheSearch as Point, ContapTheSearch as Search,
    ContapTheSegmentOfTheSearch as Segment,
};
fn arc_rank(d: &Adaptor3dTopolTool, arc: &TheArc) -> i32 {
    let mut rank = 0;
    d.init();
    while d.more() {
        rank += 1;
        if d.value().is_ok_and(|a| &a == arc) {
            return rank;
        }
        d.next();
    }
    0
}

fn path_point(d: &Adaptor3dTopolTool, p: &Point) {
    pnt(p.value());
    real(p.tolerance());
    flag(p.is_new());
    real(p.parameter());
    int(arc_rank(d, &p.arc()));
    attempt(|| {
        let v = p.vertex()?;
        pnt2d(v.value());
        real(v.resolution(&p.arc())?);
        Ok(())
    });
}

fn segment(d: &Adaptor3dTopolTool, g: &Segment) {
    int(arc_rank(d, &g.curve()));
    flag(g.has_first_point());
    attempt(|| g.first_point().map(|p| path_point(d, p)));
    flag(g.has_last_point());
    attempt(|| g.last_point().map(|p| path_point(d, p)));
}

fn sob_report(s: &Search, d: &Adaptor3dTopolTool) {
    flag(s.is_done());
    attempt(|| {
        flag(s.all_arc_solution()?);
        Ok(())
    });
    attempt(|| {
        let n = s.nb_points()?;
        int(n);
        (1..=n).try_for_each(|i| {
            path_point(d, s.point(i)?);
            Ok(())
        })
    });
    attempt(|| {
        let n = s.nb_segments()?;
        int(n);
        (1..=n).try_for_each(|i| {
            segment(d, s.segment(i)?);
            Ok(())
        })
    });
    attempt(|| s.point(0).map(|_| ()));
    attempt(|| s.point(s.nb_points()? + 1).map(|_| ()));
    attempt(|| s.segment(0).map(|_| ()));
    attempt(|| s.segment(s.nb_segments()? + 1).map(|_| ()));
}
/// Compare unset and repeated boundary searches; `boundary_case(numbers)`.
pub fn boundary_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = super::contap_surface::surface_adaptor(&mut reader)?;
        let mode = reader.integer();
        let kind = reader.integer();
        let recheck = reader.flag();
        let direction = reader.dir();
        let eye = reader.pnt();
        let angle = reader.real();
        let mut function = ContapArcFunction::new();
        function.set_surface(&surface)?;
        match kind {
            0 => function.set_direction(direction),
            1 => function.set_eye(eye),
            _ => function.set_direction_angle(direction, angle),
        }
        let domain = Adaptor3dTopolTool::with_surface(&surface)?;
        let mut search = Search::new();
        sob_report(&search, &domain);
        for _ in 0..mode {
            attempt(|| search.perform(&mut function, &domain, 1e-7, 1e-7, recheck));
            sob_report(&search, &domain);
        }
        Ok(())
    });
    end();
}
