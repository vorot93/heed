use bytemuck::Pod;
use heed_traits::{BytesDecode, BytesEncode};
use std::borrow::Cow;

use crate::CowType;

/// Describes a type that is totally owned (doesn't
/// hold any reference to the original slice).
///
/// If you need to store a type that doesn't depends on any
/// [memory alignment] and that can be big it is recommended
/// to use the [`UnalignedType`].
///
/// The [`CowType`] is recommended for borrowed types (types that holds
/// references to the original slice).
///
/// To store slices, you must look at the [`CowSlice`],
/// [`OwnedSlice`] or [`UnalignedSlice`] types.
///
/// [memory alignment]: std::mem::align_of()
/// [`UnalignedType`]: crate::UnalignedType
/// [`CowType`]: crate::CowType
/// [`UnalignedSlice`]: crate::UnalignedSlice
/// [`OwnedSlice`]: crate::OwnedSlice
/// [`CowSlice`]: crate::CowSlice
pub struct OwnedType<T>(std::marker::PhantomData<T>);

impl<T: Pod> BytesEncode for OwnedType<T> {
    type EItem = T;

    fn bytes_encode(item: &Self::EItem) -> anyhow::Result<Cow<[u8]>> {
        CowType::bytes_encode(item)
    }
}

impl<'a, T: Pod> BytesDecode<'a> for OwnedType<T> {
    type DItem = T;

    fn bytes_decode(bytes: &[u8]) -> anyhow::Result<Self::DItem> {
        CowType::<T>::bytes_decode(bytes).map(Cow::into_owned)
    }
}

unsafe impl<T> Send for OwnedType<T> {}

unsafe impl<T> Sync for OwnedType<T> {}
