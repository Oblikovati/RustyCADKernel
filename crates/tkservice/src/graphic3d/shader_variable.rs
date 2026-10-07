//! `Graphic3d_ShaderVariable` and `Graphic3d_ShaderAttribute`: the custom uniform variables
//! and vertex attributes of a shader program, with the typed uniform values
//! (`Graphic3d_ValueInterface`, `Graphic3d_UniformValue<T>`).
use rustycad_tkernel::ncollection::{Mat3, Mat4, Vec2, Vec3, Vec4};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::any::Any;

/// `Graphic3d_UniformValueTypeID<T>`: the type id of a uniform value type. Native quirk: each
/// id is the `__LINE__` of its specialisation in `Graphic3d_ShaderVariable.hxx`, so the
/// values below are those header lines. Example: `<f32 as Graphic3dUniformValueTypeId>::ID == 66`.
pub trait Graphic3dUniformValueTypeId: Clone + 'static {
    /// The type id (`ID`).
    const ID: usize;
}
macro_rules! uniform_type_ids {
    ($($ty:ty => $id:expr),* $(,)?) => {
        $(impl Graphic3dUniformValueTypeId for $ty { const ID: usize = $id; })*
    };
}
uniform_type_ids! {
    i32 => 60, f32 => 66, Vec2<f32> => 72, Vec3<f32> => 78, Vec4<f32> => 84,
    Vec2<i32> => 90, Vec3<i32> => 96, Vec4<i32> => 102, Mat3<f32> => 108, Mat4<f32> => 114,
}

/// `Graphic3d_ValueInterface`: a uniform value of any supported type.
pub trait Graphic3dValueInterface: Any + Send + Sync {
    /// The type id of the stored value; `value.type_id_of_value() == 66` for a float.
    fn type_id_of_value(&self) -> usize;
    /// The value as `Any`, for the typed accessors.
    fn as_any(&self) -> &dyn Any;
    /// The value as mutable `Any`, for the typed accessors.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
impl dyn Graphic3dValueInterface {
    /// `As<T>()`: the stored value when it has type `T`. The native `dynamic_cast` then
    /// dereferences a null pointer for a wrong type; here it is `None`.
    ///
    /// Example: `value.as_value::<f32>() == Some(&1.5)`.
    pub fn as_value<T: Graphic3dUniformValueTypeId>(&self) -> Option<&T> {
        self.as_any()
            .downcast_ref::<Graphic3dUniformValue<T>>()
            .map(|uniform| &uniform.value)
    }
    /// `As<T>()` on a mutable value; `*value.as_value_mut::<f32>().unwrap() = 2.0`.
    pub fn as_value_mut<T: Graphic3dUniformValueTypeId>(&mut self) -> Option<&mut T> {
        self.as_any_mut()
            .downcast_mut::<Graphic3dUniformValue<T>>()
            .map(|uniform| &mut uniform.value)
    }
}

/// `Graphic3d_UniformValue<T>`: one typed uniform value; `Graphic3dUniformValue::new(1.5f32)`.
#[derive(Clone, Debug, PartialEq)]
pub struct Graphic3dUniformValue<T> {
    /// The value (`Value`).
    pub value: T,
}
impl<T> Graphic3dUniformValue<T> {
    /// Wraps a value; `Graphic3dUniformValue::new(3i32).value == 3`.
    pub fn new(value: T) -> Self {
        Self { value }
    }
}
impl<T: Graphic3dUniformValueTypeId + Send + Sync> Graphic3dValueInterface
    for Graphic3dUniformValue<T>
{
    fn type_id_of_value(&self) -> usize {
        T::ID
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// `Graphic3d_UniformInt`.
pub type Graphic3dUniformInt = Graphic3dUniformValue<i32>;
/// `Graphic3d_UniformVec2i`.
pub type Graphic3dUniformVec2i = Graphic3dUniformValue<Vec2<i32>>;
/// `Graphic3d_UniformVec3i`.
pub type Graphic3dUniformVec3i = Graphic3dUniformValue<Vec3<i32>>;
/// `Graphic3d_UniformVec4i`.
pub type Graphic3dUniformVec4i = Graphic3dUniformValue<Vec4<i32>>;
/// `Graphic3d_UniformFloat`.
pub type Graphic3dUniformFloat = Graphic3dUniformValue<f32>;
/// `Graphic3d_UniformVec2`.
pub type Graphic3dUniformVec2 = Graphic3dUniformValue<Vec2<f32>>;
/// `Graphic3d_UniformVec3`.
pub type Graphic3dUniformVec3 = Graphic3dUniformValue<Vec3<f32>>;
/// `Graphic3d_UniformVec4`.
pub type Graphic3dUniformVec4 = Graphic3dUniformValue<Vec4<f32>>;
/// `Graphic3d_UniformMat3`.
pub type Graphic3dUniformMat3 = Graphic3dUniformValue<Mat3<f32>>;
/// `Graphic3d_UniformMat4`.
pub type Graphic3dUniformMat4 = Graphic3dUniformValue<Mat4<f32>>;

/// A named custom uniform variable of a shader program.
///
/// Example: `let variable = Graphic3dShaderVariable::create("occScale", 2.0f32);`.
pub struct Graphic3dShaderVariable {
    transient: StandardTransient,
    name: AsciiString,
    value: Option<Box<dyn Graphic3dValueInterface>>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dShaderVariable,
    "Graphic3d_ShaderVariable",
    StandardTransient,
    transient
);

impl Graphic3dShaderVariable {
    /// `Create<T>(name, value)`: a variable holding `value`;
    /// `Graphic3dShaderVariable::create("occScale", 2.0f32).is_done()`.
    pub fn create<T: Graphic3dUniformValueTypeId + Send + Sync>(
        name: impl Into<AsciiString>,
        value: T,
    ) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            name: name.into(),
            value: Some(Box::new(Graphic3dUniformValue::new(value))),
        })
    }
    /// The protected `Graphic3d_ShaderVariable(name)`: a variable without a value, not done;
    /// `Graphic3dShaderVariable::new("occScale").is_done() == false`.
    pub fn new(name: impl Into<AsciiString>) -> Self {
        Self {
            transient: StandardTransient::default(),
            name: name.into(),
            value: None,
        }
    }
    /// The variable name; `variable.name().as_bytes() == b"occScale"`.
    pub fn name(&self) -> &AsciiString {
        &self.name
    }
    /// Whether the variable has a name and a value; `variable.is_done()`.
    pub fn is_done(&self) -> bool {
        !self.name.is_empty() && self.value.is_some()
    }
    /// The value, `None` (native null) when unset; `variable.value().unwrap().as_value::<f32>()`.
    pub fn value(&self) -> Option<&dyn Graphic3dValueInterface> {
        self.value.as_deref()
    }
    /// `Value()`: the mutable value, `None` when unset; `variable.value_mut()`.
    pub fn value_mut(&mut self) -> Option<&mut (dyn Graphic3dValueInterface + 'static)> {
        self.value.as_deref_mut()
    }
}

/// A named vertex attribute bound to a location.
///
/// Example: `Graphic3dShaderAttribute::new("occColor", 3).location() == 3`.
pub struct Graphic3dShaderAttribute {
    transient: StandardTransient,
    name: AsciiString,
    location: i32,
}
rustycad_tkernel::impl_transient!(
    Graphic3dShaderAttribute,
    "Graphic3d_ShaderAttribute",
    StandardTransient,
    transient
);

impl Graphic3dShaderAttribute {
    /// An attribute `name` at `location`; `Graphic3dShaderAttribute::new("occColor", 3)`.
    pub fn new(name: impl Into<AsciiString>, location: i32) -> Self {
        Self {
            transient: StandardTransient::default(),
            name: name.into(),
            location,
        }
    }
    /// The attribute name; `attribute.name()`.
    pub fn name(&self) -> &AsciiString {
        &self.name
    }
    /// The attribute location; `attribute.location()`.
    pub fn location(&self) -> i32 {
        self.location
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_ids_are_the_native_header_lines() {
        assert_eq!(<i32 as Graphic3dUniformValueTypeId>::ID, 60);
        assert_eq!(<Mat4<f32> as Graphic3dUniformValueTypeId>::ID, 114);
        let value: Box<dyn Graphic3dValueInterface> =
            Box::new(Graphic3dUniformVec3::new(Vec3::new(1.0, 2.0, 3.0)));
        assert_eq!(value.type_id_of_value(), 78);
        assert_eq!(value.as_value::<Vec3<f32>>().map(|v| v.y), Some(2.0));
        assert!(value.as_value::<f32>().is_none());
    }

    #[test]
    fn variables_are_done_with_a_name_and_a_value() {
        let variable = Graphic3dShaderVariable::create("occScale", 2.5f32);
        assert!(variable.is_done());
        assert_eq!(
            variable.value().and_then(|v| v.as_value::<f32>().copied()),
            Some(2.5)
        );
        assert!(!Graphic3dShaderVariable::create("", 1i32).is_done());
        assert!(!Graphic3dShaderVariable::new("x").is_done());
    }

    #[test]
    fn attributes_keep_name_and_location() {
        let attribute = Graphic3dShaderAttribute::new("occColor", 3);
        assert_eq!(attribute.name().as_bytes(), b"occColor");
        assert_eq!(attribute.location(), 3);
    }
}
