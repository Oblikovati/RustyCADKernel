//! Contap interior search on real surface adaptors.
use super::common::{Reader, attempt, end, flag, int, pnt, real, xy, xyz};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkhlr::contap::{ContapSurfFunction, ContapTheSearchInside as Search};
fn inside_report(s: &Search) {
    flag(s.is_done());
    attempt(|| {
        let n = s.nb_points()?;
        int(n);
        (1..=n).try_for_each(|i| {
            let p = s.value(i)?;
            pnt(p.value());
            real(p.u_parameter());
            real(p.v_parameter());
            xyz(*p.direction().xyz());
            xy(p.direction2d().xy());
            Ok(())
        })
    });
    attempt(|| s.value(0).map(|_| ()));
    attempt(|| s.value(s.nb_points()? + 1).map(|_| ()));
}

/// Compare all interior-search entry points and repeated performs; `inside_case(numbers)`.
pub fn inside_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = super::contap_surface::surface_adaptor(&mut reader)?;
        let mode = reader.integer();
        let kind = reader.integer();
        let u = reader.real();
        let v = reader.real();
        let mut function = ContapSurfFunction::new();
        function.set_surface(&surface)?;
        let direction = reader.dir();
        let eye = reader.pnt();
        let angle = reader.real();
        match kind {
            0 => function.set_direction(direction),
            1 => function.set_eye(eye),
            _ => function.set_direction_angle(direction, angle),
        }
        function.set_tolerance(1e-7);
        let domain = Adaptor3dTopolTool::with_surface(&surface)?;
        let mut search = Search::new();
        if mode == 0 {
            attempt(|| {
                search = Search::with_domain(&mut function, &surface, &domain, 1e-7)?;
                Ok(())
            });
        }
        if mode == 1 {
            attempt(|| search.perform_from(&mut function, &surface, u, v));
        }
        if mode == 3 {
            attempt(|| search.perform(&mut function, &surface, &domain, 1e-7));
        }
        inside_report(&search);
        if mode != 2 {
            attempt(|| search.perform_from(&mut function, &surface, u, v));
            inside_report(&search);
        }
        Ok(())
    });
    end();
}
