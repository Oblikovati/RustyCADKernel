//! The `RecordModification` overloads of ShapeProcess_ShapeContext and their static
//! recursions (`RecModif` for a replacement map and for a ShapeBuild_ReShape,
//! `ExplodeModifier` for a BRepTools_Modifier).
use super::{ShapeProcessShapeContext, ShapeProcessShapeMap};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_extend::{BasicMsgRegistrator, ShapeExtendMsgRegistrator};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{Modifier, ReShape};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::message::Gravity;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;

/// The state the recursions share: the context map and registrator, the incoming
/// registrator and the detalisation level.
struct Recorder<'a> {
    map: &'a mut ShapeProcessShapeMap,
    own: &'a Handle<ShapeExtendMsgRegistrator>,
    incoming: &'a Handle<ShapeExtendMsgRegistrator>,
    until: ShapeEnum,
}

impl ShapeProcessShapeContext {
    /// `RecordModification(repl, msg)`: the replacement map applied down to the
    /// detalisation level (containers rebuilt from replaced children), with the messages
    /// of `msg` moved to the original shapes; the result follows the shape.
    /// Example: `context.record_modification_map(&replacements, &Handle::default())?`.
    pub fn record_modification_map(
        &self,
        replacements: &ShapeProcessShapeMap,
        messages: &Handle<ShapeExtendMsgRegistrator>,
    ) -> Result<(), Failure> {
        if replacements.extent() == 0 {
            return Ok(());
        }
        let mut fields = self.fields.write();
        let (shape, own, until) = (fields.shape.clone(), fields.messages.clone(), fields.until);
        let mut recorder = Recorder {
            map: &mut fields.map,
            own: &own,
            incoming: messages,
            until,
        };
        recorder.map_modification(&shape, replacements)?;
        if let Some(result) = fields.map.seek(&shape).cloned() {
            fields.result = result;
        }
        Ok(())
    }

    /// `RecordModification(repl, msg)` for a reshape: the recorded replacements (or the
    /// reshape applied to rebuilt containers) down to the detalisation level, the messages
    /// followed along the replacement chains; the result keeps the shape location.
    /// Example: `context.record_modification_reshape(&reshape, &Handle::default())?`.
    pub fn record_modification_reshape(
        &self,
        reshape: &Handle<ShapeBuildReShape>,
        messages: &Handle<ShapeExtendMsgRegistrator>,
    ) -> Result<(), Failure> {
        let reshape = reshape.get().ok_or_else(null_reshape)?;
        let mut fields = self.fields.write();
        let (shape, own, until) = (fields.shape.clone(), fields.messages.clone(), fields.until);
        let mut recorder = Recorder {
            map: &mut fields.map,
            own: &own,
            incoming: messages,
            until,
        };
        recorder.reshape_modification(&shape, reshape)?;
        if let Some(mut result) = fields.map.seek(&shape).cloned() {
            result.set_location(shape.location(), false)?;
            fields.result = result;
        }
        Ok(())
    }

    /// `RecordModification(repl)`: the reshape overload without messages.
    pub fn record_modification_reshape_only(
        &self,
        reshape: &Handle<ShapeBuildReShape>,
    ) -> Result<(), Failure> {
        self.record_modification_reshape(reshape, &Handle::default())
    }

    /// `RecordModification(S, repl, msg)`: the modified shapes of `modifier` from `shape`
    /// down to the detalisation level, recorded as a replacement map.
    /// Example: `context.record_modification_modifier(&shape, &modifier, &Handle::default())?`.
    pub fn record_modification_modifier(
        &self,
        shape: &Shape,
        modifier: &Modifier,
        messages: &Handle<ShapeExtendMsgRegistrator>,
    ) -> Result<(), Failure> {
        let mut map = ShapeProcessShapeMap::default();
        explode_modifier(shape, modifier, &mut map, self.get_detalisation())?;
        self.record_modification_map(&map, messages)
    }
}

fn null_reshape() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeProcess_ShapeContext::RecordModification: null ShapeBuild_ReShape",
        "",
    )
}

fn stops_at(shape: &Shape, until: ShapeEnum) -> Result<bool, Failure> {
    Ok(until == ShapeEnum::Shape || shape.shape_type()? >= until)
}

/// `ExplodeModifier`: every modified (sub)shape bound to its image.
fn explode_modifier(
    shape: &Shape,
    modifier: &Modifier,
    map: &mut ShapeProcessShapeMap,
    until: ShapeEnum,
) -> Result<(), Failure> {
    let result = modifier.modified_shape(shape)?;
    if result != *shape {
        map.bind(shape.clone(), result);
    }
    if stops_at(shape, until)? {
        return Ok(());
    }
    for child in ShapeIterator::new(shape, true, true) {
        explode_modifier(&child, modifier, map, until)?;
    }
    Ok(())
}

impl Recorder<'_> {
    fn current_image(&self, shape: &Shape) -> Result<Shape, Failure> {
        let unlocated = shape.located(&Location::default(), false)?;
        Ok(self.map.seek(&unlocated).cloned().unwrap_or(unlocated))
    }

    // The static RecModif of the replacement-map overload.
    fn map_modification(
        &mut self,
        shape: &Shape,
        repl: &ShapeProcessShapeMap,
    ) -> Result<(), Failure> {
        let current = self.current_image(shape)?;
        if !current.is_null() {
            let image = map_image(shape, &current, repl)?;
            if image != current {
                self.map
                    .bind(shape.located(&Location::default(), false)?, image);
            }
            self.move_messages_direct(shape, &current);
        }
        if stops_at(shape, self.until)? {
            return Ok(());
        }
        for child in ShapeIterator::new(shape, true, true) {
            self.map_modification(&child, repl)?;
        }
        Ok(())
    }

    fn move_messages_direct(&self, shape: &Shape, current: &Shape) {
        let (Some(own), Some(incoming)) = (self.own.get(), self.incoming.get()) else {
            return;
        };
        let list = incoming.map_shape(|map| {
            if map.extent() == 0 {
                return None;
            }
            map.seek(current).or_else(|| map.seek(shape)).cloned()
        });
        for message in list.iter().flat_map(|messages| messages.iter()) {
            own.send_shape(shape, message, Gravity::Warning);
        }
    }

    // The static RecModif of the reshape overload.
    fn reshape_modification(
        &mut self,
        shape: &Shape,
        reshape: &ShapeBuildReShape,
    ) -> Result<(), Failure> {
        if shape.is_null() {
            return Ok(());
        }
        let unlocated = shape.located(&Location::default(), false)?;
        let current = self.current_image(shape)?;
        if !current.is_null() {
            let (status, image) = reshape.status(&current, true)?;
            if status != 0 && image != current {
                self.map.bind(unlocated, image);
            } else if current.shape_type()? < shape.shape_type()? {
                let until = ShapeEnum::ALL[shape.shape_type()? as usize + 1];
                let image = ReShape::apply(reshape, &current, until)?;
                if image != current {
                    self.map.bind(unlocated, image);
                }
            }
            self.follow_messages(shape, &current, reshape)?;
        }
        if stops_at(shape, self.until)? {
            return Ok(());
        }
        for child in ShapeIterator::new(shape, false, true) {
            self.reshape_modification(&child, reshape)?;
        }
        Ok(())
    }

    // The messages of the shape (or of its current image) and of every later image in
    // the replacement chain are moved to the shape.
    fn follow_messages(
        &self,
        shape: &Shape,
        current: &Shape,
        reshape: &ShapeBuildReShape,
    ) -> Result<(), Failure> {
        let (Some(own), Some(incoming)) = (self.own.get(), self.incoming.get()) else {
            return Ok(());
        };
        if incoming.map_shape(|map| map.extent()) == 0 {
            return Ok(());
        }
        let mut next = if incoming.map_shape(|map| map.is_bound(shape)) {
            shape.clone()
        } else {
            current.clone()
        };
        loop {
            let present = next.clone();
            let list = incoming.map_shape(|map| map.seek(&present).cloned());
            for message in list.iter().flat_map(|messages| messages.iter()) {
                own.send_shape(shape, message, Gravity::Warning);
            }
            next = reshape.value(&present)?;
            if next.is_null() || present == next {
                return Ok(());
            }
        }
    }
}

/// The image of `current` (the image of `shape` so far) under the replacement map: a
/// replaced shape (reversed unless `current` is forward), or a container rebuilt from
/// its replaced children.
fn map_image(
    shape: &Shape,
    current: &Shape,
    repl: &ShapeProcessShapeMap,
) -> Result<Shape, Failure> {
    let located = current.located(shape.location(), false)?;
    if let Some(found) = repl.seek(&located) {
        let mut image = found.clone();
        if current.orientation() != Orientation::Forward {
            image.reverse();
        }
        return Ok(image);
    }
    if current.shape_type()? >= shape.shape_type()? {
        return Ok(current.clone());
    }
    rebuilt_container(current, repl)
}

fn rebuilt_container(current: &Shape, repl: &ShapeProcessShapeMap) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut result = current.empty_copied()?;
    // "protect against INTERNAL or EXTERNAL shapes".
    result.set_orientation(Orientation::Forward);
    let mut modified = false;
    for child in ShapeIterator::new(current, false, true) {
        match repl.seek(&child) {
            Some(replacement) => {
                if !replacement.is_null() {
                    builder.add(&mut result, replacement)?;
                }
                modified = true;
            }
            None => builder.add(&mut result, &child)?,
        }
    }
    if !modified {
        return Ok(current.clone());
    }
    if matches!(result.shape_type()?, ShapeEnum::Wire | ShapeEnum::Shell) {
        result.set_closed(BRepTool::is_closed(&result)?)?;
    }
    Ok(result)
}
