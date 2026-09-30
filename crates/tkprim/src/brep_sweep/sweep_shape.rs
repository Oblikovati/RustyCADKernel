// Rust translation of BRepSweep_NumLinearRegularSweep.cxx (Shape(aGenS, aDirS),
// IsUsed) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction of one cell of the sweep grid: the "floors" (a directing vertex),
//! the "walls" (the directing edge) and the directing wire case, each recursing on the
//! sub-shapes. A new cell is stored in the grid as soon as it is made: the geometric
//! members and `SetContinuity` read it back while its sub-shapes are added, which
//! share its TShape, as the native in-place array cell does.
use super::BRepSweepIterator;
use super::num_linear_regular_sweep::{BRepSweepNumLinearRegularSweep, kind_of};
use crate::sweep::{SweepNumShape, SweepNumShapeIterator};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};

/// The Standard_NoSuchObject of a generating shape the sweep cannot process.
fn no_such_object(kind: ShapeEnum) -> Failure {
    let what = if matches!(kind, ShapeEnum::Solid | ShapeEnum::CompSolid) {
        "Solids are not Processed"
    } else {
        "Unknown Shape"
    };
    Failure::with_message(
        "Standard_NoSuchObject",
        &format!(
            "BRepSweep_NumLinearRegularSweep::Shape: {what} ({kind:?}); expected a vertex, edge, wire, face, shell or compound"
        ),
    )
}

/// The grid indices of `[gen_s, dir_s]`.
fn indices<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &T,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
) -> (i32, i32) {
    let base = s.sweep_base();
    (
        base.my_gen_shape_tool.index(gen_s),
        base.my_dir_shape_tool.index(dir_s),
    )
}

/// The cell `(i, j)` of the grid.
fn cell<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &T,
    i: i32,
    j: i32,
) -> Result<Shape, Failure> {
    Ok(s.sweep_base().cell(i, j)?.clone())
}

/// `Shape(aGenS, aDirS)`: builds the cell once, then refreshes the Closed flag of a
/// wire or shell cell and returns it.
pub(crate) fn shape_at<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
) -> Result<Shape, Failure> {
    let (i, j) = indices(s, gen_s, dir_s);
    if !s.sweep_base().built(i, j)? {
        match s.sweep_base().my_dir_shape_tool.kind(dir_s) {
            ShapeEnum::Vertex => build_floor(s, gen_s, dir_s, (i, j))?,
            ShapeEnum::Edge => build_wall(s, gen_s, dir_s, (i, j))?,
            ShapeEnum::Wire => build_directing_wire(s, gen_s, dir_s, (i, j))?,
            _ => {}
        }
        s.sweep_base_mut().set_built(i, j, true)?;
    }
    let result = cell(s, i, j)?;
    // Change the "Closed" flag only for Wires and Shells.
    if matches!(result.shape_type()?, ShapeEnum::Wire | ShapeEnum::Shell) {
        result.set_closed(BRepTool::is_closed(&result)?)?;
    }
    Ok(result)
}

/// Makes the empty cell of a floor (a directing vertex) and stores it.
fn make_floor_cell<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
    (i, j): (i32, i32),
) -> Result<Shape, Failure> {
    let builder = s.sweep_base().my_builder;
    let mut new_cell = cell(s, i, j)?;
    match kind_of(gen_s)? {
        ShapeEnum::Vertex => new_cell = s.make_empty_vertex(gen_s, dir_s)?,
        ShapeEnum::Edge => new_cell = s.make_empty_generating_edge(gen_s, dir_s)?,
        ShapeEnum::Wire => builder.make_wire(&mut new_cell)?,
        ShapeEnum::Face => new_cell = s.make_empty_face(gen_s, dir_s)?,
        ShapeEnum::Shell => builder.make_shell(&mut new_cell)?,
        ShapeEnum::Compound => builder.make_compound(&mut new_cell)?,
        kind => return Err(no_such_object(kind)),
    }
    s.sweep_base_mut().set_cell(i, j, new_cell.clone())?;
    Ok(new_cell)
}

/// The floors of the shape: the generating shape itself at a directing vertex; the
/// sub-shapes are built identically to the generating ones, with their orientation.
fn build_floor<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
    (i, j): (i32, i32),
) -> Result<(), Failure> {
    let mut new_cell = make_floor_cell(s, gen_s, dir_s, (i, j))?;
    let gen_kind = kind_of(gen_s)?;
    let b_gen_s = gen_s.oriented(Orientation::Forward);
    let mut it = BRepSweepIterator::new();
    it.init(&b_gen_s);
    while it.more() {
        let (sub_gen_s, or) = (it.value()?.clone(), it.orientation()?);
        if s.has_shape(&sub_gen_s, dir_s)? {
            let new_shape = s.shape_at(&sub_gen_s, dir_s)?;
            let i_new = s.sweep_base().my_gen_shape_tool.index(&sub_gen_s);
            if s.ggd_shape_is_to_add(&new_cell, &new_shape, (gen_s, &sub_gen_s, dir_s))? {
                let builder = s.sweep_base().my_builder;
                builder.add_oriented(&mut new_cell, &new_shape, or)?;
                s.sweep_base_mut().set_used(i_new, j, true)?;
                let floor = FloorSub {
                    gen_s,
                    b_gen_s: &b_gen_s,
                    sub_gen_s: &sub_gen_s,
                    or,
                };
                floor_geometry(s, &new_cell, &new_shape, gen_kind, dir_s, floor)?;
            }
        }
        it.next();
    }
    Ok(())
}

/// A generating sub-shape of a floor: the generating shape (as given and FORWARD), the
/// sub-shape and its orientation.
struct FloorSub<'a> {
    gen_s: &'a Shape,
    b_gen_s: &'a Shape,
    sub_gen_s: &'a Shape,
    or: Orientation,
}

/// The geometry that a floor gives its new sub-shape: the parameters of a vertex and
/// the p-curves of the edges of a face (of a wire's edges too), the parameter of a
/// vertex on an edge.
fn floor_geometry<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    new_cell: &Shape,
    new_shape: &Shape,
    gen_kind: ShapeEnum,
    dir_s: &SweepNumShape,
    sub: FloorSub<'_>,
) -> Result<(), Failure> {
    let sub_kind = kind_of(sub.sub_gen_s)?;
    if gen_kind == ShapeEnum::Edge {
        return s.set_generating_parameter(
            new_cell,
            new_shape,
            (sub.b_gen_s, sub.sub_gen_s, dir_s),
        );
    }
    if gen_kind != ShapeEnum::Face {
        return Ok(());
    }
    match sub_kind {
        ShapeEnum::Vertex => {
            s.set_parameters(new_cell, new_shape, (sub.gen_s, sub.sub_gen_s, dir_s))
        }
        ShapeEnum::Edge => s.set_pcurve(
            new_cell,
            new_shape,
            (sub.gen_s, sub.sub_gen_s, dir_s),
            sub.or,
        ),
        ShapeEnum::Wire => wire_pcurves(s, new_cell, sub.gen_s, sub.sub_gen_s, dir_s),
        _ => Ok(()),
    }
}

/// The p-curves of the edges of a generating wire on a new cap face.
fn wire_pcurves<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    new_cell: &Shape,
    gen_s: &Shape,
    wire: &Shape,
    dir_s: &SweepNumShape,
) -> Result<(), Failure> {
    let c_gen_s = wire.oriented(Orientation::Forward);
    let mut jt = BRepSweepIterator::new();
    jt.init(&c_gen_s);
    while jt.more() {
        let (sub_sub_gen_s, pr) = (jt.value()?.clone(), jt.orientation()?);
        if s.has_shape(&sub_sub_gen_s, dir_s)? {
            let new_sub_edge = s.shape_at(&sub_sub_gen_s, dir_s)?;
            s.set_pcurve(new_cell, &new_sub_edge, (gen_s, &sub_sub_gen_s, dir_s), pr)?;
        }
        jt.next();
    }
    Ok(())
}

/// The containers of a wall under construction: the intermediate wire of a face (or
/// the independent wires), the intermediate shell of a solid.
#[derive(Default)]
struct WallParts {
    new_wire: Shape,
    wire_seq: Vec<Shape>,
    sepwires: bool,
    new_shell: Shape,
}

/// Makes the empty cell of a wall (the directing edge) and its intermediate
/// containers, and stores the cell.
fn make_wall_cell<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
    (i, j): (i32, i32),
    parts: &mut WallParts,
) -> Result<Shape, Failure> {
    let builder = s.sweep_base().my_builder;
    let mut new_cell = cell(s, i, j)?;
    match kind_of(gen_s)? {
        ShapeEnum::Vertex => new_cell = s.make_empty_directing_edge(gen_s, dir_s)?,
        ShapeEnum::Edge => {
            builder.make_wire(&mut parts.new_wire)?;
            new_cell = s.make_empty_face(gen_s, dir_s)?;
        }
        ShapeEnum::Wire => builder.make_shell(&mut new_cell)?,
        ShapeEnum::Face => {
            builder.make_shell(&mut parts.new_shell)?;
            builder.make_solid(&mut new_cell)?;
        }
        ShapeEnum::Shell => builder.make_comp_solid(&mut new_cell)?,
        ShapeEnum::Compound => builder.make_compound(&mut new_cell)?,
        kind => return Err(no_such_object(kind)),
    }
    s.sweep_base_mut().set_cell(i, j, new_cell.clone())?;
    Ok(new_cell)
}

/// The walls of the shape: the shapes swept along the directing edge, with the
/// shapes of the generating sub-shapes and of the directing vertices.
fn build_wall<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
    (i, j): (i32, i32),
) -> Result<(), Failure> {
    let mut parts = WallParts::default();
    let mut new_cell = make_wall_cell(s, gen_s, dir_s, (i, j), &mut parts)?;
    wall_generating_subs(s, &mut new_cell, gen_s, dir_s, (i, j), &mut parts)?;
    wall_directing_subs(s, &mut new_cell, gen_s, dir_s, i, &mut parts)?;
    finish_wall(s, &mut new_cell, gen_s, dir_s, (i, j), parts)
}

/// The new shapes of the generating sub-shapes along the directing edge.
fn wall_generating_subs<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    new_cell: &mut Shape,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
    (_, j): (i32, i32),
    parts: &mut WallParts,
) -> Result<(), Failure> {
    let b_gen_s = gen_s.oriented(Orientation::Forward);
    let mut it = BRepSweepIterator::new();
    it.init(&b_gen_s);
    while it.more() {
        let sub_gen_s = it.value()?.clone();
        let or = it.orientation()?;
        if s.has_shape(&sub_gen_s, dir_s)? {
            let new_shape = s.shape_at(&sub_gen_s, dir_s)?;
            let i_new = s.sweep_base().my_gen_shape_tool.index(&sub_gen_s);
            if s.ggd_shape_is_to_add(new_cell, &new_shape, (gen_s, &sub_gen_s, dir_s))? {
                let sub = FloorSub {
                    gen_s,
                    b_gen_s: &b_gen_s,
                    sub_gen_s: &sub_gen_s,
                    or,
                };
                wall_add_generating(s, new_cell, &new_shape, dir_s, (i_new, j), sub, parts)?;
            }
        }
        it.next();
    }
    Ok(())
}

/// Adds the new shape of one generating sub-shape to the wall, by the type of the
/// generating shape.
fn wall_add_generating<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    new_cell: &mut Shape,
    new_shape: &Shape,
    dir_s: &SweepNumShape,
    (i_new, j): (i32, i32),
    sub: FloorSub<'_>,
    parts: &mut WallParts,
) -> Result<(), Failure> {
    let builder = s.sweep_base().my_builder;
    let address = (sub.gen_s, sub.sub_gen_s, dir_s);
    match kind_of(sub.gen_s)? {
        ShapeEnum::Edge => {
            if s.separated_wires(new_cell, new_shape, address)? {
                parts.sepwires = true;
                let mut wi = Shape::default();
                builder.make_wire(&mut wi)?;
                builder.add_oriented(&mut wi, new_shape, sub.or)?;
                s.sweep_base_mut().set_used(i_new, j, true)?;
                wi.set_closed(BRepTool::is_closed(&wi)?)?;
                parts.wire_seq.push(wi);
            } else {
                builder.add_oriented(&mut parts.new_wire, new_shape, sub.or)?;
                s.sweep_base_mut().set_used(i_new, j, true)?;
            }
            return s.set_directing_pcurve(
                new_cell,
                new_shape,
                (sub.b_gen_s, sub.sub_gen_s, dir_s),
                sub.or,
            );
        }
        ShapeEnum::Face => return wall_face_add(s, new_shape, (i_new, j), sub, parts),
        ShapeEnum::Shell | ShapeEnum::Compound => {
            builder.add_oriented(new_cell, new_shape, Orientation::Forward)?
        }
        _ => builder.add_oriented(new_cell, new_shape, sub.or)?,
    }
    s.sweep_base_mut().set_used(i_new, j, true)
}

/// The side of a swept face: the faces of the shell of a wire (without marking it
/// used, as natively) or the face of an edge, into the intermediate shell.
fn wall_face_add<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    new_shape: &Shape,
    (i_new, j): (i32, i32),
    sub: FloorSub<'_>,
    parts: &mut WallParts,
) -> Result<(), Failure> {
    let builder = s.sweep_base().my_builder;
    match kind_of(sub.sub_gen_s)? {
        ShapeEnum::Wire => {
            let mut lt = BRepSweepIterator::new();
            lt.init(new_shape);
            while lt.more() {
                let composed = top_abs::compose(lt.orientation()?, sub.or);
                builder.add_oriented(&mut parts.new_shell, lt.value()?, composed)?;
                lt.next();
            }
            Ok(())
        }
        ShapeEnum::Edge => {
            builder.add_oriented(&mut parts.new_shell, new_shape, sub.or)?;
            s.sweep_base_mut().set_used(i_new, j, true)
        }
        _ => Ok(()),
    }
}

/// The new shapes of the directing vertices: the caps of a face, the vertices of a
/// directing edge, the generating edges of a side face.
fn wall_directing_subs<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    new_cell: &mut Shape,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
    i: i32,
    parts: &mut WallParts,
) -> Result<(), Failure> {
    let gen_kind = kind_of(gen_s)?;
    let mut kt = SweepNumShapeIterator::new();
    kt.init(dir_s);
    while kt.more() {
        let sub_dir_s = *kt.value();
        if s.has_shape(gen_s, &sub_dir_s)? {
            let new_shape = s.shape_at(gen_s, &sub_dir_s)?;
            let j_new = s.sweep_base().my_dir_shape_tool.index(&sub_dir_s);
            if s.gdd_shape_is_to_add(new_cell, &new_shape, gen_s, (dir_s, &sub_dir_s))? {
                let orientation = kt.orientation();
                let add = WallDirecting {
                    gen_s,
                    dir_s,
                    sub_dir_s: &sub_dir_s,
                    orientation,
                };
                wall_add_directing(s, new_cell, &new_shape, gen_kind, (i, j_new), add, parts)?;
            }
        }
        kt.next();
    }
    Ok(())
}

/// A directing vertex of a wall and its orientation in the directing edge.
struct WallDirecting<'a> {
    gen_s: &'a Shape,
    dir_s: &'a SweepNumShape,
    sub_dir_s: &'a SweepNumShape,
    orientation: Orientation,
}

/// Adds the new shape of one directing vertex to the wall, by the type of the
/// generating shape.
fn wall_add_directing<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    new_cell: &mut Shape,
    new_shape: &Shape,
    gen_kind: ShapeEnum,
    (i, j_new): (i32, i32),
    add: WallDirecting<'_>,
    parts: &mut WallParts,
) -> Result<(), Failure> {
    let builder = s.sweep_base().my_builder;
    match gen_kind {
        ShapeEnum::Edge => {
            let or = top_abs::reverse(add.orientation);
            builder.add_oriented(&mut parts.new_wire, new_shape, or)?;
            s.sweep_base_mut().set_used(i, j_new, true)?;
            s.set_generating_pcurve(
                new_cell,
                new_shape,
                add.gen_s,
                (add.dir_s, add.sub_dir_s),
                or,
            )
        }
        ShapeEnum::Vertex => {
            builder.add_oriented(new_cell, new_shape, add.orientation)?;
            s.sweep_base_mut().set_used(i, j_new, true)?;
            s.set_directing_parameter(new_cell, new_shape, add.gen_s, add.dir_s, add.sub_dir_s)
        }
        ShapeEnum::Face => {
            builder.add_oriented(&mut parts.new_shell, new_shape, add.orientation)?;
            s.sweep_base_mut().set_used(i, j_new, true)
        }
        _ => Ok(()),
    }
}

/// The end of a wall: the wires of a face (then its continuity), the continuity of a
/// shell of a wire, the shells of a solid oriented by `DirectSolid`.
fn finish_wall<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    new_cell: &mut Shape,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
    (i, j): (i32, i32),
    parts: WallParts,
) -> Result<(), Failure> {
    let builder = s.sweep_base().my_builder;
    match kind_of(gen_s)? {
        ShapeEnum::Edge => {
            if parts.sepwires {
                for wi in &parts.wire_seq {
                    builder.add(new_cell, wi)?;
                }
            } else {
                parts
                    .new_wire
                    .set_closed(BRepTool::is_closed(&parts.new_wire)?)?;
                builder.add(new_cell, &parts.new_wire)?;
            }
            s.sweep_base_mut().set_built(i, j, true)?;
            s.set_continuity(gen_s, dir_s)
        }
        ShapeEnum::Wire => s.set_continuity(gen_s, dir_s),
        ShapeEnum::Face => finish_solid(s, new_cell, gen_s, dir_s, &parts.new_shell),
        _ => Ok(()),
    }
}

/// The shells of a swept face: the split shells when there are several, else the
/// intermediate shell, all with the orientation of `DirectSolid`.
fn finish_solid<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    new_cell: &mut Shape,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
    new_shell: &Shape,
) -> Result<(), Failure> {
    let builder = s.sweep_base().my_builder;
    new_shell.set_closed(BRepTool::is_closed(new_shell)?)?;
    let temp = s.split_shell(new_shell)?;
    let shell_ori = s.direct_solid(gen_s, dir_s)?;
    let mut lt = BRepSweepIterator::new();
    lt.init(&temp);
    if lt.more() {
        lt.next();
    }
    if !lt.more() {
        return builder.add_oriented(new_cell, new_shell, shell_ori);
    }
    lt.init(&temp);
    while lt.more() {
        builder.add_oriented(new_cell, lt.value()?, shell_ori)?;
        lt.next();
    }
    Ok(())
}

/// The directing wire case: the container of the generating type and the shapes of
/// the directing sub-shapes.
fn build_directing_wire<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &mut T,
    gen_s: &Shape,
    dir_s: &SweepNumShape,
    (i, j): (i32, i32),
) -> Result<(), Failure> {
    let builder = s.sweep_base().my_builder;
    let mut new_cell = cell(s, i, j)?;
    match kind_of(gen_s)? {
        ShapeEnum::Vertex => builder.make_wire(&mut new_cell)?,
        ShapeEnum::Edge | ShapeEnum::Wire => builder.make_shell(&mut new_cell)?,
        ShapeEnum::Face | ShapeEnum::Shell => builder.make_comp_solid(&mut new_cell)?,
        ShapeEnum::Compound => builder.make_compound(&mut new_cell)?,
        kind => return Err(no_such_object(kind)),
    }
    s.sweep_base_mut().set_cell(i, j, new_cell.clone())?;
    let mut kt = SweepNumShapeIterator::new();
    kt.init(dir_s);
    while kt.more() {
        let sub_dir_s = *kt.value();
        if s.has_shape(gen_s, &sub_dir_s)? {
            let j_new = s.sweep_base().my_dir_shape_tool.index(&sub_dir_s);
            let new_shape = s.shape_at(gen_s, &sub_dir_s)?;
            builder.add_oriented(&mut new_cell, &new_shape, kt.orientation())?;
            s.sweep_base_mut().set_used(i, j_new, true)?;
        }
        kt.next();
    }
    Ok(())
}

/// `IsUsed(aGenS)`: a shape generated by `gen_s` at a directing index past 1 is used,
/// or it is built without a shape along the edge of an open sweep; an invariant vertex
/// counts only when its first shape is used or the sweep is open.
pub(crate) fn is_used<T: BRepSweepNumLinearRegularSweep + ?Sized>(
    s: &T,
    gen_s: &Shape,
) -> Result<bool, Failure> {
    let base = s.sweep_base();
    let i_gen = base.my_gen_shape_tool.index(gen_s);
    if i_gen == 0 {
        return Ok(false);
    }
    let (mut is_built, mut is_used) = (false, false);
    for j in 2..=base.my_built_shapes.upper_col() {
        is_built = is_built || base.built(i_gen, j)?;
        is_used = is_used || base.used(i_gen, j)?;
    }
    let is_vertex = gen_s.shape_type()? == ShapeEnum::Vertex;
    if is_used {
        if is_vertex && s.is_invariant(gen_s)? {
            return Ok(base.used(i_gen, 1)? || !s.closed());
        }
        return Ok(true);
    }
    if is_built {
        let dir_wire = base.my_dir_wire;
        if !s.has_shape(gen_s, &dir_wire)? && !s.closed() {
            return Ok(true);
        }
        if is_vertex && !s.closed() && !base.built(i_gen, 1)? {
            return Ok(true);
        }
    }
    Ok(is_used)
}
