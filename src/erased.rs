use crate::hkt::*;
use core::marker::PhantomData;

/// Essentially an alias for `FnOnce(&'a T::This<'brand>) -> Self::Output` including a blanket impl
pub trait BrandFn<'a, T: Branded> {
    type Output
    where
        T: 'a;

    fn call<'brand>(self, x: &'a T::This<'brand>) -> Self::Output;
}

/// Essentially an alias for `FnOnce(&'a mut T::This<'brand>) -> Self::Output` including a blanket impl
pub trait BrandMutFn<'a, T: Branded> {
    type Output
    where
        T: 'a;

    fn call<'brand>(self, x: &'a mut T::This<'brand>) -> Self::Output;
}

impl<'a, T: Branded, F: for<'brand> FnOnce(&'a T::This<'brand>) -> U, U> BrandFn<'a, T> for F {
    type Output = U where T: 'a;

    fn call<'brand>(self, x: &'a T::This<'brand>) -> U {
        self(x)
    }
}

impl<'a, T: Branded, F: for<'brand> FnOnce(&'a mut T::This<'brand>) -> U, U> BrandMutFn<'a, T>
    for F
{
    type Output = U where T: 'a;

    fn call<'brand>(self, x: &'a mut T::This<'brand>) -> U {
        self(x)
    }
}

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
    /// would be unsound when `'a` was instantiated with `'static` resulting in
    /// `(&'static self, f: impl for<'brand> FnOnce(&'static T::This<'brand>) -> U) -> U`.
    /// This would allow the compiler to assume that `&'static T::This<'brand>` is well formed so `'brand` must outlive `'static`.
    /// Since the only lifetime this is true for is `'static`, this would imply `'brand='static` simplifying the signature to
    /// `(&'static self, f: impl FnOnce(&'static T::This<'static>) -> U) -> U`.
    /// With the HRTB gone we can simply pass the identity function which will allow `T::This<'static>` to escape
    ///
    /// Unfortunately this function often seems to produces type errors when passing in anonymous functions, so passing
    /// in a named function or manually implementing `BrandFn` may be required
    ///
    /// # Examples
    ///
    /// ```
    /// use std::marker::PhantomData;
    /// use erased_brand::{Branded, Erased};
    /// use erased_brand::hkt;
    /// struct PhantomBrand<'brand>(PhantomData<*mut &'brand ()>);
    /// struct PhantomBrandHKT<X>(PhantomData<X>);
    /// impl<X> Branded for PhantomBrandHKT<X> {
    ///     type This<'brand> = (X, PhantomBrand<'brand>);
    /// }
    ///
    /// fn f0<'a, 'brand, X>(x: &'a (X, PhantomBrand<'brand>)) -> &'a X {
    ///     &x.0
    /// }
    ///
    /// fn good<X>(x: &Erased<PhantomBrandHKT<X>>) -> &X {
    ///     x.borrow(f0)
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use std::marker::PhantomData;
    /// use erased_brand::{Branded, Erased};
    /// use erased_brand::hkt;
    /// struct PhantomBrand<'brand>(PhantomData<*mut &'brand ()>);
    /// struct PhantomBrandHKT;
    /// impl Branded for PhantomBrandHKT {
    ///     type This<'brand> = PhantomBrand<'brand>;
    /// }
    ///
    /// fn id<'a: 'static, 'brand>(x: &'a PhantomBrand<'brand>) -> &'a PhantomBrand<'static> {
    ///     x
    /// }
    ///
    ///
    /// fn unsound(x: &'static Erased<PhantomBrandHKT>) -> &'static PhantomBrand<'static> {
    ///     x.borrow(id)
    /// }
    /// ```
    /// ```
    /// use std::collections::HashMap;
    /// use std::marker::PhantomData;
    /// use erased_brand::{Branded, Erased};
    /// use erased_brand::erased::BrandFn;
    /// use erased_brand::hkt;
    /// struct PhantomBrand<'brand>(PhantomData<*mut &'brand ()>);
    /// struct PhantomBrandHKT<X>(PhantomData<X>);
    /// impl<X> Branded for PhantomBrandHKT<X> {
    ///     type This<'brand> = (HashMap<String, X>, PhantomBrand<'brand>);
    /// }
    ///
    /// struct Lookup<'b>(&'b str);
    ///
    /// impl<'a, 'b, X> BrandFn<'a, PhantomBrandHKT<X>> for Lookup<'b> {
    ///     type Output = &'a X where X: 'a;
    ///
    ///     fn call<'brand>(self, x: &'a <PhantomBrandHKT<X> as Branded>::This<'brand>) -> Self::Output {
    ///         &x.0[self.0]
    ///     }
    /// }
    ///
    /// fn good<'a, 'b, X>(x: &'a Erased<PhantomBrandHKT<X>>, key: &'b str) -> &'a X {
    ///     x.borrow(Lookup(key))
    /// }
    /// ```

    pub fn borrow<'a, F>(&'a self, f: F) -> <F as BrandFn<'a, T>>::Output
    where
        F: for<'a1> BrandFn<'a1, T>,
    {
        f.call(&self.0)
    }

    /// Calls `f` with mutable access to the internal data and returns it's result.
    ///
    /// See [`Self::borrow_hkt`] for details
    pub fn borrow_mut_hkt<'a, F>(&'a mut self, f: F) -> <F as BrandMutFn<'a, T>>::Output
    where
        F: for<'a1> BrandMutFn<'a1, T>,
    {
        f.call(&mut self.0)
    }
}

impl<T: Branded2 + for<'brand> Branded<This<'brand> = Erased<Branded2Wrap<'brand, T>>>> Erased<T> {
    /// Combines two layers of erasure turning the inner type from `T::This2<'brand1, 'brand2>` to `T::This2<'brand, 'brand>`
    ///
    /// # Safety
    /// `T::This2<'brand1, 'brand2>` must not contain two versions of the same singleton type (one for each brand)
    ///
    /// Note: A good strategy for writing a safe wrapper would be to force the caller
    /// to expose a copy of singleton type and then eliminate one of them before calling this method.
    ///
    /// Because of the above technique it is likely a bad idea to have allow different singleton types sharing a brand inside of an [`Erased`].
    /// Since [`Erased`] doesn't allow this to happen when using safely this consideration only applies when dealing with other unsafe code
    pub unsafe fn flatten(self) -> Erased<Flattened<T>> {
        Erased(self.0 .0)
    }
}
