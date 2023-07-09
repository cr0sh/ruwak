macro_rules! test_primitives {
    // entrypoint
    ($(($export_name:ident, fn $import_name:ident($arg:ident: $type:ty), $value:expr),)*) => {
        test_primitives!(
            @ $(($import_name, $type),)*
            @ $(($export_name, fn $import_name($arg: $type), $value),)*
        );
    };
    // termination
    (@ $(($cb_import_name:ident, $cb_param_type:ty),)* @) => {};
    // cb_import_name, cb_param_type: to 'wrap' unused functions in each test cases with
    //                                  unreachable!()
    // export_name, import_name(arg: type), value: guest export_name calls host import_name
    //                                              function with given FFI definition
    // tt: generic 'TT-muncher' pattern
    (@ $(($cb_import_name:ident, $cb_param_type:ty),)*
        @ ($export_name:ident, fn $import_name:ident($arg:ident: $type:ty), $value:expr), $($tt:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn $export_name() {
            extern "C" {
                fn $import_name($arg: $type);
            }
            unsafe { $import_name($value) };
        }

        #[cfg(not(target_arch = "wasm32"))]
        #[test]
        fn $export_name() {
            run_test::<$type>(stringify!($export_name), stringify!($import_name), $value, |linker| {
                $(
                    if stringify!($cb_import_name) != stringify!($import_name) {
                        linker
                            .func_wrap("env", stringify!($cb_import_name), move |_: $cb_param_type| unreachable!())
                            .expect("cannot wrap function");
                    }
                )*
            });
        }

        test_primitives!(@ $(($cb_import_name, $cb_param_type),)* @ $($tt)*);
    };
}

#[cfg(all(not(target_arch = "wasm32"), test))]
fn run_test<T: wasmtime::WasmTy + Eq + Send + Sync + Copy + std::fmt::Debug + 'static>(
    test_name: &'static str,
    recv_name: &'static str,
    value: T,
    func_wrap_cb: impl FnOnce(&mut wasmtime::Linker<()>),
) {
    use std::{
        path::PathBuf,
        process::{Command, Stdio},
    };

    use wasmtime::{Config, Engine, Linker, Module, Store};

    let status = Command::new("cargo")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "build",
            "-p",
            "ruwak-test",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("cannot spawn cargo");
    assert!(status.success(), "cargo build failed");

    let engine = Engine::new(&Config::new()).unwrap();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned();
    path.push("target/wasm32-unknown-unknown/debug/ruwak_test.wasm");
    let module = Module::from_file(&engine, path).expect("cannot load module");
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);
    linker
        .func_wrap("env", recv_name, move |x: T| assert_eq!(x, value))
        .expect("cannot wrap function");
    (func_wrap_cb)(&mut linker);
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("cannot instantiate module");
    instance
        .get_typed_func::<(), ()>(&mut store, test_name)
        .expect("cannot get function")
        .call(store, ())
        .expect("cannot call test");
}

test_primitives!(
    // (send_u8, fn receive_u8(arg: u8), 42),
    // (send_u16, fn receive_u16(arg: u16), 42),
    (send_u32, fn receive_u32(arg: u32), 42),
    (send_u64, fn receive_u64(arg: u64), 42),
    // (send_i8, fn receive_i8(arg: i8), 42),
    // (send_i16, fn receive_i16(arg: i16), 42),
    (send_i32, fn receive_i32(arg: i32), 42),
    (send_i64, fn receive_i64(arg: i64), 42),
    // (send_const_i8, fn receive_const_i8(arg: *const i8), 42 as *const i8),
    // (send_const_i16, fn receive_const_i16(arg: *const i16), 42 as *const i16),
    // (send_const_i32, fn receive_const_i32(arg: *const i32), 42 as *const i32),
    // (send_const_i64, fn receive_const_i64(arg: *const i64), 42 as *const i64),
    // (send_mut_i8, fn receive_mut_i8(arg: *mut i8), 42 as *mut i8),
    // (send_mut_i16, fn receive_mut_i16(arg: *mut i16), 42 as *mut i16),
    // (send_mut_i32, fn receive_mut_i32(arg: *mut i32), 42 as *mut i32),
    // (send_mut_i64, fn receive_mut_i64(arg: *mut i64), 42 as *mut i64),
);
