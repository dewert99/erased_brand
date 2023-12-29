pub use crate::*;

impl<T: Branded> Erased<T> {
    /// Consumes self and passes it to `f` and returns the result
    pub fn into_inner<U>(self, f: impl for<'brand> FnOnce(T::This<'brand>) -> U) -> U {
        let mut res = None;
        self.map::<()>(|token, _| res = Some(f(token)));
        res.unwrap()
    }
}
