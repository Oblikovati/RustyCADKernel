// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_Identifier.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_Identifier`: decides how a shape can be named (by its primitive, by a
//! generation, by an intersection of named ancestors).
use super::named_shape::used_shapes_of;
use super::{
    TNamingEvolution, TNamingIterator, TNamingLocalizer, TNamingNameType, TNamingNamedShape,
    TNamingNamedShapeMap, TNamingTool,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::{Failure, Handle, exceptions};
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfChildIterator, TdfLabel};
use std::collections::VecDeque;

/// `TNaming_Identifier`. The native constructors leave the name type uninitialized until an
/// identification sets it; the port starts at `UNKNOWN`.
pub struct TNamingIdentifier {
    access: TdfLabel,
    shape: Shape,
    done: bool,
    is_feature: bool,
    name_type: TNamingNameType,
    primitive_args: VecDeque<Handle<TNamingNamedShape>>,
    shape_args: VecDeque<Shape>,
    context_named: Handle<TNamingNamedShape>,
}

impl TNamingIdentifier {
    /// `TNaming_Identifier(Lab, S, Context, Geom)`: identifies `S` (the last flag is unused).
    pub fn new(
        label: &TdfLabel,
        shape: &Shape,
        context: &Shape,
        _geometry: bool,
    ) -> Result<Self, Failure> {
        let mut identifier = Self::empty(label, shape);
        identifier.init(context)?;
        Ok(identifier)
    }
    /// `TNaming_Identifier(Lab, S, ContextNS, Geom)`: the context is the shape of `ContextNS`.
    pub fn with_context_named_shape(
        label: &TdfLabel,
        shape: &Shape,
        context: &Handle<TNamingNamedShape>,
        _geometry: bool,
    ) -> Result<Self, Failure> {
        let mut identifier = Self::empty(label, shape);
        identifier.init(&TNamingTool::get_shape(context)?)?;
        Ok(identifier)
    }
    fn empty(label: &TdfLabel, shape: &Shape) -> Self {
        Self {
            access: label.clone(),
            shape: shape.clone(),
            done: false,
            is_feature: false,
            name_type: TNamingNameType::Unknown,
            primitive_args: VecDeque::new(),
            shape_args: VecDeque::new(),
            context_named: Handle::default(),
        }
    }
    /// The private `Init(Context)`: by the evolution of the named shape that made the
    /// shape, or by its ancestors in the context when none did.
    fn init(&mut self, context: &Shape) -> Result<(), Failure> {
        let used = used_shapes_of(&self.access)?;
        let mut localizer = TNamingLocalizer::new();
        localizer.init(&used, self.access.transaction()?);
        let named = TNamingTool::named_shape(&self.shape, &self.access)?;
        let Some(named_ref) = named.get() else {
            return self.ancestor_identification(&mut localizer, context);
        };
        match named_ref.evolution() {
            TNamingEvolution::Primitive => self.primitive_identification(&mut localizer, &named),
            TNamingEvolution::Generated => self.generated_identification(&mut localizer, &named),
            _ => self.identification(&mut localizer, &named),
        }
    }
    /// `IsDone`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `Type`.
    pub fn name_type(&self) -> TNamingNameType {
        self.name_type
    }
    /// `IsFeature`.
    pub fn is_feature(&self) -> bool {
        self.is_feature
    }
    /// `Feature`: the first primitive argument (`Standard_NoSuchObject` when none).
    pub fn feature(&self) -> Result<Handle<TNamingNamedShape>, Failure> {
        first(&self.primitive_args, "TNaming_Identifier::Feature")
    }
    /// `InitArgs`: does nothing.
    pub fn init_args(&mut self) {}
    /// `MoreArgs`.
    pub fn more_args(&self) -> bool {
        !self.primitive_args.is_empty() || !self.shape_args.is_empty()
    }
    /// `NextArg`: drops the first primitive argument, else the first shape argument.
    pub fn next_arg(&mut self) {
        if self.primitive_args.pop_front().is_none() {
            self.shape_args.pop_front();
        }
    }
    /// `ArgIsFeature`.
    pub fn arg_is_feature(&self) -> bool {
        !self.primitive_args.is_empty()
    }
    /// `FeatureArg`.
    pub fn feature_arg(&self) -> Result<Handle<TNamingNamedShape>, Failure> {
        first(&self.primitive_args, "TNaming_Identifier::FeatureArg")
    }
    /// `ShapeArg`.
    pub fn shape_arg(&self) -> Result<Shape, Failure> {
        first(&self.shape_args, "TNaming_Identifier::ShapeArg")
    }
    /// `ShapeContext`: the context of the first shape argument
    /// (`TNaming_Localizer::FindShapeContext`, szy).
    pub fn shape_context(&self) -> Result<Shape, Failure> {
        let shape = first(&self.shape_args, "TNaming_Identifier::ShapeContext")?;
        let named = TNamingTool::named_shape(&shape, &self.access)?;
        let mut context = Shape::default();
        TNamingLocalizer::find_shape_context(&named, &shape, &mut context)?;
        Ok(context)
    }
    /// `NamedShapeOfGeneration`.
    pub fn named_shape_of_generation(&self) -> Handle<TNamingNamedShape> {
        self.context_named.clone()
    }
    /// `AncestorIdentification(Localizer, Context)`: an intersection of the named ancestors
    /// of the shape in the context.
    pub fn ancestor_identification(
        &mut self,
        localizer: &mut TNamingLocalizer,
        context: &Shape,
    ) -> Result<(), Failure> {
        if context.is_null() {
            self.name_type = TNamingNameType::Unknown;
            self.done = false;
            return Ok(());
        }
        let mut ancestors = MapOfShape::default();
        localizer.find_features_in_ancestors(&self.shape, context, &mut ancestors)?;
        if ancestors.is_empty() {
            self.done = false;
            return Ok(());
        }
        self.name_type = TNamingNameType::Intersection;
        self.shape_args.extend(ancestors.iter().cloned());
        self.done = true;
        Ok(())
    }
    /// `PrimitiveIdentification(L, NS)`: modified until `NS`, or a constant shape when `NS`
    /// is imported.
    pub fn primitive_identification(
        &mut self,
        _localizer: &mut TNamingLocalizer,
        named: &Handle<TNamingNamedShape>,
    ) -> Result<(), Failure> {
        self.primitive_args.push_back(named.clone());
        self.is_feature = true;
        self.name_type = TNamingNameType::ModifUntil;
        // A dead shape.
        if is_imported(named)? {
            self.name_type = TNamingNameType::ConstShape;
            self.is_feature = false;
        }
        self.done = true;
        Ok(())
    }
    /// `GeneratedIdentification(Localizer, NS)`: a generation from the generators of the
    /// shape in `NS` (szy 16.10.03: all of them).
    pub fn generated_identification(
        &mut self,
        _localizer: &mut TNamingLocalizer,
        named: &Handle<TNamingNamedShape>,
    ) -> Result<(), Failure> {
        let mut generators = Vec::new();
        TNamingLocalizer::find_generator(named, &self.shape, &mut generators)?;
        self.shape_args.extend(generators);
        self.context_named = named.clone();
        self.name_type = TNamingNameType::Generation;
        self.done = true;
        Ok(())
    }
    /// `Identification(Localizer, NS)`: an intersection of the primitives and generated
    /// shapes the shape goes back to; one primitive alone is a `MODIFUNTIL` (or a constant
    /// shape when it is imported).
    pub fn identification(
        &mut self,
        localizer: &mut TNamingLocalizer,
        named: &Handle<TNamingNamedShape>,
    ) -> Result<(), Failure> {
        let mut primitives = TNamingNamedShapeMap::default();
        let mut shapes = MapOfShape::default();
        localizer.backward(named, &self.shape, &mut primitives, &mut shapes)?;
        if primitives.is_empty() && shapes.is_empty() {
            self.done = false;
            return Ok(());
        }
        self.name_type = TNamingNameType::Intersection;
        self.primitive_args.extend(primitives.iter().cloned());
        self.shape_args.extend(shapes.iter().cloned());
        if self.primitive_args.len() == 1 && self.shape_args.is_empty() {
            self.name_type = TNamingNameType::ModifUntil;
            self.is_feature = true;
            // A dead shape.
            if is_imported(&self.primitive_args[0])? {
                self.name_type = TNamingNameType::ConstShape;
                self.is_feature = false;
            }
        }
        self.done = true;
        Ok(())
    }
}

/// `NCollection_List::First` of an argument list: `Standard_NoSuchObject` when it is empty.
fn first<T: Clone>(list: &VecDeque<T>, who: &str) -> Result<T, Failure> {
    list.front().cloned().ok_or_else(|| {
        exceptions::no_such_object(&format!(
            "{who}: the argument list is empty; expected MoreArgs()"
        ))
    })
}

/// The global `IsImported(NS)` of TNaming_Identifier.cxx: the label is the only child of
/// its father and the named shape has at least two evolutions.
pub(crate) fn is_imported(named: &Handle<TNamingNamedShape>) -> Result<bool, Failure> {
    let label = live(named, "IsImported")?.label();
    let mut children = TdfChildIterator::new(&label.father(), false)?;
    while children.more() {
        if children.value() != label {
            return Ok(false);
        }
        children.next();
    }
    let mut iterator = TNamingIterator::new(named)?;
    if !iterator.more() {
        return Ok(false);
    }
    iterator.next()?;
    Ok(iterator.more())
}
