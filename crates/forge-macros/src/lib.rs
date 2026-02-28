use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ForgeAuthUser)]
pub fn derive_auth_user(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = input.ident;

  let expanded = quote! {
      impl ::forge::axum_login::AuthUser for #name {
          type Id = ::forge::uuid::Uuid;

          fn id(&self) -> Self::Id {
              self.id
          }

          fn session_auth_hash(&self) -> &[u8] {
              self.password_hash.as_bytes()
          }
      }
  };

  TokenStream::from(expanded)
}
