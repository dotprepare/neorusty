use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemFn};

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    quote! {
        impl neorusty_core::Event for #name {}
    }
    .into()
}

#[proc_macro_attribute]
pub fn neoforge_mod(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn neorusty_plugin_entry() -> Box<dyn neorusty_plugin_system::Plugin> {
            #input_fn
            (#fn_name)()
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn event_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
