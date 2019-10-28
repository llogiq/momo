extern crate proc_macro;

use proc_macro::TokenStream;

static WASM: &[u8] = include_bytes!("momo.wasm");

#[proc_macro_attribute]
pub fn momo(attrs: TokenStream, input: TokenStream) -> TokenStream {
        watt::proc_macro_attribute("momo", input, attrs, WASM)
}

