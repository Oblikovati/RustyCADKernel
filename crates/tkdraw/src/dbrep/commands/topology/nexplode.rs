// Rust translation of the `nexplode` command of Draw/TKDraw/DBRep/DBRep.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `nexplode`: an explode numbered by position, stable under re-meshing.
use crate::dbrep::commands::{CommandEntry, sub_name};
use crate::draw::{DrawInterpretor, DrawSession};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{Explorer, map_shapes_of_type};
use rustycad_tkbrep::topods::{Shape, vertex as to_vertex};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// `nexplode`.
pub(in crate::dbrep::commands) const NEXPLODE: CommandEntry = (
    "nexplode",
    "stable numbered explode for vertex, edge and face: nexplode name [V/E/F]",
    nexplode,
);

/// `nexplode name F|E|V`: the distinct faces, edges or vertices named `name_1`... in
/// the order of `999 x + 99 y + 0.9 z` of their centres of mass (vertex points),
/// sorted by a bubble sort; ties are reported.
fn nexplode(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() <= 2 {
        return Ok(1);
    }
    let shape = session.shape(a[1])?;
    if shape.is_null() {
        return Ok(0);
    }
    let kind = match a[2].as_bytes().first().map(u8::to_ascii_uppercase) {
        Some(b'F') => ShapeEnum::Face,
        Some(b'E') => ShapeEnum::Edge,
        Some(b'V') => ShapeEnum::Vertex,
        _ => return Ok(1),
    };
    let shapes = distinct_shapes(&shape, kind)?;
    let keys: Vec<f64> = shapes.iter().map(sort_key).collect::<Result<_, _>>()?;
    let order = bubble_order(&keys);
    for pair in order.windows(2) {
        if keys[pair[1]] == keys[pair[0]] {
            di.append("Warning! For this shape the results may be incorrect.\n")?;
        }
    }
    for (position, index) in order.iter().enumerate() {
        let name = sub_name(a[1], position + 1);
        session.set_shape(&name, &shapes[*index])?;
        di.append_element(&name)?;
    }
    Ok(0)
}

/// The sub-shapes of `kind` in exploration order, each once, at most as many as the
/// indexed map of the shape and its sub-shapes counts minus one (`MaxShapes`).
fn distinct_shapes(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut indexed = IndexedMapOfShape::default();
    indexed.add(shape.clone());
    map_shapes_of_type(shape, kind, &mut indexed)?;
    let max_shapes = indexed.extent().saturating_sub(1);
    let mut seen = MapOfShape::default();
    let mut shapes = Vec::new();
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        if seen.add(explorer.current().clone()) {
            shapes.push(explorer.current().clone());
        }
        explorer.next()?;
    }
    shapes.truncate(max_shapes);
    Ok(shapes)
}

/// `X * 999 + Y * 99 + Z * 0.9` of the centre of mass (linear properties), or of the
/// point of a vertex.
fn sort_key(shape: &Shape) -> Result<f64, Failure> {
    let point = if shape.shape_type()? == ShapeEnum::Vertex {
        BRepTool::pnt(&to_vertex(shape)?)?
    } else {
        let mut props = GProps::default();
        BRepGProp::linear_properties(shape, &mut props, false, false)?;
        props.centre_of_mass()
    };
    Ok(point.x() * 999. + point.y() * 99. + point.z() * 0.9)
}

/// The indices of `keys` sorted ascending by the C++ bubble sort (swap on `>`).
fn bubble_order(keys: &[f64]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..keys.len()).collect();
    let mut swapped = true;
    while swapped {
        swapped = false;
        for i in 1..order.len() {
            if keys[order[i - 1]] > keys[order[i]] {
                order.swap(i - 1, i);
                swapped = true;
            }
        }
    }
    order
}

#[cfg(test)]
mod tests {
    use super::bubble_order;

    #[test]
    fn the_bubble_order_is_stable_on_ties() {
        assert_eq!(bubble_order(&[3.0, 1.0, 3.0, 2.0]), vec![1, 3, 0, 2]);
    }
}
