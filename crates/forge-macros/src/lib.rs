use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(ForgeAuthUser)]
pub fn derive_auth_user(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  TokenStream::from(expand_forge_auth_user(input))
}

#[proc_macro_derive(ForgeScoped, attributes(forge_scoped))]
pub fn derive_forge_scoped(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  TokenStream::from(expand_forge_scoped(input))
}

/// Expansion logic for `#[derive(ForgeAuthUser)]`. Exposed for tests.
fn expand_forge_auth_user(input: DeriveInput) -> proc_macro2::TokenStream {
  let name = input.ident;
  quote! {
      impl ::forge::axum_login::AuthUser for #name {
          type Id = ::forge::uuid::Uuid;

          fn id(&self) -> Self::Id {
              self.id
          }

          fn session_auth_hash(&self) -> &[u8] {
              self.password_hash.as_bytes()
          }
      }
  }
}

/// Expansion logic for `#[derive(ForgeScoped)]`. Exposed for tests.
fn expand_forge_scoped(input: DeriveInput) -> proc_macro2::TokenStream {
  let _name = input.ident;

  // Default column is organization_id
  let mut scope_column = "organization_id".to_string();

  // Parse #[forge_scoped(column_name)]
  for attr in &input.attrs {
    if attr.path().is_ident("forge_scoped")
      && let Ok(nested) = attr.parse_args::<syn::Ident>()
    {
      scope_column = nested.to_string();
    }
  }

  let column_ident = quote::format_ident!("{}", to_pascal_case(&scope_column));

  quote! {
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
  }
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

#[cfg(test)]
mod tests {
  use super::*;
  use syn::parse_str;

  #[test]
  fn to_pascal_case_snake() {
    assert_eq!(to_pascal_case("organization_id"), "OrganizationId");
  }

  #[test]
  fn to_pascal_case_single_word() {
    assert_eq!(to_pascal_case("id"), "Id");
  }

  #[test]
  fn to_pascal_case_empty() {
    assert_eq!(to_pascal_case(""), "");
  }

  #[test]
  fn to_pascal_case_multiple_underscores() {
    assert_eq!(to_pascal_case("foo_bar_baz"), "FooBarBaz");
  }

  #[test]
  fn expand_forge_auth_user_produces_expected_tokens() {
    let input: DeriveInput = parse_str("struct User { id: Uuid, password_hash: String }")
      .expect("valid struct");
    let out = expand_forge_auth_user(input);
    let s = out.to_string();
    assert!(s.contains("AuthUser"), "expansion should implement AuthUser");
    assert!(s.contains("id"), "expansion should have id()");
    assert!(s.contains("session_auth_hash"), "expansion should have session_auth_hash()");
    assert!(s.contains("password_hash"), "expansion should use password_hash");
  }

  #[test]
  fn expand_forge_scoped_default_column() {
    let input: DeriveInput = parse_str("struct Entity {}").expect("valid struct");
    let out = expand_forge_scoped(input);
    let s = out.to_string();
    assert!(s.contains("ForgeScoped"), "expansion should implement ForgeScoped");
    assert!(s.contains("OrganizationId"), "default column should be organization_id -> OrganizationId");
    assert!(s.contains("scoped"), "expansion should have scoped method");
  }

  #[test]
  fn expand_forge_scoped_custom_column() {
    let input: DeriveInput = parse_str(r#"
      #[forge_scoped(tenant_id)]
      struct Entity {}
    "#)
    .expect("valid struct");
    let out = expand_forge_scoped(input);
    let s = out.to_string();
    assert!(s.contains("TenantId"), "custom column tenant_id -> TenantId");
  }

  /// When #[forge_scoped(...)] fails to parse as Ident, default column is used.
  #[test]
  fn expand_forge_scoped_invalid_attr_falls_back_to_default() {
    let input: DeriveInput = parse_str(r#"
      #[forge_scoped("not_an_ident")]
      struct Entity {}
    "#)
    .expect("valid struct");
    let out = expand_forge_scoped(input);
    let s = out.to_string();
    assert!(s.contains("OrganizationId"), "fallback to default when attr parse fails");
  }
}
