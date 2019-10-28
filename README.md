# momo

### Keep your compile time during MOnoMOrphization

This is a `proc_macro` crate to help keeping the code footprint of
generic methods in check. Often, generics are used in libraries to
improve ergonomics.  However,  this has a cost in compile time and
binary size.  Optimally,  one creates a  small shell function that
does the generic conversions and then calls an inner function, but
that makes the code less readable.

Add a `#[momo]`  annotation from this crate to split your function
into an outer conversion and a private inner function.  In return,
you get some compile time for a tiny bit of runtime  (if at all) –
without impairing readability.

For now, the only place where we can put the `#[momo]` annotations
is on plain functions.

This new updated version usesi D. Tolnay's [watt] runtime to speed
up the compile time, which was negatively affected with proc macro
baggage. Rebuilding the wasm can be done with the commands:

```bash
cd wasm
cargo build --release --target wasm32-unknown-unknown
cp wasm/target/wasm32-unknown-unknown/release/momo.wasm ../src
```

You might need to add the  `wasm32-unknown-unknown` target to your
Rust toolchain.

[watt]: https://github.com/dtolnay/watt
