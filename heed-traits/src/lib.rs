#![feature(generic_associated_types)]

use std::borrow::Cow;
use std::cmp::Ordering;

pub trait BytesEncode {
    type EItem<'a>: ?Sized;

    fn bytes_encode<'a, 'b>(item: &'b Self::EItem<'a>) -> Option<Cow<'a, [u8]>>;
}

pub trait BytesDecode<'a> {
    type DItem: 'a;

    fn bytes_decode(bytes: &'a [u8]) -> Option<Self::DItem>;
}

pub trait CustomKeyCmp {
    fn compare(a: &[u8], b: &[u8]) -> Ordering;
}
