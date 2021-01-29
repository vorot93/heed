use bytemuck::{try_cast_slice, Pod};
use heed_traits::{BytesDecode, BytesEncode};
use std::borrow::Cow;

/// Describes a type that is totally borrowed and doesn't
/// depends on any [memory alignment].
///
/// If you need to store a type that does depend on memory alignment
/// and that can be big it is recommended to use the [`CowType`].
///
/// [memory alignment]: std::mem::align_of()
/// [`CowType`]: crate::CowType
pub struct UnalignedSlice<'a, T>(std::marker::PhantomData<&'a T>);

impl<'a, T: Pod> BytesEncode for UnalignedSlice<'a, T> {
    type EItem = &'a [T];

    fn bytes_encode(item: &Self::EItem) -> anyhow::Result<Cow<[u8]>> {
        try_cast_slice(item).map(Cow::Borrowed).map_err(Into::into)
    }
}

impl<'a, T: Pod> BytesDecode<'a> for UnalignedSlice<'_, T> {
    type DItem = &'a [T];

    fn bytes_decode(bytes: &'a [u8]) -> anyhow::Result<Self::DItem> {
        try_cast_slice(bytes).map_err(Into::into)
    }
}

unsafe impl<T> Send for UnalignedSlice<'_, T> {}

unsafe impl<T> Sync for UnalignedSlice<'_, T> {}
