use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{CompSolid, Compound, Edge, Shape, Shell, Solid, Wire},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
/// Replace output with an empty supported container, otherwise leave it alone; `make_container(ShapeEnum::Shell,&mut shape)`.
pub fn make_container(kind: ShapeEnum, shape: &mut Shape) {
    let builder = BRepBuilder::default();
    macro_rules! empty {
        ($kind:ty,$method:ident) => {{
            let mut container = <$kind>::default();
            builder.$method(&mut container);
            container.into_shape()
        }};
    }
    *shape = match kind {
        ShapeEnum::Compound => empty!(Compound, make_compound),
        ShapeEnum::CompSolid => empty!(CompSolid, make_comp_solid),
        ShapeEnum::Solid => empty!(Solid, make_solid),
        ShapeEnum::Shell => empty!(Shell, make_shell),
        ShapeEnum::Wire => empty!(Wire, make_wire),
        _ => return,
    };
}
/// Evaluate the located spatial curve without clamping its edge range; `point_on_edge(&edge,parameter,&mut point)?`.
pub fn point_on_edge(edge: &Edge, parameter: f64, point: &mut Point3) -> Result<(), Failure> {
    let (curve, _, _) = BRepTool::curve(edge)?;
    *point = curve.value(parameter)?;
    Ok(())
}
