//! The non-virtual members of `Graphic3d_Buffer`: layout queries, typed element access and
//! `Init`.
use super::{
    Graphic3dAttribute, Graphic3dBufferSource, Graphic3dBufferState, Graphic3dTypeOfAttribute,
};
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use rustycad_tkernel::standard::Failure;

/// A plain value stored in buffer bytes in native (little-endian x86-64) layout.
pub trait Graphic3dBufferValue: Copy {
    /// The byte size of the value.
    const SIZE: usize;
    /// Reads the value from `bytes` (at least `SIZE` long).
    fn read(bytes: &[u8]) -> Self;
    /// Writes the value into `bytes` (at least `SIZE` long).
    fn write(self, bytes: &mut [u8]);
}
macro_rules! scalar_value {
    ($($ty:ty),*) => {$(
        impl Graphic3dBufferValue for $ty {
            const SIZE: usize = std::mem::size_of::<$ty>();
            fn read(bytes: &[u8]) -> Self {
                <$ty>::from_le_bytes(bytes[..Self::SIZE].try_into().unwrap_or_default())
            }
            fn write(self, bytes: &mut [u8]) {
                bytes[..Self::SIZE].copy_from_slice(&self.to_le_bytes());
            }
        }
    )*};
}
scalar_value!(u8, u16, u32, i32, f32);

/// Reads `N` consecutive scalars.
fn read_array<T: Graphic3dBufferValue, const N: usize>(bytes: &[u8]) -> [T; N] {
    std::array::from_fn(|i| T::read(&bytes[i * T::SIZE..]))
}
/// Writes `N` consecutive scalars.
fn write_array<T: Graphic3dBufferValue, const N: usize>(values: [T; N], bytes: &mut [u8]) {
    values
        .iter()
        .enumerate()
        .for_each(|(i, v)| v.write(&mut bytes[i * T::SIZE..]));
}
macro_rules! vector_value {
    ($vec:ident, $n:literal, [$($c:ident),*]) => {
        impl<T: Graphic3dBufferValue> Graphic3dBufferValue for $vec<T> {
            const SIZE: usize = T::SIZE * $n;
            fn read(bytes: &[u8]) -> Self {
                let [$($c),*] = read_array::<T, $n>(bytes);
                $vec::new($($c),*)
            }
            fn write(self, bytes: &mut [u8]) {
                write_array([$(self.$c),*], bytes);
            }
        }
    };
}
vector_value!(Vec2, 2, [x, y]);
vector_value!(Vec3, 3, [x, y, z]);
vector_value!(Vec4, 4, [x, y, z, w]);

/// The `AttributeData` walk over a buffer state.
fn attribute_data_of(
    state: &Graphic3dBufferState,
    interleaved: bool,
    attrib: Graphic3dTypeOfAttribute,
) -> Option<(i32, usize, usize)> {
    let max_elements = nb_max_elements_of(state) as usize;
    let mut offset = 0usize;
    for (index, attribute) in state
        .attributes
        .iter()
        .take(state.nb_attributes.max(0) as usize)
        .enumerate()
    {
        let attribute_stride = attribute.stride() as usize;
        if attribute.id == attrib {
            let stride = if interleaved {
                state.stride as usize
            } else {
                attribute_stride
            };
            return Some((index as i32, offset, stride));
        }
        offset += if interleaved {
            attribute_stride
        } else {
            attribute_stride * max_elements
        };
    }
    None
}

fn nb_max_elements_of(state: &Graphic3dBufferState) -> i32 {
    if state.stride == 0 {
        return 0;
    }
    (state.data.size() / state.stride as usize) as i32
}

/// The non-virtual members of `Graphic3d_Buffer`.
pub trait Graphic3dBufferMembers: Graphic3dBufferSource {
    /// The allocated element count, `size / Stride`; `buffer.nb_max_elements()`.
    fn nb_max_elements(&self) -> i32 {
        nb_max_elements_of(&self.buffer_base().state.read())
    }
    /// The attribute at an index; `None` past the stored table (undefined natively).
    /// Example: `buffer.attribute(0)`.
    fn attribute(&self, index: i32) -> Option<Graphic3dAttribute> {
        let state = self.buffer_base().state.read();
        usize::try_from(index)
            .ok()
            .and_then(|slot| state.attributes.get(slot).copied())
    }
    /// `AttributesArray`: the stored attribute table; `buffer.attributes_array()`.
    fn attributes_array(&self) -> Vec<Graphic3dAttribute> {
        self.buffer_base().state.read().attributes.clone()
    }
    /// `ChangeAttribute`: replaces a stored attribute; false past the table. `buffer.set_attribute(0, attribute)`.
    fn set_attribute(&self, index: i32, attribute: Graphic3dAttribute) -> bool {
        let mut state = self.buffer_base().state.write();
        let slot = usize::try_from(index)
            .ok()
            .and_then(|slot| state.attributes.get_mut(slot));
        slot.map(|stored| *stored = attribute).is_some()
    }
    /// The index of the first attribute of a role, or -1; `buffer.find_attribute(Graphic3dTypeOfAttribute::Norm)`.
    fn find_attribute(&self, attrib: Graphic3dTypeOfAttribute) -> i32 {
        let state = self.buffer_base().state.read();
        let count = state.nb_attributes.max(0) as usize;
        state
            .attributes
            .iter()
            .take(count)
            .position(|a| a.id == attrib)
            .map_or(-1, |i| i as i32)
    }
    /// The interleaved byte offset of an attribute (sum of the preceding strides); `buffer.attribute_offset(1)`.
    fn attribute_offset(&self, index: i32) -> i32 {
        let state = self.buffer_base().state.read();
        state
            .attributes
            .iter()
            .take(index.max(0) as usize)
            .map(Graphic3dAttribute::stride)
            .sum()
    }
    /// `AttributeData`/`ChangeAttributeData`: `(index, byte offset, stride)` of an attribute
    /// role, interleaved or packed per attribute; `None` (natively null) when absent.
    /// Example: `buffer.attribute_data(Graphic3dTypeOfAttribute::Pos)`.
    fn attribute_data(&self, attrib: Graphic3dTypeOfAttribute) -> Option<(i32, usize, usize)> {
        attribute_data_of(
            &self.buffer_base().state.read(),
            self.is_interleaved(),
            attrib,
        )
    }
    /// The byte offset of element `index` (`value`/`changeValue`); `buffer.element_offset(2)`.
    fn element_offset(&self, index: i32) -> usize {
        (self.buffer_base().state.read().stride as usize).wrapping_mul(index as usize)
    }
    /// `Value<T>(theElem)`: the value at element `index`; `None` past the bytes (undefined natively).
    /// Example: `buffer.value::<Vec3<f32>>(0)`.
    fn value<T: Graphic3dBufferValue>(&self, index: i32) -> Option<T> {
        self.read_at(self.element_offset(index))
    }
    /// `ChangeValue<T>(theElem) = value`; false past the bytes. `buffer.set_value(0, 7u16)`.
    fn set_value<T: Graphic3dBufferValue>(&self, index: i32, value: T) -> bool {
        self.write_at(self.element_offset(index), value)
    }
    /// The value at a byte offset; `buffer.read_at::<f32>(4)`.
    fn read_at<T: Graphic3dBufferValue>(&self, offset: usize) -> Option<T> {
        let state = self.buffer_base().state.read();
        state
            .data
            .as_slice()
            .get(offset..offset.checked_add(T::SIZE)?)
            .map(T::read)
    }
    /// Writes a value at a byte offset; false past the bytes. `buffer.write_at(4, 1.0f32)`.
    fn write_at<T: Graphic3dBufferValue>(&self, offset: usize, value: T) -> bool {
        let mut state = self.buffer_base().state.write();
        let end = offset.checked_add(T::SIZE);
        let target = end.and_then(|end| state.data.as_mut_slice().get_mut(offset..end));
        target.map(|bytes| value.write(bytes)).is_some()
    }
    /// A copy of the data bytes (`Data()`); `buffer.data_bytes()`.
    fn data_bytes(&self) -> Vec<u8> {
        self.buffer_base().state.read().data.as_slice().to_vec()
    }
    /// The element count in use; `buffer.nb_elements()`.
    fn nb_elements(&self) -> i32 {
        self.buffer_base().state.read().nb_elements
    }
    /// Sets the element count in use (the public `NbElements` field); `buffer.set_nb_elements(3)`.
    fn set_nb_elements(&self, count: i32) {
        self.buffer_base().state.write().nb_elements = count;
    }
    /// The stride; `buffer.stride()`.
    fn stride(&self) -> i32 {
        self.buffer_base().state.read().stride
    }
    /// The attribute count; `buffer.nb_attributes()`.
    fn nb_attributes(&self) -> i32 {
        self.buffer_base().state.read().nb_attributes
    }
    /// `release`: frees the bytes and zeroes the layout; `buffer.release()?`.
    fn release(&self) -> Result<(), Failure> {
        let mut state = self.buffer_base().state.write();
        state.data.free()?;
        (state.stride, state.nb_elements, state.nb_attributes) = (0, 0, 0);
        state.attributes.clear();
        Ok(())
    }
    /// `Init`: lays `count` elements of the attributes out interleaved; false for a zero
    /// stride or a failed allocation. With 0 elements nothing is allocated and, as natively,
    /// the attribute table is not stored. Example: `buffer.init(4, &attributes)?`.
    fn init(&self, count: i32, attributes: &[Graphic3dAttribute]) -> Result<bool, Failure> {
        self.release()?;
        let stride: i32 = attributes.iter().map(Graphic3dAttribute::stride).sum();
        if stride == 0 {
            return Ok(false);
        }
        let mut state = self.buffer_base().state.write();
        (state.stride, state.nb_elements, state.nb_attributes) =
            (stride, count, attributes.len() as i32);
        if count == 0 {
            return Ok(true);
        }
        if !state
            .data
            .allocate((stride as usize).wrapping_mul(count as usize))?
        {
            drop(state);
            self.release()?;
            return Ok(false);
        }
        state.attributes = attributes.to_vec();
        Ok(true)
    }
}
impl<T: Graphic3dBufferSource + ?Sized> Graphic3dBufferMembers for T {}
