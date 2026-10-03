//! Transfer source-face p-curves to a temporary same-support reference face.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct TransferReport {
    pub transferred_edges: usize,
    pub missing_source_edges: usize,
}

pub(super) fn transfer_first_cylinder_pair(shape: &Shape) -> Result<TransferReport, Failure> {
    let Some((reference, source)) = first_cylinder_pair(shape)? else {
        return Ok(TransferReport::default());
    };
    transfer_to_reference(&reference, &source)
}

fn first_cylinder_pair(shape: &Shape) -> Result<Option<(Face, Face)>, Failure> {
    let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut edge_faces)?;
    for index in 1..=edge_faces.extent() {
        let faces = edge_faces
            .find_from_index(index)?
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        if let Some(pair) = cylinder_pair(&faces)? {
            return Ok(Some(pair));
        }
    }
    Ok(None)
}

fn cylinder_pair(faces: &[Shape]) -> Result<Option<(Face, Face)>, Failure> {
    for first_index in 0..faces.len() {
        for second_index in first_index + 1..faces.len() {
            let first = topods::face(&faces[first_index])?;
            let second = topods::face(&faces[second_index])?;
            if same_cylindrical_support(&first, &second)? {
                return Ok(Some((first, second)));
            }
        }
    }
    Ok(None)
}

fn same_cylindrical_support(first: &Face, second: &Face) -> Result<bool, Failure> {
    let first_type = rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface::with_face(first, false)?;
    let second_type = rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface::with_face(second, false)?;
    if first_type.get_type()? != SurfaceType::Cylinder
        || second_type.get_type()? != SurfaceType::Cylinder
    {
        return Ok(false);
    }
    let (first_surface, first_location) = BRepTool::surface_with_location(first)?;
    let (second_surface, second_location) = BRepTool::surface_with_location(second)?;
    Ok(
        first_surface.object_address() == second_surface.object_address()
            && first_location == second_location,
    )
}

fn transfer_to_reference(reference: &Face, source: &Face) -> Result<TransferReport, Failure> {
    let (surface, location) = BRepTool::surface_with_location(reference)?;
    let mut reference_face = Face::default();
    BRepBuilder::default().make_face_on_located_surface(
        &mut reference_face,
        &surface,
        &location,
        precision::confusion(),
    )?;
    reference_face.set_orientation(reference.orientation());
    transfer_face_curves(&reference_face, source)
}

fn transfer_face_curves(reference: &Face, source: &Face) -> Result<TransferReport, Failure> {
    let mut report = TransferReport::default();
    for edge in unique_face_edges(source)? {
        if transfer_edge_curves(reference, source, &edge)? {
            report.transferred_edges += 1;
        } else {
            report.missing_source_edges += 1;
        }
    }
    Ok(report)
}

fn unique_face_edges(face: &Face) -> Result<Vec<Edge>, Failure> {
    let explorer = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    let mut result = Vec::new();
    for shape in explorer {
        let edge = topods::edge(&shape)?;
        if !result.iter().any(|known: &Edge| known.is_same(&edge)) {
            result.push(edge);
        }
    }
    Ok(result)
}

fn transfer_edge_curves(reference: &Face, source: &Face, edge: &Edge) -> Result<bool, Failure> {
    let forward = topods::edge(&edge.oriented(Orientation::Forward))?;
    let reverse = topods::edge(&edge.oriented(Orientation::Reversed))?;
    let first = BRepTool::curve_on_face(&forward, source)?;
    let second = BRepTool::curve_on_face(&reverse, source)?;
    if !first.is_stored || !second.is_stored {
        return Ok(false);
    }
    copy_edge_curves(
        reference,
        &forward,
        &first.pcurve,
        &second.pcurve,
        first.first,
        first.last,
    )?;
    Ok(true)
}

fn copy_edge_curves(
    reference: &Face,
    edge: &Edge,
    first: &Handle<dyn Geom2dCurve>,
    second: &Handle<dyn Geom2dCurve>,
    first_parameter: f64,
    last_parameter: f64,
) -> Result<(), Failure> {
    let first_copy = copy_pcurve(first)?;
    let builder = BRepBuilder::default();
    if first == second {
        builder.update_edge_pcurve_on_face(edge, &first_copy, reference, 0.0)?;
    } else {
        let second_copy = copy_pcurve(second)?;
        builder.update_edge_pcurves_on_face(edge, (&first_copy, &second_copy), reference, 0.0)?;
    }
    builder.range(edge, first_parameter, last_parameter, false)
}

fn copy_pcurve(curve: &Handle<dyn Geom2dCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    curve
        .get()
        .ok_or_else(|| Failure::new("ShapeUpgrade_UnifySameDomain requires a source p-curve"))?
        .copy()
        .map(|copy| copy.cast::<dyn Geom2dCurve>())
}
