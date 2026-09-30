use super::{flag, integer, real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_start::{TheArc, TheTopolTool};
use rustycad_tkmath::gp::Point2;
pub fn emit(domain: &dyn TheTopolTool) -> Result<(), Failure> {
    flag(domain.domain_is_infinite()?);
    flag(domain.has_3d());
    integer(domain.classify(Point2::default(), 1e-7, true)? as i32);
    flag(domain.is_the_point_on(Point2::default(), 1e-7, true)?);
    flag(domain.is_uniform_sampling()?);
    domain.init();
    while domain.more() {
        let arc = domain.value()?;
        integer(domain.arc_orientation(&arc)? as i32);
        flag(domain.edge()?.is_some());
        print!("{} ", domain.tol_3d_arc(&arc).unwrap_err().exception_type());
        vertices(domain, &arc)?;
        domain.next();
    }
    flag(domain.more());
    domain.sample_pnts(0.1, 4, 5)?;
    for array in [domain.u_parameters()?, domain.v_parameters()?] {
        real(array[1]);
    }
    Ok(())
}
fn vertices(domain: &dyn TheTopolTool, arc: &TheArc) -> Result<(), Failure> {
    domain.initialize_arc(arc)?;
    domain.init_vertex_iterator()?;
    while domain.more_vertex() {
        let vertex = domain.vertex()?;
        integer(domain.vertex_orientation(&vertex)? as i32);
        flag(domain.identical(&vertex, &vertex)?);
        print!(
            "{} ",
            domain.tol_3d_vertex(&vertex).unwrap_err().exception_type()
        );
        domain.next_vertex()?;
    }
    flag(domain.more_vertex());
    Ok(())
}
