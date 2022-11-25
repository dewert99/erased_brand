use crate::erased::Erased;
use core::marker::PhantomData;

/// Uses GATs to mimic higher kinded types for branded types
pub trait Branded {
    type This<'brand>;
}

impl Branded for () {
    type This<'brand> = ();
}

/// Similar to [`Branded`] but for types with two lifetime parameters
///
/// This wrapper implements [`Branded`] by currying and erasing so that an [`Erased<Branded2>`] contains `This2` after 2 layers of unwrapping
pub trait Branded2: Branded {
    type This2<'brand1, 'brand2>;
}

/// Partially curried form of `T` representing `T::This<'brand1, _>`
pub struct Branded2Wrap<'brand1, T: Branded2>(T, PhantomData<&'brand1 ()>);

impl<'brand1, T: Branded2> Branded for Branded2Wrap<'brand1, T> {
    type This<'brand2> = T::This2<'brand1, 'brand2>;
}

impl<T: Branded2> Branded for T {
    type This<'brand1> = Erased<Branded2Wrap<'brand1, T>>;
}

/// Converts a [`Branded2`] to a [`Branded`] by duplicating the brand rather than by currying
pub struct Flattened<T: Branded2>(T);

impl<T: Branded2> Branded for Flattened<T> {
    type This<'brand> = T::This2<'brand, 'brand>;
}

impl<T: Branded, U: Branded> Branded2 for (T, U) {
    type This2<'brand1, 'brand2> = (T::This<'brand1>, U::This<'brand2>);
}

/// Uses GATs to mimic higher kinded types for types with normal lifetime parameters
pub trait HKTRef {
    type This<'a>
    where
        Self: 'a;
}

pub struct Ref<T>(PhantomData<T>);
impl<T> HKTRef for Ref<T> {
    type This<'a> = &'a T where T: 'a;
}

pub struct Mut<T>(PhantomData<T>);
impl<T> HKTRef for Mut<T> {
    type This<'a> = &'a mut T where T: 'a;
}

pub struct Owned<T>(PhantomData<T>);
impl<T> HKTRef for Owned<T> {
    type This<'a> = T where T: 'a;
}
