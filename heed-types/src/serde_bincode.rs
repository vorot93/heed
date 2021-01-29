use heed_traits::{BytesDecode, BytesEncode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Describes a type that is [`Serialize`]/[`Deserialize`] and uses `bincode` to do so.
///
/// It can borrow bytes from the original slice.
pub struct SerdeBincode<T>(std::marker::PhantomData<T>);

impl<T> BytesEncode for SerdeBincode<T>
where
    T: Serialize,
{
    type EItem = T;

    fn bytes_encode(item: &Self::EItem) -> anyhow::Result<Cow<[u8]>> {
        bincode::serialize(item).map(Cow::Owned).map_err(Into::into)
    }
}

impl<'a, T: 'a> BytesDecode<'a> for SerdeBincode<T>
where
    T: Deserialize<'a>,
{
    type DItem = T;

    fn bytes_decode(bytes: &'a [u8]) -> anyhow::Result<Self::DItem> {
        bincode::deserialize(bytes).map_err(Into::into)
    }
}

unsafe impl<T> Send for SerdeBincode<T> {}

unsafe impl<T> Sync for SerdeBincode<T> {}
