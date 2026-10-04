//! `BRepOffset_MakeOffset::CheckInputData` null, connectivity, continuity and normal gates.
use super::{BRepOffsetError, BRepOffsetMakeOffset};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::{Quilt, uv_bounds},
    topexp::{Explorer, first_vertex},
    topods::{Compound, Face, Shape, Vertex},
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::{geom::GeomSurface, top_abs::ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as GeomAbsShape;

impl BRepOffsetMakeOffset {
    /// Validates offset existence, face connectivity, continuity, and sampled surface normals.
    /// `maker.check_input_data()?`.
    pub fn check_input_data(&mut self) -> Result<bool, Failure> {
        let config = self.configuration()?;
        self.error = Some(BRepOffsetError::NoError);
        self.bad_shape = Shape::default();
        if is_null_offset(config.offset, config.tolerance)
            && !has_face_offset(self, config.tolerance)
        {
            return Ok(self.set_check_error(BRepOffsetError::NullOffset));
        }
        let faces = face_component(&self.initial_shape)?;
        if !is_connected_shell(&faces.clone().into_shape())? {
            return Ok(self.set_check_error(BRepOffsetError::NotConnectedShell));
        }
        self.check_face_geometry(&faces.clone().into_shape())
    }

    fn check_face_geometry(&mut self, faces: &Shape) -> Result<bool, Failure> {
        let mut seen_faces = Vec::new();
        for face_shape in Explorer::new(faces, ShapeEnum::Face, ShapeEnum::Shape)? {
            if seen_faces
                .iter()
                .any(|seen: &Shape| seen.is_partner(&face_shape))
            {
                continue;
            }
            seen_faces.push(face_shape.clone());
            if self.face_has_c0_geometry(&face_shape)? {
                return Ok(self.set_check_error(BRepOffsetError::C0Geometry));
            }
            if self.face_has_bad_normals(&face_shape)? {
                return Ok(self.set_check_error(BRepOffsetError::BadNormalsOnGeometry));
            }
        }
        Ok(true)
    }

    fn face_has_c0_geometry(&self, face_shape: &Shape) -> Result<bool, Failure> {
        let face = Face::unchecked(face_shape.clone());
        let surface = BRepTool::surface(&face)?;
        let surface = surface.get().ok_or_else(null_offset_surface)?;
        Ok(surface.continuity() == GeomAbsShape::C0)
    }

    fn face_has_bad_normals(&self, face_shape: &Shape) -> Result<bool, Failure> {
        let face = Face::unchecked(face_shape.clone());
        let surface = BRepTool::surface(&face)?;
        let surface = surface.get().ok_or_else(null_offset_surface)?;
        let bad_points = degenerate_boundary_points(&face)?;
        let (umin, umax, vmin, vmax) = uv_bounds(&face)?;
        if self.sample_normal_grid(surface, &bad_points, (umin, umax, vmin, vmax))? {
            return Ok(true);
        }
        self.face_has_bad_vertex_normals(&face, surface, &bad_points)
    }

    fn face_has_bad_vertex_normals(
        &self,
        face: &Face,
        surface: &dyn GeomSurface,
        bad_points: &[rustycad_tkmath::gp::Point3],
    ) -> Result<bool, Failure> {
        for shape in Explorer::new(
            &face.clone().into_shape(),
            ShapeEnum::Vertex,
            ShapeEnum::Shape,
        )? {
            let vertex = Vertex::unchecked(shape);
            let uv = BRepTool::parameters(&vertex, face)?;
            if is_bad_normal(surface, bad_points, uv.x(), uv.y())? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn sample_normal_grid(
        &self,
        surface: &dyn GeomSurface,
        bad_points: &[rustycad_tkmath::gp::Point3],
        bounds: (f64, f64, f64, f64),
    ) -> Result<bool, Failure> {
        for i in 0..=20 {
            let u = sample_parameter(bounds.0, bounds.1, i);
            if self.sample_normal_row(surface, bad_points, u, (bounds.2, bounds.3))? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn sample_normal_row(
        &self,
        surface: &dyn GeomSurface,
        bad_points: &[rustycad_tkmath::gp::Point3],
        u: f64,
        v_bounds: (f64, f64),
    ) -> Result<bool, Failure> {
        for j in 0..=20 {
            let v = sample_parameter(v_bounds.0, v_bounds.1, j);
            if is_bad_normal(surface, bad_points, u, v)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn set_check_error(&mut self, error: BRepOffsetError) -> bool {
        self.error = Some(error);
        false
    }
}

fn is_null_offset(offset: f64, tolerance: f64) -> bool {
    offset.abs() <= tolerance
}

fn has_face_offset(maker: &BRepOffsetMakeOffset, tolerance: f64) -> bool {
    maker
        .face_offsets
        .iter()
        .any(|(_, offset)| offset.abs() > tolerance)
}

fn face_component(shape: &Shape) -> Result<Compound, Failure> {
    let mut faces = Compound::default();
    BRepBuilder::default().make_compound(&mut faces);
    if shape.is_null() {
        return Ok(faces);
    }
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        BRepBuilder::default().add(&mut faces, &face)?;
    }
    Ok(faces)
}

fn is_connected_shell(face_component: &Shape) -> Result<bool, Failure> {
    let mut quilt = Quilt::new();
    quilt.add(face_component)?;
    let shells = quilt.shells()?;
    let mut explorer = Explorer::new(&shells, ShapeEnum::Shell, ShapeEnum::Shape)?;
    if !explorer.more() {
        return Ok(true);
    }
    explorer.next()?;
    Ok(!explorer.more())
}

fn degenerate_boundary_points(face: &Face) -> Result<Vec<rustycad_tkmath::gp::Point3>, Failure> {
    let mut points = Vec::new();
    for edge_shape in Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )? {
        let edge = rustycad_tkbrep::topods::Edge::unchecked(edge_shape);
        if BRepTool::degenerated(&edge)? {
            points.push(BRepTool::pnt(&first_vertex(&edge, true))?);
        }
    }
    Ok(points)
}

fn sample_parameter(first: f64, last: f64, index: usize) -> f64 {
    first + (last - first) * index as f64 / 20.0
}

fn is_bad_normal(
    surface: &dyn GeomSurface,
    bad_points: &[rustycad_tkmath::gp::Point3],
    u: f64,
    v: f64,
) -> Result<bool, Failure> {
    let d1 = surface.d1(u, v)?;
    if has_singular_derivative(d1.d1u.square_magnitude(), d1.d1v.square_magnitude()) {
        return Ok(!near_known_degenerate_point(&d1.point, bad_points));
    }
    Ok(d1.d1u.cross_magnitude(d1.d1v)
        <= precision::confusion() * d1.d1u.magnitude() * d1.d1v.magnitude())
}

fn has_singular_derivative(u_square: f64, v_square: f64) -> bool {
    u_square < precision::square_confusion() || v_square < precision::square_confusion()
}

fn near_known_degenerate_point(
    sample: &rustycad_tkmath::gp::Point3,
    bad_points: &[rustycad_tkmath::gp::Point3],
) -> bool {
    bad_points
        .iter()
        .any(|point| sample.square_distance(point) < precision::square_confusion())
}

fn null_offset_surface() -> Failure {
    Failure::with_message(
        "Standard_NoSuchObject",
        "BRepOffset_MakeOffset::CheckInputData has a null face surface; expected a geometric surface",
    )
}
