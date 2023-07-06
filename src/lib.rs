/// A single **guest-side** paramter which can be moved across guest-host boundary.
pub trait Parameter {
    type Host;
    fn from_host(host: Self::Host) -> Self;
    fn into_host(self) -> Self::Host;
}

macro_rules! identity_impls {
    () => {};
    ($type:ty $(,$other:ty)* $(,)?) => {
        identity_impls!(@ $type);
        identity_impls!($($other,)*);
    };
    (@ $type:ty) => {
        impl $crate::Parameter for $type {
            type Host = $type;
            fn from_host(host: Self::Host) -> Self {
                host
            }
            fn into_host(self) -> Self::Host {
                self
            }
        }
    };
}

identity_impls!(u8, u16, u32, u64, i8, i16, i32, i64);

/// A function definition that can be shared between a guest and a host.
pub trait Function {}

macro_rules! fn_impls {
    () => {};
    ($type:tt $(,$other:tt)* $(,)?) => {
        fn_impls!(@ $type $(,$other)*);
        fn_impls!($($other,)*);
    };
    (@ $head:tt) => {
        impl<$head> $crate::Function for fn($head)
        where
            $head: $crate::Parameter
        {
        }
    };
    (@ $head:tt $(,$other:tt)*) => {
        impl<$head $(,$other)*> $crate::Function for fn($head $(,$other)*)
        where
            $head: $crate::Parameter
            $(,
                $other: $crate::Parameter
            )*
        {
        }
    };
}

fn_impls!(T1, T2, T3, T4, T5, T6, T7, T8);
