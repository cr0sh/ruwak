#[cfg(feature = "rt-wasmtime")]
use wasmtime::{Memory, Store};

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

identity_impls!(
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    *const u8,
    *const u16,
    *const u32,
    *const u64,
    *const i8,
    *const i16,
    *const i32,
    *const i64,
    *mut u8,
    *mut u16,
    *mut u32,
    *mut u64,
    *mut i8,
    *mut i16,
    *mut i32,
    *mut i64,
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

    fn from_host(_host: Self::Host) -> Self {
        unreachable!("host created an instance of GuestStringView")
    }

    fn into_host(self) -> Self::Host {
        GuestStringView::new(self)
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

    fn from_host(_host: Self::Host) -> Self {
        unreachable!("host created an instance of GuestMemoryView")
    }

    fn into_host(self) -> Self::Host {
        GuestMemoryView::new(self)
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
