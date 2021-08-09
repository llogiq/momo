extern crate proc_macro;

use proc_macro::TokenStream;
use watt::WasmMacro;

static WASM: &[u8] = include_bytes!("momo.wasm");
static MACRO: WasmMacro = WasmMacro::new(WASM);

#[proc_macro_attribute]
pub fn momo(attrs: TokenStream, input: TokenStream) -> TokenStream {
    MACRO.proc_macro_attribute("momo", input, attrs)
}
