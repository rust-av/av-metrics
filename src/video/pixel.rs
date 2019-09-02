//! Traits for generic code over low and high bit depth video.
//!
//! Borrowed from rav1e.

use num_traits::{AsPrimitive, PrimInt};
use std::fmt::{Debug, Display};

/// Defines a type which supports being cast to from a generic integer type.
///
/// Intended for casting to and from a [`Pixel`](trait.Pixel.html).
pub trait CastFromPrimitive<T>: Copy + 'static {
    /// Cast from a generic integer type to the given type.
    fn cast_from(v: T) -> Self;
}

macro_rules! impl_cast_from_primitive {
  ( $T:ty => $U:ty ) => {
    impl CastFromPrimitive<$U> for $T {
      #[inline(always)]
      fn cast_from(v: $U) -> Self { v as Self }
    }
  };
  ( $T:ty => { $( $U:ty ),* } ) => {
    $( impl_cast_from_primitive!($T => $U); )*
  };
}

// casts to { u8, u16 } are implemented separately using Pixel, so that the
// compiler understands that CastFromPrimitive<T: Pixel> is always implemented
impl_cast_from_primitive!(u8 => { u32, u64, usize });
impl_cast_from_primitive!(u8 => { i8, i16, i32, i64, isize });
impl_cast_from_primitive!(u16 => { u32, u64, usize });
impl_cast_from_primitive!(u16 => { i8, i16, i32, i64, isize });
impl_cast_from_primitive!(i16 => { u32, u64, usize });
impl_cast_from_primitive!(i16 => { i8, i16, i32, i64, isize });
impl_cast_from_primitive!(i32 => { u32, u64, usize });
impl_cast_from_primitive!(i32 => { i8, i16, i32, i64, isize });

#[doc(hidden)]
pub enum PixelType {
    U8,
    U16,
}

/// A trait for types which may represent a pixel in a video.
/// Currently implemented for `u8` and `u16`.
/// `u8` should be used for low-bit-depth video, and `u16`
/// for high-bit-depth video.
pub trait Pixel:
    PrimInt
    + Into<u32>
    + Into<i32>
    + AsPrimitive<u8>
    + AsPrimitive<i16>
    + AsPrimitive<u16>
    + AsPrimitive<i32>
    + AsPrimitive<u32>
    + AsPrimitive<usize>
    + CastFromPrimitive<u8>
    + CastFromPrimitive<i16>
    + CastFromPrimitive<u16>
    + CastFromPrimitive<i32>
    + CastFromPrimitive<u32>
    + CastFromPrimitive<usize>
    + Debug
    + Display
    + Send
    + Sync
    + 'static
{
    #[doc(hidden)]
    fn type_enum() -> PixelType;
}

impl Pixel for u8 {
    fn type_enum() -> PixelType {
        PixelType::U8
    }
}

impl Pixel for u16 {
    fn type_enum() -> PixelType {
        PixelType::U16
    }
}

macro_rules! impl_cast_from_pixel_to_primitive {
    ( $T:ty ) => {
        impl<T: Pixel> CastFromPrimitive<T> for $T {
            #[inline(always)]
            fn cast_from(v: T) -> Self {
                v.as_()
            }
        }
    };
}

impl_cast_from_pixel_to_primitive!(u8);
impl_cast_from_pixel_to_primitive!(i16);
impl_cast_from_pixel_to_primitive!(u16);
impl_cast_from_pixel_to_primitive!(i32);
impl_cast_from_pixel_to_primitive!(u32);
