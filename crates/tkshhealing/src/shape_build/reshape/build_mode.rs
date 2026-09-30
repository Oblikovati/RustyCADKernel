// Rust translation of ShapeBuild_ReShape::Apply(shape, until, buildmode) of
// ShapeBuild_ReShape.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871 (abv 28.04.99 S4137).

//! The build-mode `Apply`: compounds, solids and shells rebuilt from their replaced items.
use super::ShapeBuildReShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator, Shell, Solid};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The two results a solid or a shell is rebuilt into: a compound of every item and the
/// container of the items of the right type, with the modification flag.
struct Rebuilt {
    compound: Shape,
    container: Shape,
    modif: i32,
}

impl ShapeBuildReShape {
    /// `Apply(shape, until, buildmode)`: a shape with a recorded status gives its new shape
    /// (null when removed); a compound or composite solid becomes a compound of its items'
    /// new shapes (the removed ones dropped; the shape itself when nothing changed); a
    /// solid or a shell is rebuilt from its items applied recursively. `build_mode` says
    /// what a changed solid or shell gives: 0, a compound when an item was replaced or
    /// removed; 1, a compound when one was removed; 2, always the rebuilt solid or shell.
    /// Types below the shell are not handled: the shape is returned after the native
    /// "BRepTools_ReShape::Apply NOT YET IMPLEMENTED" line on the standard output;
    /// `reshape.apply_with_build_mode(&shape, ShapeEnum::Face, 0)?`.
    pub fn apply_with_build_mode(
        &self,
        shape: &Shape,
        until: ShapeEnum,
        build_mode: i32,
    ) -> Result<Shape, Failure> {
        if shape.is_null() {
            return Ok(shape.clone());
        }
        let (status, new_shape) = self.status(shape, false)?;
        if status != 0 {
            return Ok(new_shape);
        }
        let kind = shape.shape_type()?;
        if kind == until {
            return Ok(new_shape); // critere d arret
        }
        match kind {
            ShapeEnum::Compound | ShapeEnum::CompSolid => self.apply_compound(shape),
            ShapeEnum::Solid => self.apply_container(shape, until, build_mode, ShapeEnum::Shell),
            ShapeEnum::Shell => self.apply_container(shape, until, build_mode, ShapeEnum::Face),
            _ => {
                println!("BRepTools_ReShape::Apply NOT YET IMPLEMENTED");
                Ok(shape.clone())
            }
        }
    }

    /// The compound branch: each item's recorded new shape (removed items dropped).
    fn apply_compound(&self, shape: &Shape) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let mut modif = false;
        let mut it = ShapeIterator::new(shape, true, true);
        while it.more() {
            let (status, new_shape) = self.status(it.value()?, false)?;
            it.next();
            modif |= status != 0;
            if status >= 0 {
                builder.add(&mut compound, &new_shape)?;
            }
        }
        Ok(if modif {
            compound.into_shape()
        } else {
            shape.clone()
        })
    }

    /// The solid and shell branches: each item applied recursively; an item of another
    /// type contributes its sub-shapes of type `part` (none marks a removal).
    fn apply_container(
        &self,
        shape: &Shape,
        until: ShapeEnum,
        build_mode: i32,
        part: ShapeEnum,
    ) -> Result<Shape, Failure> {
        let rebuilt = self.rebuild_container(shape, until, build_mode, part)?;
        let modif = rebuilt.modif;
        if (modif < 0 && build_mode < 2) || (modif == 0 && build_mode < 1) {
            return Ok(rebuilt.compound);
        }
        if part == ShapeEnum::Face {
            let closed = BRepTool::is_closed(&rebuilt.container)?;
            rebuilt.container.set_closed(closed)?;
        }
        Ok(rebuilt.container)
    }

    fn rebuild_container(
        &self,
        shape: &Shape,
        until: ShapeEnum,
        build_mode: i32,
        part: ShapeEnum,
    ) -> Result<Rebuilt, Failure> {
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let container = if part == ShapeEnum::Shell {
            let mut solid = Solid::default();
            builder.make_solid(&mut solid);
            solid.into_shape()
        } else {
            let mut shell = Shell::default();
            builder.make_shell(&mut shell);
            shell.into_shape()
        };
        let mut rebuilt = Rebuilt {
            compound: compound.into_shape(),
            container,
            modif: 0,
        };
        let mut it = ShapeIterator::new(shape, true, true);
        while it.more() {
            let sub = it.value()?.clone();
            it.next();
            let new_shape = self.apply_with_build_mode(&sub, until, build_mode)?;
            add_item(&mut rebuilt, &sub, &new_shape, part)?;
        }
        Ok(rebuilt)
    }
}

/// One item of a solid or a shell: its new shape goes to the compound; to the container
/// itself when of type `part`, else its sub-shapes of that type.
fn add_item(
    rebuilt: &mut Rebuilt,
    sub: &Shape,
    new_shape: &Shape,
    part: ShapeEnum,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    if new_shape.is_null() {
        rebuilt.modif = -1;
        return Ok(());
    }
    if new_shape.shape_type()? != part {
        let mut nb_sub = 0;
        let mut explorer = Explorer::new(new_shape, part, ShapeEnum::Shape)?;
        while explorer.more() {
            builder.add(&mut rebuilt.container, explorer.current())?;
            nb_sub += 1;
            explorer.next()?;
        }
        if nb_sub == 0 {
            rebuilt.modif = -1;
        }
        return builder.add(&mut rebuilt.compound, new_shape); // c est tout
    }
    if rebuilt.modif == 0 && !sub.is_equal(new_shape) {
        rebuilt.modif = 1;
    }
    builder.add(&mut rebuilt.compound, new_shape)?;
    builder.add(&mut rebuilt.container, new_shape)
}
