extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

#[proc_macro_attribute]
pub fn mqtt_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let input = parse_macro_input!(item as ItemFn);

    let mut topic = None;
    for arg in args {
        if let NestedMeta::Meta(Meta::NameValue(nv)) = arg {
            if nv.path.is_ident("topic") {
                if let Lit::Str(lit) = nv.lit {
                    topic = Some(lit.value());
                }
            }
        }
    }

    let topic = topic.expect("expected a topic");

    let fn_name = &input.sig.ident;
    let register_fn_name = format_ident!("__register_{}", fn_name);

    let expanded = quote! {
        #input

        #[ctor::ctor]
        fn #register_fn_name() {
            crate::handlers::register_handler(
                #topic,
                Box::new(
                    move |current_task_handle, config, topic, payload|
                        Box::pin(#fn_name(current_task_handle, config, topic, payload)),
                ),
            );
        }
    };

    TokenStream::from(expanded)
}
