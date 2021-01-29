use bytemuck::try_cast_slice;
use heed_traits::{BytesDecode, BytesEncode};
use std::{borrow::Cow, marker, str};

/// Describes an [`prim@str`].
pub struct Str<'a> {
    _phantom: marker::PhantomData<&'a ()>,
}

impl<'a> BytesEncode for Str<'a> {
    type EItem = &'a str;

    fn bytes_encode(item: &Self::EItem) -> anyhow::Result<Cow<[u8]>> {
        try_cast_slice(item.as_bytes())
            .map(Cow::Borrowed)
            .map_err(Into::into)
    }
}

impl<'a> BytesDecode<'a> for Str<'_> {
    type DItem = &'a str;

    fn bytes_decode(bytes: &'a [u8]) -> anyhow::Result<Self::DItem> {
        str::from_utf8(bytes).map_err(Into::into)
    }
}
