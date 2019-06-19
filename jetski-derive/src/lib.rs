extern crate proc_macro;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::Result;


struct LispPattern {

}

impl Parse for LispPattern {
    fn parse(input: ParseStream) -> Result<Self> {
        unimplemented!()
    }
}


#[proc_macro]
pub fn hello(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    //let input = proc_macro2::TokenStream::from(item);
    //input.into()
    let output = quote! { };
    output.into()
}
