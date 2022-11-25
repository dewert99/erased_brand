//! This could be moved to the [`ghost_cell`] crate but this would require using extension traits

use crate::{hkt::*, Erased};
use ghost_cell::GhostToken;

/// HKT for a tuple of a some data and a [`GhostToken`]
pub struct WithGhostTokenBranded<T: Branded>(T);

impl<T: Branded> Branded for WithGhostTokenBranded<T> {
    type This<'brand> = (T::This<'brand>, GhostToken<'brand>);
}

pub type WithGhostToken<T> = Erased<WithGhostTokenBranded<T>>;

unsafe fn static_ghost_token() -> GhostToken<'static> {
    GhostToken::new(|g| std::mem::transmute(g))
}

impl WithGhostToken<()> {
    /// Creates an erased [`GhostToken`]
    ///
    /// This can be used to implement [`GhostToken::new()`]
    /// ```
    /// use ghost_cell::GhostToken;
    /// use erased_brand::ghost_token::WithGhostToken;
    /// fn new_ghost_token<T>(f: impl for<'brand> FnOnce(GhostToken<'brand>) -> T) -> T {
    ///     WithGhostToken::<()>::new_token().into_inner(|((), gt)| f(gt))
    /// }
    /// ```
    pub fn new_token() -> WithGhostToken<()> {
        let g = unsafe { static_ghost_token() };
        Self::new(((), g))
    }
}

impl<T: Branded2 + for<'brand> Branded<This<'brand> = Erased<Branded2Wrap<'brand, T>>>>
    WithGhostToken<T>
{
    /// Combines two brands by removing the outer [`GhostToken`]
    pub fn flatten_ghost_tokens(self) -> Erased<Flattened<T>> {
        // SAFETY
        // The outer brand contained a GhostToken so it be used with any other structs with singleton types
        // After removing it's ghost token it has no singleton types so it is safe to flatten into the inner brand
        unsafe { self.map(|(data, _g), _| (data)).flatten() }
    }
}


impl<T: Branded> Erased<T> {
    pub fn from_ghost_token(
        f: impl for<'brand> FnOnce(GhostToken<'brand>) -> T::This<'brand>,
    ) -> Erased<T> {
        WithGhostToken::<()>::new_token().map(|g, _| f(g.1))
    }
}

impl<T: Branded> WithGhostToken<T> {
    pub fn new_with_token(
        f: impl for<'brand> FnOnce(&mut GhostToken<'brand>) -> T::This<'brand>,
    ) -> WithGhostToken<T> {
        WithGhostToken::<()>::new_token().map(|mut g, _| (f(&mut g.1), g.1))
    }

    pub fn merge_tokens<U: Branded>(self, other: Erased<U>) -> Erased<Flattened<(T, U)>> {
        self.map(|(data1, ghost1), _| (other.map(|data2, _| (data1, data2)), ghost1))
            .flatten_ghost_tokens()
    }
}

