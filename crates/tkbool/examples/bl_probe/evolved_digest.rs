//! Address-independent twin of tools/oracle/tkbool_probe_evolved_digest.cxx.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp,
    topods::{self, Edge, Face, Shape, ShapeIterator},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::top_abs::{self, ShapeEnum};
use rustycad_tkmath::gp::Point3;

fn key_point(key: &mut Vec<u64>, point: Point3) {
    key.extend([
        point.x().to_bits(),
        point.y().to_bits(),
        point.z().to_bits(),
    ]);
}
fn key_type<T: RuntimeTyped + ?Sized>(key: &mut Vec<u64>, value: &Handle<T>) {
    let name = value.dynamic_type().map_or("null", |kind| kind.name());
    key.push(name.len() as u64);
    key.extend(name.bytes().map(u64::from));
}
fn key_children(mut children: Vec<Vec<u64>>) -> Vec<u64> {
    children.sort();
    let mut key = vec![children.len() as u64];
    for child in children {
        key.push(child.len() as u64);
        key.extend(child);
    }
    key
}
fn key_pcurves(edge: &Edge) -> Result<Vec<u64>, Failure> {
    let mut rows = Vec::new();
    for index in 1.. {
        let found = BRepTool::curve_on_surface_by_index(edge, index)?;
        let Some(curve) = found.pcurve.get() else {
            break;
        };
        let mut row = vec![
            u64::from(found.location.is_identity()),
            found.first.to_bits(),
            found.last.to_bits(),
        ];
        key_type(&mut row, &found.surface);
        key_type(&mut row, &found.pcurve);
        for u in [found.first, found.last, 0.5 * (found.first + found.last)] {
            let point = curve.value(u)?;
            row.extend([point.x().to_bits(), point.y().to_bits()]);
        }
        rows.push(row);
    }
    Ok(key_children(rows))
}
fn key_edge(key: &mut Vec<u64>, edge: &Edge) -> Result<(), Failure> {
    key.extend([
        BRepTool::edge_tolerance(edge)?.to_bits(),
        u64::from(BRepTool::same_parameter(edge)?),
        u64::from(BRepTool::same_range(edge)?),
        u64::from(BRepTool::degenerated(edge)?),
    ]);
    let (curve, first, last) = BRepTool::curve(edge)?;
    key_type(key, &curve);
    if let Some(curve) = curve.get() {
        key.extend([first.to_bits(), last.to_bits()]);
        for u in [first, last, 0.5 * (first + last)] {
            key_point(key, curve.value(u)?);
        }
    }
    key.extend(key_pcurves(edge)?);
    Ok(())
}
fn key_face(key: &mut Vec<u64>, face: &Face) -> Result<(), Failure> {
    key.extend([
        BRepTool::face_tolerance(face)?.to_bits(),
        u64::from(BRepTool::natural_restriction(face)?),
    ]);
    let surface = BRepTool::surface(face)?;
    key_type(key, &surface);
    let Some(surface) = surface.get() else {
        return Ok(());
    };
    key_point(key, surface.value(0.25, 0.5)?);
    let frame = surface.d1(0.3, 0.7)?;
    key_point(key, frame.point);
    key_point(
        key,
        Point3::new(frame.d1u.x(), frame.d1u.y(), frame.d1u.z()),
    );
    key_point(
        key,
        Point3::new(frame.d1v.x(), frame.d1v.y(), frame.d1v.z()),
    );
    Ok(())
}
fn key_incidence(shape: &Shape, solid: bool) -> Result<Vec<u64>, Failure> {
    let (mut children, mut reversed) = (Vec::new(), Vec::new());
    let mut iterator = ShapeIterator::new(
        shape,
        solid && shape.shape_type()? == ShapeEnum::Shell,
        true,
    );
    while iterator.more() {
        let child = iterator.value()?;
        let mut key = evolved_key(child, solid)?;
        if shape.shape_type()? == ShapeEnum::Edge {
            key.push(
                BRepTool::parameter(&topods::vertex(child)?, &topods::edge(shape)?)?.to_bits(),
            );
        }
        key.insert(
            0,
            if solid && child.shape_type()? == ShapeEnum::Shell {
                0
            } else {
                child.orientation() as u64
            },
        );
        children.push(key.clone());
        key[0] = top_abs::reverse(child.orientation()) as u64;
        reversed.push(key);
        iterator.next();
    }
    let key = key_children(children);
    Ok(if !solid && shape.shape_type()? == ShapeEnum::Shell {
        key.min(key_children(reversed))
    } else {
        key
    })
}
fn evolved_key(shape: &Shape, in_solid: bool) -> Result<Vec<u64>, Failure> {
    if shape.is_null() {
        return Ok(vec![u64::MAX]);
    }
    let kind = shape.shape_type()?;
    let mut key = vec![
        kind as u64,
        u64::from(shape.closed()?),
        u64::from(shape.location().is_identity()),
    ];
    key.extend(key_incidence(shape, in_solid || kind == ShapeEnum::Solid)?);
    match kind {
        ShapeEnum::Vertex => {
            let vertex = topods::vertex(shape)?;
            key_point(&mut key, BRepTool::pnt(&vertex)?);
            key.push(BRepTool::vertex_tolerance(&vertex)?.to_bits());
        }
        ShapeEnum::Edge => key_edge(&mut key, &topods::edge(shape)?)?,
        ShapeEnum::Face => key_face(&mut key, &topods::face(shape)?)?,
        _ => {}
    }
    Ok(key)
}
/// Compare geometry and oriented incidence independently of hash traversal; `evolved_digest(&shape)?`.
pub fn evolved_digest(shape: &Shape) -> Result<(), Failure> {
    let key = evolved_key(shape, false)?;
    print!("{} ", key.len());
    for value in key {
        print!("{value} ");
    }
    for kind in [
        ShapeEnum::Compound,
        ShapeEnum::CompSolid,
        ShapeEnum::Solid,
        ShapeEnum::Shell,
        ShapeEnum::Face,
        ShapeEnum::Wire,
        ShapeEnum::Edge,
        ShapeEnum::Vertex,
    ] {
        let mut shapes = IndexedMapOfShape::default();
        if !shape.is_null() {
            topexp::map_shapes_of_type(shape, kind, &mut shapes)?;
        }
        print!("{} ", shapes.len());
    }
    Ok(())
}
