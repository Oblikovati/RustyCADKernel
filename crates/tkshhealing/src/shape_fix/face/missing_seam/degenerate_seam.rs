use super::plan::MissingSeamPlan;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Builder, Edge, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSphericalSurface, GeomToroidalSurface};
use rustycad_tkmath::gp::{Direction2, Point2};

struct DegenerateSeamLine {
    origin: Point2,
    direction: Direction2,
    range: f64,
}

pub(super) fn create_for_single_open_wire(plan: &mut MissingSeamPlan) -> Result<bool, Failure> {
    if !plan.second_wire.is_null() {
        return Ok(true);
    }
    let Some(line) = single_open_wire_line(plan)? else {
        return Ok(false);
    };
    let wire = create_degenerate_wire(plan, line)?;
    plan.second_wire = wire.clone();
    plan.wires.push(wire);
    Ok(true)
}

fn single_open_wire_line(plan: &MissingSeamPlan) -> Result<Option<DegenerateSeamLine>, Failure> {
    if plan.direction.u != 0 && plan.degenerate_torus_surface() {
        return Ok(Some(plan.torus_line()?));
    }
    if plan.direction.u != 0 && !plan.surface.cast::<GeomSphericalSurface>().is_null() {
        return Ok(Some(plan.sphere_line()));
    }
    if !plan.surface.cast::<GeomBSplineSurface>().is_null() {
        return plan.spline_line();
    }
    Ok(None)
}

impl MissingSeamPlan {
    fn degenerate_torus_surface(&self) -> bool {
        let torus = self.surface.cast::<GeomToroidalSurface>();
        torus
            .get()
            .is_some_and(|surface| surface.major_radius() < surface.minor_radius())
    }

    fn torus_line(&self) -> Result<DegenerateSeamLine, Failure> {
        let torus = self.surface.cast::<GeomToroidalSurface>();
        let torus = torus.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_TypeMismatch",
                "ShapeFix_Face: expected toroidal surface",
            )
        })?;
        let phi = (-torus.major_radius() / torus.minor_radius()).acos();
        let v = if self.direction.u > 0 {
            std::f64::consts::PI + phi
        } else {
            phi
        };
        Ok(DegenerateSeamLine {
            origin: Point2::new(0.0, v),
            direction: Direction2::new(-(self.direction.u as f64), 0.0)?,
            range: std::f64::consts::TAU,
        })
    }

    fn sphere_line(&self) -> DegenerateSeamLine {
        DegenerateSeamLine {
            origin: Point2::new(
                if self.direction.u < 0 {
                    0.0
                } else {
                    std::f64::consts::TAU
                },
                self.direction.u as f64 * 0.5 * std::f64::consts::PI,
            ),
            direction: Direction2::from_standard(if self.direction.u < 0 {
                rustycad_tkmath::gp::StandardDirection2::X
            } else {
                rustycad_tkmath::gp::StandardDirection2::NegX
            }),
            range: std::f64::consts::TAU,
        }
    }

    fn spline_line(&self) -> Result<Option<DegenerateSeamLine>, Failure> {
        if self.direction.v != 0 {
            return self.spline_v_line();
        }
        if self.direction.u != 0 {
            return self.spline_u_line();
        }
        Ok(None)
    }

    fn spline_v_line(&self) -> Result<Option<DegenerateSeamLine>, Failure> {
        let (u_first, u_last, v_first, v_last) = self.surface_bounds;
        let middle_v = (v_first + v_last) / 2.0;
        let first = self.surface.value(u_first, v_first)?;
        let lower_middle = self.surface.value(u_first, middle_v)?;
        let upper_first = self.surface.value(u_last, v_first)?;
        if first.distance(&lower_middle) < precision::confusion() {
            return Ok(Some(vertical_seam(
                u_first,
                self.direction.v,
                self.v_period,
            )));
        }
        if upper_first.distance(&self.surface.value(u_last, middle_v)?) < precision::confusion() {
            return Ok(Some(vertical_seam(u_last, self.direction.v, self.v_period)));
        }
        Ok(None)
    }

    fn spline_u_line(&self) -> Result<Option<DegenerateSeamLine>, Failure> {
        let (u_first, u_last, v_first, v_last) = self.surface_bounds;
        let middle_u = (u_first + u_last) / 2.0;
        let first = self.surface.value(u_first, v_first)?;
        let lower_middle = self.surface.value(middle_u, v_first)?;
        let upper_corner = self.surface.value(u_last, v_last)?;
        if first.distance(&lower_middle) < precision::confusion() {
            return Ok(Some(horizontal_seam(
                v_first,
                self.direction.u,
                self.u_period,
            )));
        }
        if upper_corner.distance(&self.surface.value(middle_u, v_last)?) < precision::confusion() {
            return Ok(Some(horizontal_seam(
                v_last,
                self.direction.u,
                self.u_period,
            )));
        }
        Ok(None)
    }
}

fn horizontal_seam(v: f64, direction: i8, range: f64) -> DegenerateSeamLine {
    let origin_u = if direction < 0 { 0.0 } else { range };
    DegenerateSeamLine {
        origin: Point2::new(origin_u, v),
        direction: Direction2::from_standard(if direction < 0 {
            rustycad_tkmath::gp::StandardDirection2::X
        } else {
            rustycad_tkmath::gp::StandardDirection2::NegX
        }),
        range,
    }
}

fn vertical_seam(u: f64, direction: i8, range: f64) -> DegenerateSeamLine {
    let origin_v = if direction < 0 { 0.0 } else { range };
    DegenerateSeamLine {
        origin: Point2::new(u, origin_v),
        direction: Direction2::from_standard(if direction < 0 {
            rustycad_tkmath::gp::StandardDirection2::Y
        } else {
            rustycad_tkmath::gp::StandardDirection2::NegY
        }),
        range,
    }
}

fn create_degenerate_wire(
    plan: &MissingSeamPlan,
    line: DegenerateSeamLine,
) -> Result<Wire, Failure> {
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(line.origin, line.direction)).cast();
    let edge = make_degenerate_edge(plan, &pcurve, line.range)?;
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, &edge.into_shape())?;
    Ok(wire)
}

fn make_degenerate_edge(
    plan: &MissingSeamPlan,
    pcurve: &Handle<dyn Geom2dCurve>,
    range: f64,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    builder.degenerated(&edge, true)?;
    builder.update_edge_pcurve_on_face(&edge, pcurve, &plan.face, precision::confusion())?;
    let (support, location) = BRepTool::surface_with_location(&plan.face)?;
    builder.range_on_surface(&edge, &support, &location, 0.0, range)?;
    add_degenerate_vertex(
        &builder,
        &edge,
        plan.surface
            .value(pcurve.value(0.0)?.x(), pcurve.value(0.0)?.y())?,
    )?;
    Ok(edge)
}

fn add_degenerate_vertex(
    builder: &BRepBuilder,
    edge: &Edge,
    point: rustycad_tkmath::gp::Point3,
) -> Result<(), Failure> {
    let mut vertex = Vertex::default();
    builder.make_vertex_at(&mut vertex, point, precision::confusion())?;
    let mut shape = edge.clone().into_shape();
    Builder.add(&mut shape, &vertex.clone().into_shape())?;
    Builder.add(
        &mut shape,
        &vertex
            .into_shape()
            .oriented(rustycad_tkg3d::top_abs::Orientation::Reversed),
    )
}
