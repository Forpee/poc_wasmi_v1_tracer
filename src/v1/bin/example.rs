//  The `wasmi` virtual machine definitions.

//  These closely mirror the WebAssembly specification definitions.
//  The overall structure is heavily inspired by the `wasmtime` virtual
//  machine architecture.

//  # Example

//  The following example shows a "Hello, World!"-like example of creating
//  a Wasm module from some initial `.wat` contents, defining a simple host
//  function and calling the exported Wasm function.

//  The example was inspired by
//  [Wasmtime's API example](https://docs.rs/wasmtime/0.39.1/wasmtime/).

//  use anyhow::{anyhow, Result};
//  use wasmi::*;

// fn main() -> Result<()> {
//     // First step is to create the Wasm execution engine with some config.
//     // In this example we are using the default configuration.
//     let engine = Engine::default();
//     let wat = r#"
//         (module
//             (import "host" "hello" (func $host_hello (param i32)))
//             (func (export "hello")
//                 (call $host_hello (i32.const 3))
//             )
//         )
//     "#;
//     // Wasmi does not yet support parsing `.wat` so we have to convert
//     // out `.wat` into `.wasm` before we compile and validate it.
//     let wasm = wat::parse_str(&wat)?;
//     let module = Module::new(&engine, &mut &wasm[..])?;

//     // All Wasm objects operate within the context of a `Store`.
//     // Each `Store` has a type parameter to store host-specific data,
//     // which in this case we are using `42` for.
//     type HostState = u32;
//     let mut store = Store::new(&engine, 42);
//     let host_hello = Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
//         println!("Got {param} from WebAssembly");
//         println!("My host state is: {}", caller.host_data());
//     });

//     // In order to create Wasm module instances and link their imports
//     // and exports we require a `Linker`.
//     let mut linker = <Linker<HostState>>::new();
//     // Instantiation of a Wasm module requires defning its imports and then
//     // afterwards we can fetch exports by name, as well as asserting the
//     // type signature of the function with `get_typed_func`.
//     //
//     // Also before using an instance created this way we need to start it.
//     linker.define("host", "hello", host_hello)?;
//     let instance = linker
//         .instantiate(&mut store, &module)?
//         .start(&mut store)?;
//     let hello = instance
//         .get_export(&store, "hello")
//         .and_then(Extern::into_func)
//         .ok_or_else(|| anyhow!("could not find function \"hello\""))?
//         .typed::<(), (), _>(&mut store)?;

//     // And finally we can call the wasm!
//     hello.call(&mut store, ())?;

use std::{cell::RefCell, fs::File, rc::Rc};

//     Ok(())
// }
use anyhow::{anyhow, Result};
use wabt;
use wasmi::{
    v1::{Caller, Engine, Extern, Func, Linker, Module, Store, Tracer},
    RuntimeValue,
};

fn load_from_file(filename: &str) -> Vec<u8> {
    use std::io::prelude::*;
    let mut file = File::open(filename).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

pub fn main() -> Result<()> {
    let engine = Engine::default();
    let tracer = Tracer::new();
    let tracer = Rc::new(RefCell::new(tracer));

    //     let wat = r#"
    //     (module
    //         (import "host" "hello" (func $host_hello (param i32)))
    //         (func (export "hello")
    //             (call $host_hello (i32.const 3))
    //         )
    //     )
    // "#;
    //     let wat = r#"
    //         (module
    //             (func (export "main") (result i32)
    //                 i32.const 100
    //                 i32.const 20
    //                 i32.add
    //                 i32.const 100
    //                 i32.add
    //             )
    //         )
    // "#;

    // let wasm = wabt::wat2wasm(wat)?;
    let wasm = load_from_file("src/v1/tests/test_rust.wasm");
    let module = Module::new(&engine, wasm)?;

    type HostState = u32;
    let mut store = Store::new(&engine, 42);
    let host_hello = Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
        println!("Got {param} from WebAssembly");
        println!("My host state is: {}", caller.host_data());
    });

    let mut linker = <Linker<HostState>>::new();

    linker.define("host", "main", host_hello)?;
    let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;

    let hello = instance
        .get_export(&store, "main")
        .and_then(Extern::into_func)
        .ok_or_else(|| anyhow!("could not find function \"hello\""))?;

    hello.call_with_trace(&mut store, &[], &mut [], tracer.clone())?;
    // hello.call(&mut store, &[], &mut [])?;
    println!("{:?}", (*tracer).borrow());
    Ok(())
}
