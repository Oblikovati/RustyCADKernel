//! `BOPAlgo_MakePeriodic::RepeatShape` and `UpdateTwins`: the periodic shape repeated along
//! a periodic direction, with the twins of the repeated shape.
use super::identical::{compound_of, direction, translate};
use super::{BOPAlgoMakePeriodic, TwinMap};
use crate::bop_algo::{AlertUnableToRepeat, BOPAlgoBuilder, Glue};
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Vector3;
use std::sync::Arc;

impl BOPAlgoMakePeriodic {
    /// Adds `times` copies of the repeated shape translated by the current repeat period
    /// in the periodic `direction` (backward when negative), glued together; the history
    /// gets the translations as generated shapes and the gluing, and the twins follow. A
    /// failed gluing only warns (`RepeatShape`); `maker.repeat_shape(0, 2)?`.
    pub fn repeat_shape(&mut self, direction_id: i32, times: i32) -> Result<&Shape, Failure> {
        if self.repeated_shape.is_null() {
            self.repeated_shape = self.shape.clone();
        }
        if !self.is_periodic(direction_id) || times == 0 {
            return Ok(&self.repeated_shape);
        }
        let slot = Self::slot(direction_id);
        if self.repeat_period[slot] < precision::confusion() {
            self.repeat_period[slot] = self.period(slot as i32);
        }
        let (shapes, translation) = self.translated_copies(slot, times)?;
        if let Some(history) = self.history.get() {
            history.merge(&translation)?;
        }
        let mut gluer = BOPAlgoBuilder::new();
        gluer.set_arguments(&shapes.iter().cloned().collect::<List<Shape>>());
        gluer.set_glue(Glue::Full);
        gluer
            .options_mut()
            .set_run_parallel(self.options.run_parallel());
        gluer.perform();
        if gluer.options().has_errors() {
            let alert: AlertHandle = Arc::new(AlertUnableToRepeat::new(&compound_of(&shapes)?));
            self.options.add_warning(&alert);
            return Ok(&self.repeated_shape);
        }
        self.repeated_shape = gluer.shape().clone();
        self.repeat_period[slot] += f64::from(times.abs()) * self.repeat_period[slot];
        let gluing = History::from_algorithm(&shapes, &mut *gluer)?;
        if let Some(history) = self.history.get() {
            history.merge(&gluing)?;
        }
        self.update_twins(&translation, &gluing)?;
        Ok(&self.repeated_shape)
    }

    /// The repeated shape and its translated copies, with the history of each supported
    /// sub-shape generating itself and its translations.
    fn translated_copies(&self, slot: usize, times: i32) -> Result<(Vec<Shape>, History), Failure> {
        let period = self.repeat_period[slot];
        let sense = if times > 0 { 1 } else { -1 };
        let translation = History::new();
        let mut sub_shapes = IndexedMapOfShape::default();
        topexp::map_shapes(&self.repeated_shape, &mut sub_shapes, true, true);
        let supported = supported_shapes(&sub_shapes)?;
        for shape in &supported {
            translation.add_generated(shape, shape)?;
        }
        let mut shapes = vec![self.repeated_shape.clone()];
        for step in 1..=times.abs() {
            let distance = f64::from(sense * step) * period;
            let (translated, transform) = translate(
                &self.repeated_shape,
                Vector3::from(direction(slot)).scaled(distance),
            )?;
            shapes.push(translated);
            for shape in &supported {
                translation.add_generated(shape, &transform.modified_shape(shape)?)?;
            }
        }
        Ok((shapes, translation))
    }

    /// Each group of twins becomes the shapes generated from its members by the
    /// translations, through their gluing, all twins of each other (`UpdateTwins`).
    fn update_twins(&mut self, translation: &History, gluing: &History) -> Result<(), Failure> {
        if self.twins.is_empty() {
            return Ok(());
        }
        if self.repeated_twins.is_empty() {
            self.repeated_twins = self.twins.clone();
        }
        let mut new_twins = TwinMap::default();
        let mut done = MapOfShape::default();
        for (shape, twins) in self.repeated_twins.items() {
            done.add(shape.clone());
            if twins.iter().any(|twin| done.contains(twin)) {
                // The group of twins has already been processed.
                continue;
            }
            let group = repeated_group(shape, twins, translation, gluing)?;
            bind_group(&group, &mut new_twins)?;
        }
        self.repeated_twins = new_twins;
        Ok(())
    }
}

/// The supported sub-shapes of the map in its order.
fn supported_shapes(sub_shapes: &IndexedMapOfShape) -> Result<Vec<Shape>, Failure> {
    let mut supported = Vec::new();
    for shape in sub_shapes.iter() {
        if History::is_supported_type(shape)? {
            supported.push(shape.clone());
        }
    }
    Ok(supported)
}

/// The shapes generated from the shape and its twins by the translations, replaced by
/// their gluing images.
fn repeated_group(
    shape: &Shape,
    twins: &List<Shape>,
    translation: &History,
    gluing: &History,
) -> Result<IndexedMapOfShape, Failure> {
    let mut group = IndexedMapOfShape::default();
    for member in std::iter::once(shape).chain(twins.iter()) {
        for generated in translation.generated(member)? {
            let glued = gluing.modified(&generated)?;
            if glued.is_empty() {
                group.add(generated);
            } else {
                for image in glued {
                    group.add(image);
                }
            }
        }
    }
    Ok(group)
}

/// Makes every shape of the group the twin of all the others.
fn bind_group(group: &IndexedMapOfShape, twins: &mut TwinMap) -> Result<(), Failure> {
    for first in 1..=group.extent() {
        let list = twins.bound(group.find_key(first)?.clone(), List::default());
        for second in (1..=group.extent()).filter(|second| *second != first) {
            list.append(group.find_key(second)?.clone());
        }
    }
    Ok(())
}
