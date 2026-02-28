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

#[proc_macro_derive(ForgeScoped, attributes(forge_scoped))]
pub fn derive_forge_scoped(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let _name = input.ident;

  // Default column is organization_id
  let mut scope_column = "organization_id".to_string();

  // Parse #[forge_scoped(column_name)]
  for attr in &input.attrs {
    if attr.path().is_ident("forge_scoped") {
      if let Ok(nested) = attr.parse_args::<syn::Ident>() {
        scope_column = nested.to_string();
      }
    }
  }

  let column_ident = quote::format_ident!("{}", to_pascal_case(&scope_column));

  let expanded = quote! {
      impl ::forge::authz::ForgeScoped<Entity> for ::forge::sea_orm::Select<Entity> {
          fn scoped<C: ::forge::authz::AuthzContext>(self, context: &C) -> ::forge::sea_orm::Select<Entity> {
              use ::forge::sea_orm::ColumnTrait;
              if let Some(org_id) = context.organization_id() {
                  self.filter(Column::#column_ident.eq(org_id))
              } else {
                  // Ghost Mode: If no org context, return nothing by default
                  self.filter(::forge::sea_orm::Condition::all().add(::forge::sea_orm::Expr::val(1).eq(0)))
              }
          }
      }
  };

  TokenStream::from(expanded)
}

/// Converts a snake_case string to PascalCase.
fn to_pascal_case(s: &str) -> String {
  let mut res = String::new();
  let mut capitalize = true;
  for c in s.chars() {
    if c == '_' {
      capitalize = true;
    } else if capitalize {
      res.push(c.to_uppercase().next().unwrap());
      capitalize = false;
    } else {
      res.push(c);
    }
  }
  res
}
