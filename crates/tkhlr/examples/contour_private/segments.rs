//! Segment processing includes existing endpoint aliases and negative tolerances.
use super::super::{
    common::{Reader, attempt, end, flag, int},
    contap_surface, process_segments,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkhlr::contap::{ContapLine, ContapPoint, ContapTheSearch};
fn seed(lines: &[ContapLine], search: &ContapTheSearch) -> Result<(), Failure> {
    for index in 1..=search.nb_segments()? {
        let segment = search.segment(index)?;
        if !segment.has_first_point() {
            continue;
        }
        let start = segment.first_point()?;
        let mut point = ContapPoint::with_value(start.value(), 3., 4.);
        point.set_parameter(start.parameter());
        lines[0].add_vertex(&point);
        lines[0].add_vertex(&point);
        lines[1].add_vertex(&point);
    }
    Ok(())
}
pub fn segments_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = contap_surface::surface_adaptor(&mut reader)?;
        let domain = Adaptor3dTopolTool::with_surface(&surface)?;
        let (mut arc, mut function) = super::attachment::functions(&surface, &mut reader)?;
        let seeded = reader.flag();
        let tolerance = reader.real();
        let mut search = ContapTheSearch::new();
        search.perform(&mut arc, &domain, 1e-7, 1e-7, false)?;
        flag(search.is_done());
        int(search.nb_segments()?);
        let mut lines = super::attachment::candidates()?;
        if seeded {
            seed(&lines, &search)?;
        }
        attempt(|| {
            process_segments::process_segments(
                &search,
                &mut lines,
                tolerance,
                &mut function,
                &domain,
            )
        });
        for line in &lines {
            super::sections::report(line)?;
        }
        int(-1);
        Ok(())
    });
    end();
}
