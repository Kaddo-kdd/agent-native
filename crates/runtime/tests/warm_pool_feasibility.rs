//! AN-064 feasibility evidence: a live Wasmtime instance retains guest state.
//!
//! This is intentionally outside the public runtime API. It shows why a generic
//! warm pool cannot reuse the current Core WASM instances: there is no generic,
//! verifiable reset for their globals, memory, tables, and host resources.

#[test]
fn reusing_a_live_instance_retains_mutable_guest_state() {
    let engine = wasmtime::Engine::default();
    let wasm = wat::parse_str(
        r#"(module
            (global $counter (mut i32) (i32.const 0))
            (func (export "increment") (result i32)
                global.get $counter
                i32.const 1
                i32.add
                global.set $counter
                global.get $counter)
        )"#,
    )
    .expect("encode stateful module");
    let module = wasmtime::Module::new(&engine, wasm).expect("compile stateful module");
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new(&mut store, &module, &[]).expect("instantiate");
    let increment = instance
        .get_typed_func::<(), i32>(&mut store, "increment")
        .expect("increment export");

    assert_eq!(increment.call(&mut store, ()).expect("first call"), 1);
    assert_eq!(
        increment
            .call(&mut store, ())
            .expect("reused instance call"),
        2,
        "a reused instance leaks the prior invocation's mutable global"
    );

    let mut clean_store = wasmtime::Store::new(&engine, ());
    let clean_instance =
        wasmtime::Instance::new(&mut clean_store, &module, &[]).expect("fresh instantiate");
    let clean_increment = clean_instance
        .get_typed_func::<(), i32>(&mut clean_store, "increment")
        .expect("increment export in fresh instance");
    assert_eq!(
        clean_increment
            .call(&mut clean_store, ())
            .expect("fresh call"),
        1,
        "a fresh Store and instance restore the ADR-005 baseline"
    );
}
