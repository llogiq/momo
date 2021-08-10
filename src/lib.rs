#![doc = include_str!("../README.md")]
use proc_macro::TokenStream;
use watt::WasmMacro;

static WASM: &[u8] = include_bytes!("momo.wasm");
static MACRO: WasmMacro = WasmMacro::new(WASM);

#[proc_macro_attribute]
/// Generate lightweight monomorphized wrapper around main implementation.
/// May be applied to functions and methods.
pub fn momo(attrs: TokenStream, input: TokenStream) -> TokenStream {
    MACRO.proc_macro_attribute("momo", input, attrs)
}
