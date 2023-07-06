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

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
