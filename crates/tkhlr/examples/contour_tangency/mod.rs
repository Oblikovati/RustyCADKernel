mod domain;
// Real restriction searches feed the private helper, including prefilled destinations.
use super::{
    common::{Reader, attempt, end, flag, int, pnt, real, xy, xyz},
    contap_surface, tangency,
};
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
use rustycad_tkg3d::adaptor3d::{Adaptor3dTopolTool, h_surface_tool::Adaptor3dSurfaceHandle};
use rustycad_tkgeomalgo::int_surf::IntSurfPathPoint;
use rustycad_tkhlr::contap::{ContapArcFunction, ContapSurfFunction, ContapTheSearch};
use rustycad_tkmath::gp::Point3;
pub fn case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| run(&mut reader));
    end();
}
struct Scenario {
    search: ContapTheSearch,
    domain: domain::OrientedDomain,
    function: ContapSurfFunction,
    destination: Array1<'static, i32>,
    output: Vec<IntSurfPathPoint>,
}
fn scenario(reader: &mut Reader) -> Result<Scenario, Failure> {
    let surface = contap_surface::surface_adaptor(reader)?;
    let domain = Adaptor3dTopolTool::with_surface(&surface)?;
    let (mut arc, function) = functions(&surface, reader)?;
    let mode = reader.integer();
    let orientation = reader.integer();
    let mut search = ContapTheSearch::new();
    search.perform(&mut arc, &domain, 1e-7, 1e-7, false)?;
    Scenario::new(search, domain, function, mode, orientation)
}

impl Scenario {
    fn new(
        search: ContapTheSearch,
        domain: Adaptor3dTopolTool,
        function: ContapSurfFunction,
        mode: i32,
        orientation: i32,
    ) -> Result<Self, Failure> {
        let destination = destinations(search.nb_points()?, mode);
        let output = initial_output(mode);
        Ok(Scenario {
            search,
            domain: domain::OrientedDomain {
                tool: domain,
                mode: orientation,
            },
            function,
            destination,
            output,
        })
    }

    fn perform(&mut self) -> Result<(), Failure> {
        tangency::compute_tangency(
            &self.search,
            &self.domain,
            &mut self.function,
            &mut self.output,
            &mut self.destination,
        )
    }
}
fn run(reader: &mut Reader) -> Result<(), Failure> {
    let mut case = scenario(reader)?;
    flag(case.search.is_done());
    int(case.search.nb_points()?);
    int(case.search.nb_segments()?);
    attempt(|| case.perform());
    int(case.output.len() as i32);
    for point in &case.output {
        report_point(point)?;
    }
    for index in 1..=case.search.nb_points()? {
        int(case.destination[index]);
    }
    Ok(())
}
fn initial_output(mode: i32) -> Vec<IntSurfPathPoint> {
    if mode != 3 {
        return Vec::new();
    }
    vec![IntSurfPathPoint::with_value(
        Point3::new(7., 8., 9.),
        7.,
        8.,
    )]
}
fn destinations(count: i32, mode: i32) -> Array1<'static, i32> {
    let mut destination = Array1::with_bounds(1, count.max(1)).unwrap();
    for index in 1..=count.max(1) {
        destination[index] = match mode {
            1 if index % 2 == 0 => -3,
            2 if index % 2 != 0 => 2,
            _ => 0,
        };
    }
    destination
}
fn functions(
    surface: &Adaptor3dSurfaceHandle,
    reader: &mut Reader,
) -> Result<(ContapArcFunction, ContapSurfFunction), Failure> {
    let mode = reader.integer();
    let direction = reader.dir();
    let mut arc = ContapArcFunction::new();
    arc.set_surface(surface)?;
    let mut function = ContapSurfFunction::new();
    function.set_surface(surface)?;
    configure_functions(&mut arc, &mut function, mode, direction);
    function.set_tolerance(1e-7);
    Ok((arc, function))
}
fn configure_functions(
    arc: &mut ContapArcFunction,
    function: &mut ContapSurfFunction,
    mode: i32,
    direction: rustycad_tkmath::gp::Direction3,
) {
    match mode {
        0 => {
            arc.set_direction(direction);
            function.set_direction(direction);
        }
        1 => {
            let eye = Point3::new(3., 4., 5.);
            arc.set_eye(eye);
            function.set_eye(eye);
        }
        _ => {
            arc.set_direction_angle(direction, 0.2);
            function.set_direction_angle(direction, 0.2);
        }
    }
}
fn report_point(point: &IntSurfPathPoint) -> Result<(), Failure> {
    pnt(point.value());
    flag(point.is_passing_pnt());
    flag(point.is_tangent());
    int(point.multiplicity()?);
    for index in 0..=point.multiplicity()? {
        let (u, v) = point.parameters(index)?;
        real(u);
        real(v);
    }
    attempt(|| {
        xyz(*point.direction3d()?.xyz());
        Ok(())
    });
    attempt(|| {
        xy(point.direction2d()?.xy());
        Ok(())
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const SPHERE_CORNER: [f64; 23] = [
        1., 0., 0., 0., 0., 0., 1., 1., 0., 0., 0., 2., 0.4, 0., 1., 0., 1., 0., 0., 1., 1., 0., 0.,
    ];
    fn sphere_corner(seed: f64, orientation: f64) -> Scenario {
        let mut values = SPHERE_CORNER;
        values[21] = seed;
        values[22] = orientation;
        scenario(&mut Reader::new(&values)).unwrap()
    }
    #[test]
    fn rejected_corner_marks_entire_vertex_group_negative() {
        let mut case = sphere_corner(0., 1.);
        case.perform().unwrap();
        assert!(case.output.is_empty());
        assert_eq!((case.destination[1], case.destination[2]), (-1, -1));
    }
    #[test]
    fn internal_corner_keeps_both_uvs_and_shared_copy() {
        let mut case = sphere_corner(0., 2.);
        case.perform().unwrap();
        assert_eq!(case.output.len(), 1);
        assert!(case.output[0].is_passing_pnt());
        assert!(!case.output[0].is_tangent());
        assert_eq!(case.output[0].multiplicity().unwrap(), 1);
        assert_eq!((case.destination[1], case.destination[2]), (1, 1));
        let copied = case.output[0].clone();
        copied.add_uv(3., 4.).unwrap();
        assert_eq!(case.output[0].multiplicity().unwrap(), 2);
    }
    #[test]
    fn prefilled_output_does_not_offset_destination_numbering() {
        let mut case = sphere_corner(3., 2.);
        case.perform().unwrap();
        assert_eq!(case.output.len(), 2);
        assert_eq!((case.destination[1], case.destination[2]), (1, 1));
        assert_eq!(case.output[0].value(), Point3::new(7., 8., 9.));
    }
}
