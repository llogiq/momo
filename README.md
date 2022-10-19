# momo
**Keep your compile time during MOnoMOrphization**
[![badge](https://docs.rs/momo/badge.svg)](https://docs.rs/momo)

This is a `proc_macro` crate to help keeping the code footprint of
generic methods in check. Often, generics are used in libraries to
improve ergonomics.  However,  this has a cost in compile time and
binary size.  Optimally,  one creates a  small shell function that
does the generic conversions and then calls an inner function, but
that makes the code less readable.

Add a `#[momo]`  annotation from this crate to split your function
into an outer conversion and a private inner function.  In return,
you get some compile time for a tiny bit of runtime (if at all) –
without impairing readability.

Conversions currently supported are `Into` (`.into()`), `AsRef`
(`.as_ref()`), and `AsMut` (`.as_mut()`).  See `enum Conversions`
in code.


## Notes on watt

This new updated version uses D. Tolnay's [watt] runtime to speed
up the compile time, which was negatively affected with proc macro
baggage.

The main crate uses a pre-built wasm containing the tagged version.
Rebuilding the wasm can be done with the commands:

```bash
cd wasm

cargo +nightly build \
    --release \
    --target wasm32-unknown-unknown \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort

# If wasm-opt is unavailable, copying the file is fine.
wasm-opt target/wasm32-unknown-unknown/release/momo_watt.wasm -Oz \
--strip-debug --simplify-globals --vacuum -o ../src/momo.wasm
```

You might need to add the  `wasm32-unknown-unknown` target to your
Rust toolchain.

[watt]: https://github.com/dtolnay/watt

(If you are tagging a new version, remember to commit the new `wasm` file.
Also change the versions in both `Cargo.toml` files.)

## Debugging the macro

The [cargo-expand] tool may be used to expand the output of macro expansion,
including from this proc-macro.  To examine the results of the example file,
use `cargo expand --example check`.

[cargo-expand]: https://github.com/dtolnay/cargo-expand
