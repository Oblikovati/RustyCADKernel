//! Native shared sampling state, arc projection, and vertex forwarding.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show};
use super::contap_sampling_fixture::sampling_surface;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertexBase, h_surface_tool::Adaptor3dSurfaceHandle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkhlr::contap::ContapHContTool as Tool;
fn sample_pair(surface: &Adaptor3dSurfaceHandle, index: i32) {
    show(Tool::sample_point(surface, index), |(u, v)| {
        real(u);
        real(v);
    });
}
/// Compare state across surfaces and failed cache refreshes; `sampling_case(numbers)`.
pub fn sampling_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let baseline = sampling_surface(&mut reader);
    let changed = sampling_surface(&mut reader);
    show(Tool::nb_sample_points(&baseline), int);
    show(Tool::nb_samples_u(&changed, -1.0, 2.0), int);
    show(Tool::nb_samples_v(&changed, -1.0, 2.0), int);
    for index in [-1, 0, 1, 2, 3, 4, 5, 6, 17, 40] {
        sample_pair(&changed, index);
    }
    show(Tool::nb_sample_points(&changed), int);
    for index in [-1, 0, 1, 2, 3, 4, 5, 6, 17, 40] {
        sample_pair(&baseline, index);
        sample_pair(&changed, index);
    }
    end();
}
fn arc_constants(arc: &TheArc) {
    flag(Tool::has_been_seen(arc));
    int(Tool::nb_points(arc));
    attempt(|| {
        Tool::value(arc, 1)?;
        Ok(())
    });
    flag(Tool::is_vertex(arc, 1));
    attempt(|| {
        Tool::vertex(arc, 1)?;
        Ok(())
    });
    int(Tool::nb_segments(arc));
    attempt(|| {
        Tool::has_first_point(arc, 1)?;
        Ok(())
    });
    attempt(|| {
        Tool::has_last_point(arc, 1)?;
        Ok(())
    });
    flag(Tool::is_all_solution(arc));
}
/// Compare every arc/vertex member on real adaptors; `arc_case(numbers)`.
pub fn arc_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let arc = super::contap_hcurve::arc(&mut reader)?;
        let point = reader.pnt2d();
        let vertex = Handle::new(Adaptor3dHVertexBase::new(
            point,
            Orientation::Forward,
            reader.real(),
        ))
        .cast();
        show(Tool::nb_samples_on_arc(&arc), int);
        show(Tool::bounds(&arc), |(first, last)| {
            real(first);
            real(last);
        });
        show(Tool::project(&arc, point), |projection| {
            flag(projection.is_some());
            if let Some((parameter, point)) = projection {
                real(parameter);
                pnt2d(point);
            }
        });
        show(Tool::tolerance(&vertex, &arc), real);
        show(Tool::parameter(&vertex, &arc), real);
        arc_constants(&arc);
        Ok(())
    });
    end();
}
