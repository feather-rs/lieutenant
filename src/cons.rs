//! Liberally borrowed from https://github.com/TomGillen/legion/blob/master/legion_core/src/cons.rs, licensed
//! under MIT.

/// Append a new type into a cons list
pub trait ConsAppend<T> {
    /// Result of append
    type Output;
    /// Append to runtime cons value
    fn append(self, t: T) -> Self::Output;
}

impl<T> ConsAppend<T> for () {
    type Output = (T, Self);
    fn append(self, t: T) -> Self::Output {
        (t, ())
    }
}

impl<T, A, B: ConsAppend<T>> ConsAppend<T> for (A, B) {
    type Output = (A, <B as ConsAppend<T>>::Output);
    fn append(self, t: T) -> Self::Output {
        let (a, b) = self;
        (a, b.append(t))
    }
}

/// transform cons list into a flat tuple
pub trait ConsFlatten {
    /// Flattened tuple
    type Output;
    /// Flatten runtime cons value
    fn flatten(self) -> Self::Output;
}

impl ConsFlatten for () {
    type Output = ();
    fn flatten(self) -> Self::Output {
        self
    }
}

macro_rules! cons {
    () => (
        ()
    );
    ($head:tt) => (
        ($head, ())
    );
    ($head:tt, $($tail:tt),*) => (
        ($head, cons!($($tail),*))
    );
}

macro_rules! impl_flatten {
    ($($items:ident),*) => {
    #[allow(unused_parens)] // This is added because the nightly compiler complains
        impl<$($items),*> ConsFlatten for cons!($($items),*)
        {
            type Output = ($($items),*);
            fn flatten(self) -> Self::Output {
                #[allow(non_snake_case)]
                let cons!($($items),*) = self;
                ($($items),*)
            }
        }

        impl_flatten!(@ $($items),*);
    };
    (@ $head:ident, $($tail:ident),*) => {
        impl_flatten!($($tail),*);
    };
    (@ $head:ident) => {};
}

impl_flatten!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
