#[cfg(feature = "rt-wasmtime")]
use wasmtime::{Memory, Store};

/// A single **guest-side** paramter which can be moved across guest-host boundary.
pub trait Parameter {
    /// A host-side type representation.
    type Host;
    /// An intermediate representation to cross the FFI boundary.
    type Abi: Abi<Self::Host>;

    fn into_abi(self) -> Self::Abi;
    fn from_abi(abi: Self::Abi) -> Self;
}

/// A single WASM ABI compatible type.
pub trait Abi<Host> {
    fn into_host(self) -> Host;
    fn from_host(host: Host) -> Self;
}

impl<T> Abi<T> for T {
    #[inline]
    fn into_host(self) -> T {
        self
    }

    #[inline]
    fn from_host(host: T) -> Self {
        host
    }
}

macro_rules! identity_impls {
    () => {};
    ($type:ty $(as $cast:ty $(as $castcast:ty)?)? $(,$other:ty $(as $cast_other:ty $(as $castcast_other:ty)?)?)* $(,)?) => {
        identity_impls!(@ $type $(as $cast $(as $castcast)?)?);
        identity_impls!($($other $(as $cast_other $(as $castcast_other)?)?,)*);
    };
    (@ $type:ty) => {
        impl $crate::Parameter for $type {
            type Host = $type;
            type Abi = $type;

            #[inline]
            fn into_abi(self) -> Self::Abi {
                self
            }

            #[inline]
            fn from_abi(abi: Self::Abi) -> Self {
                abi
            }
        }
    };
    (@ $type:ty as $cast:ty) => {
        impl $crate::Parameter for $type {
            type Host = $type;
            type Abi = $cast;

            #[inline]
            fn into_abi(self) -> Self::Abi {
                self as $cast
            }

            #[inline]
            fn from_abi(abi: Self::Abi) -> Self {
                abi as $type
            }
        }

        impl Abi<$type> for $cast {
            #[inline]
            fn into_host(self) -> $type {
                self as $type
            }

            #[inline]
            fn from_host(host: $type) -> Self {
                host as Self
            }
        }
    };
    (@ $type:ty as $cast:ty as $castcast:ty) => {
        impl $crate::Parameter for $type {
            type Host = $type;
            type Abi = $castcast;

            #[inline]
            fn into_abi(self) -> Self::Abi {
                self as $cast as $castcast
            }

            #[inline]
            fn from_abi(abi: Self::Abi) -> Self {
                abi as $cast as $type
            }
        }

        impl Abi<$type> for $castcast {
            #[inline]
            fn into_host(self) -> $type {
                self as $type
            }

            #[inline]
            fn from_host(host: $type) -> Self {
                host as Self
            }
        }
    };
}

identity_impls!(
    u8 as u32,
    u16 as u32,
    u32,
    u64,
    i8 as i32,
    i16 as i32,
    i32,
    i64,
    *const u8 as usize as u32,
    *const u16 as usize as u32,
    *const u32 as usize as u32,
    *const u64 as usize as u32,
    *const i8 as usize as u32,
    *const i16 as usize as u32,
    *const i32 as usize as u32,
    *const i64 as usize as u32,
    *mut u8 as usize as u32,
    *mut u16 as usize as u32,
    *mut u32 as usize as u32,
    *mut u64 as usize as u32,
    *mut i8 as usize as u32,
    *mut i16 as usize as u32,
    *mut i32 as usize as u32,
    *mut i64 as usize as u32,
);

/// A FFI-safe `&str` equivalent passed from the guest side.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GuestStringView {
    len: u32,
    ptr: u32,
}

impl GuestStringView {
    /// Creates a new [`GuestStringView`] instance.
    pub fn new(s: &str) -> Self {
        Self {
            len: s.len().try_into().expect("len overflow"),
            ptr: (s.as_ptr() as usize).try_into().expect("ptr overflow"),
        }
    }

    /// Constructs a `&str`, a reference of string slice from this view.
    #[cfg(feature = "rt-wasmtime")]
    pub fn as_slice<T>(self, memory: Memory, store: &Store<T>) -> &str {
        // SAFETY: this is safe because the wasmtime API asserts the `store` owns the `memory` and
        // the lifetime of the slice reference is bounded by the store's lifetime.
        unsafe {
            std::str::from_utf8_unchecked(
                memory
                    .data(store)
                    .get(
                        usize::try_from(self.ptr).expect("range from overflow")
                            ..usize::try_from(self.ptr + self.len).expect("range to overflow"),
                    )
                    .expect("memory range out of bounds"),
            )
        }
    }
}

impl<'a> Parameter for &'a str {
    type Host = GuestStringView;
    type Abi = GuestStringView;

    fn into_abi(self) -> Self::Abi {
        GuestStringView::new(self)
    }

    fn from_abi(_abi: Self::Abi) -> Self {
        unreachable!("host created an instance of GuestStringView")
    }
}

/// A FFI-safe `&[u8]` equivalent passed from the guest side.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GuestMemoryView {
    len: u32,
    ptr: u32,
}

impl GuestMemoryView {
    /// Creates a new [`GuestMemoryView`] instance.
    pub fn new(s: &[u8]) -> Self {
        Self {
            len: s.len().try_into().expect("len overflow"),
            ptr: (s.as_ptr() as usize).try_into().expect("ptr overflow"),
        }
    }

    /// Constructs a `&[u8]`, a reference of u8 slice from this view.
    #[cfg(feature = "rt-wasmtime")]
    pub fn as_str<T>(self, memory: Memory, store: &Store<T>) -> &[u8] {
        // safety:
        memory
            .data(store)
            .get(
                usize::try_from(self.ptr).expect("range from overflow")
                    ..usize::try_from(self.ptr + self.len).expect("range to overflow"),
            )
            .expect("memory range out of bounds")
    }
}

impl<'a> Parameter for &'a [u8] {
    type Host = GuestMemoryView;
    type Abi = GuestMemoryView;

    fn into_abi(self) -> Self::Abi {
        GuestMemoryView::new(self)
    }

    fn from_abi(_abi: Self::Abi) -> Self {
        unreachable!("host created an instance of GuestMemoryView")
    }
}

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
