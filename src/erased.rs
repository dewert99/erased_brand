use crate::hkt::*;
use core::marker::PhantomData;

/// Wrapper around a `Branded` that erases it's brand
/// an `Erased<T>` acts similar to `T::This<'static>`,
/// but ensures that two erased types can't be seen to share there lifetime parameter.
/// It can be more accurately thought of as `exists<'brand> T::This<'brand>`
/// where `'brand` can live as long as required but can never be unified with `'static`
/// This is useful for cases where the lifetime is used as a unique brand
#[repr(transparent)]
pub struct Erased<T: Branded>(T::This<'static>);

impl<T: Branded> Erased<T> {
    /// Wraps value to be erased
    /// While safe this isn't particularly useful on it's own since `T::This<'static>` is strictly more useful than `Self`
    /// The main use case for this method is as a way of exposing an api for branded singleton types
    pub fn new(inner: T::This<'static>) -> Self {
        Erased(inner)
    }

    /// Maps `self` while keeping it erased
    /// The callback takes an additional PhantomData<&()> parameter to anchor lifetimes (see [#86702](https://github.com/rust-lang/rust/issues/86702)) This parameter should just be ignored in the callback.
    pub fn map<U: Branded>(
        self,
        f: impl for<'brand> FnOnce(T::This<'brand>, PhantomData<&'brand ()>) -> U::This<'brand>,
    ) -> Erased<U> {
        Erased(f(self.0, PhantomData))
    }

    /// Calls `f` with shared access to the internal data and returns it's result.
    ///
    /// Note: using the naive signature:
    /// `(&'a self, f: impl for<'brand> FnOnce(&'a T::This<'brand>) -> U) -> U`
    /// would be unsound when `'a` was instanciated with `'static` resulting in
    /// `(&'static self, f: impl for<'brand> FnOnce(&'static T::This<'brand>) -> U) -> U`.
    /// This would allow the compiler to assume that `&'static T::This<'brand>` is well formed so `'brand` must outlive `'static`.
    /// Since the only lifetime this is true for is `'static`, this would imply `'brand='static` simplifying the signature to
    /// `(&'static self, f: impl FnOnce(&'static T::This<'static>) -> U) -> U`.
    /// With the HRTB gone we can simply pass the identity function which will allow `T::This<'static>` to escape
    ///
    /// The name `borrow_hkt` is chosen to leave the `borrow` open for if a more ergonomic solution is found
    ///
    /// # Examples
    ///
    /// ```
    /// use std::marker::PhantomData;
    /// use erased_brand::{Branded, Erased};
    /// use erased_brand::hkt;
    /// struct PhantomBrand<'a>(PhantomData<*mut &'a ()>);
    /// struct PhantomBrandHKT;
    /// impl Branded for PhantomBrandHKT {
    ///     type This<'brand> = (u32, PhantomBrand<'brand>);
    /// }
    ///
    /// fn good(x: &'static Erased<PhantomBrandHKT>) {
    ///     let good: &'static u32 = x.borrow_hkt::<hkt::Ref<_>>(|x| &x.0);
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use std::marker::PhantomData;
    /// use erased_brand::{Branded, Erased};
    /// use erased_brand::hkt;
    /// struct PhantomBrand<'a>(PhantomData<*mut &'a ()>);
    /// struct PhantomBrandHKT;
    /// impl Branded for PhantomBrandHKT {
    ///     type This<'brand> = PhantomBrand<'brand>;
    /// }
    ///
    /// fn unsound(x: &'static Erased<PhantomBrandHKT>) {
    ///     let bad = x.borrow_hkt::<hkt::Ref<_>>(|x| x);
    /// }
    /// ```
    pub fn borrow_hkt<'a, U: HKTRef>(
        &'a self,
        f: impl for<'a1, 'brand> FnOnce(&'a1 T::This<'brand>) -> U::This<'a1>,
    ) -> U::This<'a> {
        f(&self.0)
    }

    /// Calls `f` with mutable access to the internal data and returns it's result.
    ///
    /// See [`Self::borrow_hkt`] for details on the unusual signature
    pub fn borrow_mut_hkt<'a, U: HKTRef>(
        &'a mut self,
        f: impl for<'a1, 'brand> FnOnce(&'a1 mut T::This<'brand>) -> U::This<'a1>,
    ) -> U::This<'a> {
        f(&mut self.0)
    }
}

impl<T: Branded2 + for<'brand> Branded<This<'brand> = Erased<Branded2Wrap<'brand, T>>>> Erased<T> {
    /// Combines two layers of erasure turning the inner type from `T::This2<'brand1, 'brand2>` to `T::This2<'brand, 'brand>`
    ///
    /// # Safety
    /// `T::This2<'brand1, 'brand2>` must not contain two versions of the same singleton type (one for each brand)
    ///
    /// Note: A good strategy for writing a safe wrapper would be to force the caller
    /// to expose two versions of the same singleton type and then eliminate one of them before calling this method.
    ///
    /// Because of the above technique it is likely a bad idea to have allow different singleton types sharing a brand inside of an [`Erased`].
    /// Since [`Erased`] doesn't allow this to happen when using safely this consideration only applies when dealing with other unsafe code
    pub unsafe fn flatten(self) -> Erased<Flattened<T>> {
        Erased(self.0 .0)
    }
}