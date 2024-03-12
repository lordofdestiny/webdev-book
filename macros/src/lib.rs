extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::DeriveInput;


/// Derive the `From<i32>` and `FromStr` traits for types that represent a database object id.
///
/// This macro is intended to be used with types that represent an id of a database object. It
/// derives the `From<i32>` and `FromStr` traits for the type. The `From<i32>` trait allows
/// converting an `i32` to the type, and the `FromStr` trait allows parsing a string to the type.
/// String that is parsed to the type must be a valid `i32` string.
/// ```
/// use macros::DbObjectId;
///
/// #[derive(DbObjectId, Debug)]
/// struct AccountId(i32);
///
/// let id: AccountId = 1.into();
/// debug_assert_eq!(id.0, 1);
///
/// let id: AccountId = "1".parse().unwrap();
/// debug_assert_eq!(id.0, 1);
/// ```
///
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

        impl std::str::FromStr for #name {
            type Err = std::io::Error;

            fn from_str(id: &str) -> Result<Self, Self::Err> {
                id.parse()
                .map(Self)
                .map_err(
                    |_| Self::Err::new(
                        std::io::ErrorKind::InvalidInput,
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