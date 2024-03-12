extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(DbObjectId)]
pub fn derive_db_object_id_fn(_item: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(_item).unwrap();
    let name = &ast.ident;
    TokenStream::from(quote!(
        impl From<i32> for #name {
            fn from(id: i32) -> Self {
                Self(id)
            }
        }

        impl FromStr for #name {
            type Err = std::io::Error;

            fn from_str(id: &str) -> Result<Self, Self::Err> {
                id.parse()
                .map(Self)
                .map_err(
                    |_| Self::Err::new(
                        ErrorKind::InvalidInput,
                        if id.is_empty() {
                            "No id provided"
                        } else {
                            "Invalid id format"
                        }
                    ))
            }
        }
    ))
}