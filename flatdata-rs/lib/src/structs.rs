//! This module contains traits that are used by the generated code to
//! define flatdata's structs.
//!
//! flatdata's code generator translates a flatdata schema to Rust code. The
//! generated code contains all schema definitions embedded as strings, and for
//! each schema struct it defines 3 Rust structs: a factory with the same name
//! as the archive struct, a reference struct for reading with suffix `Ref` and
//! a mutable reference struct for writing with suffix `Mut`. The factory
//! is used to bind the lifetime to the references as late as possible. It
//! implement the `Struct` trait.
//!
//! ## Example
//!
//! A flatdata struct
//!
//! ```
//! struct A {
//!     x : u32 : 16;
//!     y : u32 : 16;
//! }
//! ```
//!
//! yields a factor `A` which is an empty struct. It implements `Struct` by
//! providing the schema, size in bytes which is 32 here, and refers to the
//! reference type `ARef` and `AMut`. The latter structs are pointers to
//! the location where the data is stored. The data is accessed for reading
//! and writing by using getters.
//!
//! All containers are parameterized by factory structs, i.e. in this example
//! an read-only array of `A` is represented as `ArrayView<A>`. When indexing
//! an element in it, it returns a reference of type `ARef` with lifetime
//! bound to it. Similarly, `Vector<A>` is an array for writing `A`'s.
//!
//! ## Indexes and variadic types
//!
//! A `MultiVector` is a heterogeneous container which consists of indexed
//! items, each containing several elements of different types (cf.
//! `MultiVector`). The generator generates two further types for each
//! multivector: an index type with a single field which is used for
//! index the data, and a variadic type which unifies the heterogeneous
//! types in the multivector.

use std::fmt::Debug;

#[doc(hidden)]
pub use std::marker;

/// A type in flatdata used for reading data.
///
/// Each struct reference in generated code implements this trait.
pub trait Ref: Clone + Copy + Debug + PartialEq {}

/// A mutable type in flatdata used for writing data.
///
/// Each struct reference in generated code has a corresponding type with suffix
/// `Mut` which implements this trait.
pub trait RefMut: Debug {}

/// A factory trait used to bind lifetime to Ref implementations.
///
/// Vector/ArrayView-like classes cannot be directly implemented over the
/// structs since that binds lifetime too early. Instead this generic factory
/// and Higher-Rank-Trait-Bounds are used to emulate higher-kinded-generics.
pub trait Struct<'a>: Clone {
    /// Schema of the type. Used only for debug and inspection purposes.
    const SCHEMA: &'static str;
    /// Size of an object of this type in bytes.
    const SIZE_IN_BYTES: usize;
    /// Whether this structs requires data of the next instance
    const IS_OVERLAPPING_WITH_NEXT: bool;

    /// Item this factory will produce.
    type Item: Ref;

    /// Creates a new item from a slice.
    fn create(data: &'a [u8]) -> Self::Item;

    /// Item this factory will produce.
    type ItemMut: RefMut;

    /// Creates a new item from a slice.
    fn create_mut(data: &'a mut [u8]) -> Self::ItemMut;
}

/// Shortcut trait for Structs that are able to produce references of any given
/// lifetime
///
/// Equivalent to ```for<'a> Struct<'a>'''
pub trait RefFactory: for<'a> Struct<'a> {}
impl<T> RefFactory for T where T: for<'a> Struct<'a> {}

/// Marks structs that can be used stand-alone, e.g. no range
pub trait NoOverlap {}

/// A specialized Struct factory producing Index items.
/// Used primarily by the MultiVector/MultiArrayView.
pub trait IndexStruct<'a>: Struct<'a> {
    /// Provide getter for index
    fn range(data: Self::Item) -> std::ops::Range<usize>;

    /// Provide setter for index
    fn set_index(data: Self::ItemMut, value: usize);
}

/// Shortcut trait for IndexStructs that are able to produce references of any
/// given lifetime
///
/// Equivalent to ```for<'a> IndexStruct<'a>'''
pub trait IndexRefFactory: for<'a> IndexStruct<'a> {}
impl<T> IndexRefFactory for T where T: for<'a> IndexStruct<'a> {}

/// A type in archive used as index of a `MultiArrayView`.
pub trait Index: Ref {}

/// A type in archive used as mutable index of a `MultiVector`.
pub trait IndexMut: RefMut {}

/// Index specifying a variadic type of `MultiArrayView`.
pub type TypeIndex = u8;

/// A type used as element of `MultiArrayView`.
///
/// Implemented by an enum type.
pub trait VariadicRef: Clone + Debug + PartialEq {
    /// Returns size in bytes of the current variant type.
    ///
    /// Since a variadic struct can contain types of different sized, this is a
    /// method based on the current value type.
    fn size_in_bytes(&self) -> usize;
}

/// A type used to create VariadicStructs.
///
/// Vector/ArrayView-like classes cannot be directly implemented over the
/// structs since that binds lifetime too early. Instead this generic factory
/// and Higher-Rank-Trait-Bounds are used to emulate higher-kinded-generics.
pub trait VariadicStruct<'a>: Clone {
    /// Index type
    type Index: IndexRefFactory;

    /// Reader type
    type Item: VariadicRef;

    /// Creates a reader for specific type of data.
    fn create(data: TypeIndex, _: &'a [u8]) -> Self::Item;

    /// Associated type used for building an item in `MultiVector` based on
    /// this variadic type.
    ///
    /// The builder is returned by
    /// [`MultiVector::grow`](struct.MultiVector.html#method.grow)
    /// method. It provides convenient methods `add_{variant_name}` for each
    /// enum variant.
    type ItemMut;

    /// Creates a builder for a list of VariadicRef.
    fn create_mut(data: &'a mut Vec<u8>) -> Self::ItemMut;
}

/// Shortcut trait for VariadicStructs that are able to produce references of
/// any given lifetime
///
/// Equivalent to ```for<'a> VariadicStruct<'a>'''
pub trait VariadicRefFactory: for<'a> VariadicStruct<'a> {}
impl<T> VariadicRefFactory for T where T: for<'a> VariadicStruct<'a> {}

#[cfg(test)]
mod test {
    use super::{
        super::{helper::Int, structbuf::StructBuf},
        *,
    };

    macro_rules! define_enum_test {
        ($test_name:ident, $type:tt, $is_signed:expr, $val1:expr, $val2:expr) => {
            #[test]
            #[allow(dead_code)]
            fn $test_name() {
                #[derive(Debug, PartialEq, Eq)]
                #[repr($type)]
                pub enum Variant {
                    X = $val1,
                    Y = $val2,
                }

                impl Int for Variant {
                    const IS_SIGNED: bool = $is_signed;
                }


                #[derive(Clone, Debug)]
                pub struct A {}

                #[derive(Clone, Copy)]
                pub struct ARef<'a> {
                    data: *const u8,
                    _phantom: std::marker::PhantomData<&'a u8>,
                }

                impl<'a> Struct<'a> for A {
                    const SCHEMA: &'static str = "";
                    const SIZE_IN_BYTES: usize = 1;
                    const IS_OVERLAPPING_WITH_NEXT: bool = false;

                    type Item = ARef<'a>;

                    #[inline]
                    fn create(data: &'a [u8]) -> Self::Item {
                        Self::Item {
                            data: data.as_ptr(),
                            _phantom: std::marker::PhantomData,
                        }
                    }

                    type ItemMut = AMut<'a>;

                    #[inline]
                    fn create_mut(data: &'a mut [u8]) -> Self::ItemMut {
                        Self::ItemMut {
                            data: data.as_mut_ptr(),
                            _phantom: std::marker::PhantomData,
                        }
                    }
                }

                impl NoOverlap for A {}

                impl<'a> ARef<'a> {
                    #[inline]
                    pub fn x(&self) -> Variant {
                        let value = flatdata_read_bytes!($type, self.data, 0, 2);
                        unsafe { std::mem::transmute::<$type, Variant>(value) }
                    }

                    #[inline]
                    pub fn as_ptr(&self) -> *const u8 {
                        self.data
                    }
                }

                impl<'a> std::fmt::Debug for ARef<'a> {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        f.debug_struct("A").field("x", &self.x()).finish()
                    }
                }

                impl<'a> std::cmp::PartialEq for ARef<'a> {
                    #[inline]
                    fn eq(&self, other: &Self) -> bool {
                        self.x() == other.x()
                    }
                }

                impl<'a> Ref for ARef<'a> {}

                pub struct AMut<'a> {
                    data: *mut u8,
                    _phantom: std::marker::PhantomData<&'a u8>,
                }

                impl<'a> AMut<'a> {
                    #[inline]
                    pub fn x(&self) -> Variant {
                        let value = flatdata_read_bytes!($type, self.data, 0, 8);
                        unsafe { std::mem::transmute::<$type, Variant>(value) }
                    }

                    #[inline]
                    pub fn set_x(&mut self, value: Variant) {
                        let buffer = unsafe { std::slice::from_raw_parts_mut(self.data, 1) };
                        flatdata_write_bytes!(u8; value, buffer, 0, 8)
                    }

                    #[inline]
                    pub fn fill_from(&mut self, other: &ARef) {
                        self.set_x(other.x());
                    }

                    #[inline]
                    pub fn as_ptr(&self) -> *const u8 {
                        self.data
                    }

                    #[inline]
                    pub fn as_mut_ptr(&self) -> *mut u8 {
                        self.data
                    }
                }

                impl<'a> std::fmt::Debug for AMut<'a> {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        ARef {
                            data: self.data,
                            _phantom: std::marker::PhantomData,
                        }
                        .fmt(f)
                    }
                }

                impl<'a> RefMut for AMut<'a> {}


                // define_struct!(
                //     A,
                //     RefA,
                //     RefMutA,
                //     "no_schema",
                //     1,
                //     (x, set_x, Variant, $type, 0, 2)
                // );

                let mut a = StructBuf::<A>::new();
                let output = format!("{:?}", a);
                assert_eq!(output, "StructBuf { resource: A { x: X } }");

                a.get_mut().set_x(Variant::Y);
                let output = format!("{:?}", a);
                assert_eq!(output, "StructBuf { resource: A { x: Y } }");
            }
        };
    }

    define_enum_test!(test_enum_u8_1, u8, false, 0, 1);
    define_enum_test!(test_enum_u8_2, u8, false, 0, 2);
    define_enum_test!(test_enum_u16_1, u16, false, 0, 1);
    define_enum_test!(test_enum_u16_2, u16, false, 0, 2);
    define_enum_test!(test_enum_u32_1, u32, false, 0, 1);
    define_enum_test!(test_enum_u32_2, u32, false, 0, 2);
    define_enum_test!(test_enum_u64_1, u64, false, 0, 1);
    define_enum_test!(test_enum_u64_2, u64, false, 0, 2);

    // Note: Right now, there a regression bug for binary enums with underlying
    // type i8: https://github.com/rust-lang/rust/issues/51582
    //
    // Until it is backported into stable release, we have to disable this test.
    //
    // define_enum_test!(test_enum_i8, i8, true, 0, 1);
    // define_enum_test!(test_enum_i8, i8, true, 0, -1);
    define_enum_test!(test_enum_i16_1, i16, true, 0, 1);
    define_enum_test!(test_enum_i16_2, i16, true, 0, -1);
    define_enum_test!(test_enum_i32_1, i32, true, 0, 1);
    define_enum_test!(test_enum_i32_2, i32, true, 0, -1);
    define_enum_test!(test_enum_i64_1, i64, true, 0, 1);
    define_enum_test!(test_enum_i64_2, i64, true, 0, -1);
}
