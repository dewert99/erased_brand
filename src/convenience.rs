pub use crate::*;

// // This would be helpful to help type inference but unfortunately leads
// // error[E0275]: overflow evaluating the requirement `<T as hkt::Branded>::This<'brand>`
// pub trait BrandFn<'a, T> {
//     type Output;
// }
//
// impl<'a, T: Branded, F: for<'brand> FnOnce(&'a T::This<'brand>) -> U, U> BrandFn<'a, T> for F {
//     type Output = U;
// }
//
// pub struct FunctionHKTRef<T, F>(T, F);
//
// impl<T: Branded, F: for<'a> BrandFn<'a, T>> HKTRef for FunctionHKTRef<T, F> {
//     type This<'a> = <F as BrandFn<'a, T>>::Output where Self: 'a,;
// }
//
// impl<T: Branded> Erased<T> {
//
//     pub fn borrow<'a, F>(&'a self, f: F) -> <FunctionHKTRef<T, F> as HKTRef>::This<'a>
//         where
//             FunctionHKTRef<T, F>: HKTRef,
//             F: for<'a1, 'brand> FnOnce(
//                 &'a1 T::This<'brand>,
//             ) -> <FunctionHKTRef<T, F> as HKTRef>::This<'a1>,
//     {
//         self.borrow_hkt::<FunctionHKTRef<T, F>>(f)
//     }
// }

impl<T: Branded> Erased<T> {
    /// Consumes self and passes it to `f` and returns the result
    pub fn into_inner<U>(self, f: impl for<'brand> FnOnce(T::This<'brand>) -> U) -> U {
        let mut res = None;
        self.map::<()>(|token, _| res = Some(f(token)));
        res.unwrap()
    }
}
