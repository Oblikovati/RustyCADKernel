//! Generated entity/owner/type palettes and native selection-mode lookup order.
use super::selection_image::SelectMgrSelectionImageFillerBase;
use super::*;
use crate::{
    prs_mgr::PrsMgrPresentableObjectMembers,
    select3d::{Select3DSensitiveEntity, Select3DSensitiveEntityMembers},
};
use rustycad_tkernel::{
    ncollection::{DataMap, HashKey},
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::Graphic3dPresentationAttributesMembers;

pub(super) enum SelectionImageColors {
    None,
    Entities(DataMap<Handle<dyn Select3DSensitiveEntity>, Color>),
    Types(DataMap<usize, Color>),
    Owners(DataMap<Handle<dyn SelectMgrEntityOwnerSource>, Color>),
    Modes(DataMap<i32, Color>),
}
impl SelectionImageColors {
    pub fn new<S: SelectMgrViewerSelectorSource + ?Sized>(
        kind: i32,
        base: &mut SelectMgrSelectionImageFillerBase<'_, S>,
    ) -> Result<Self, Failure> {
        let mut colors = match kind {
            4 => Self::Entities(DataMap::default()),
            5 => Self::Types(DataMap::default()),
            6 => Self::Owners(DataMap::default()),
            7 => Self::Modes(standard_mode_colors()),
            _ => return Ok(Self::None),
        };
        if kind == 7 {
            return Ok(colors);
        }
        colors.collect_objects(base)?;
        Ok(colors)
    }
    fn collect_objects<S: SelectMgrViewerSelectorSource + ?Sized>(
        &mut self,
        base: &mut SelectMgrSelectionImageFillerBase<'_, S>,
    ) -> Result<(), Failure> {
        let mut objects = SelectMgrSelectableObjectSetIterator::new(
            &base.selector.viewer_selector_base().selectable_objects,
        );
        while objects.more() {
            self.add_object(base, &objects.value()?)?;
            objects.next();
        }
        Ok(())
    }
    fn add_object<S: SelectMgrViewerSelectorSource + ?Sized>(
        &mut self,
        base: &mut SelectMgrSelectionImageFillerBase<'_, S>,
        object: &Handle<dyn SelectMgrSelectableObject>,
    ) -> Result<(), Failure> {
        // The native constructor includes inactive selections despite its activation comment.
        for selection in object.selections() {
            for sensitive in selection.entities() {
                self.add_entity(base, sensitive.base_sensitive())?;
            }
        }
        Ok(())
    }
    fn add_entity<S: SelectMgrViewerSelectorSource + ?Sized>(
        &mut self,
        base: &mut SelectMgrSelectionImageFillerBase<'_, S>,
        entity: Handle<dyn Select3DSensitiveEntity>,
    ) -> Result<(), Failure> {
        match self {
            Self::Entities(colors) => bind_image_color(colors, entity, base),
            Self::Types(colors) => bind_image_color(colors, entity_type_key(&entity)?, base),
            Self::Owners(colors) => bind_image_color(colors, entity.owner_id(), base),
            _ => Ok(()),
        }
    }
    pub fn picked_color<S: SelectMgrViewerSelectorSource + ?Sized>(
        &mut self,
        base: &mut SelectMgrSelectionImageFillerBase<'_, S>,
        rank: i32,
    ) -> Result<Color, Failure> {
        match self {
            Self::Owners(colors) => Ok(image_palette_color(colors, &base.selector.picked(rank)?)),
            Self::Modes(colors) => mode_color(colors, base, rank),
            Self::Entities(colors) => Ok(image_palette_color(
                colors,
                &*base.selector.picked_entity(rank)?,
            )),
            Self::Types(colors) => {
                let entity = (*base.selector.picked_entity(rank)?).clone();
                Ok(image_palette_color(colors, &entity_type_key(&entity)?))
            }
            Self::None => Ok(black()),
        }
    }
}
fn image_palette_color<K: HashKey + PartialEq + std::fmt::Debug>(
    colors: &DataMap<K, Color>,
    key: &K,
) -> Color {
    colors.find(key).copied().unwrap_or_else(|_| black())
}
fn bind_image_color<
    K: HashKey + PartialEq + std::fmt::Debug,
    S: SelectMgrViewerSelectorSource + ?Sized,
>(
    colors: &mut DataMap<K, Color>,
    key: K,
    base: &mut SelectMgrSelectionImageFillerBase<'_, S>,
) -> Result<(), Failure> {
    if colors.is_bound(&key) {
        return Ok(());
    }
    colors.bind(key, base.random_pastel_color()?);
    Ok(())
}
fn entity_type_key(entity: &Handle<dyn Select3DSensitiveEntity>) -> Result<usize, Failure> {
    let entity = entity.get().ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::null_object(
            "selection entity=null; expected a non-null entity",
        )
    })?;
    Ok(entity.dynamic_type() as *const _ as usize)
}
const STANDARD_IMAGE_MODE_NAMES: [NamedColor; 9] = [
    NamedColor::WHITE,
    NamedColor::YELLOW,
    NamedColor::GREEN,
    NamedColor::RED,
    NamedColor::BLUE1,
    NamedColor::CYAN1,
    NamedColor::PURPLE,
    NamedColor::MAGENTA1,
    NamedColor::BROWN,
];
fn standard_mode_colors() -> DataMap<i32, Color> {
    let mut colors = DataMap::default();
    for (mode, name) in STANDARD_IMAGE_MODE_NAMES.into_iter().enumerate() {
        colors.bind(mode as i32, Color::named(name));
    }
    for (mode, name) in [
        (0x10, NamedColor::PINK),
        (0x1e, NamedColor::LIMEGREEN),
        (0x1f, NamedColor::DARKOLIVEGREEN),
        (0x100, NamedColor::GOLD),
    ] {
        colors.bind(mode, Color::named(name));
    }
    colors
}
fn mode_color<S: SelectMgrViewerSelectorSource + ?Sized>(
    colors: &mut DataMap<i32, Color>,
    base: &mut SelectMgrSelectionImageFillerBase<'_, S>,
    rank: i32,
) -> Result<Color, Failure> {
    let mode = picked_selection_mode(base.selector, rank)?;
    if mode == -1 {
        return Ok(black());
    }
    bind_image_color(colors, mode, base)?;
    colors.find(&mode).copied()
}
fn picked_selection_mode<S: SelectMgrViewerSelectorSource + ?Sized>(
    selector: &S,
    rank: i32,
) -> Result<i32, Failure> {
    let selectable = selector.picked(rank)?.selectable();
    let entity = (*selector.picked_entity(rank)?).clone();
    let mut mode = -1;
    for selection in selectable.selections() {
        if selection
            .entities()
            .iter()
            .any(|value| value.base_sensitive() == entity)
        {
            mode = selection.mode();
        }
    }
    Ok(mode)
}
pub(super) fn black() -> Color {
    Color::named(NamedColor::BLACK)
}
pub(super) fn detected_object_color<S: SelectMgrViewerSelectorSource + ?Sized>(
    selector: &S,
    rank: i32,
) -> Result<Color, Failure> {
    let object = selector.picked(rank)?.selectable();
    Ok(object.attributes().color())
}
