//! Server-side i18n: fluent_templates lookup for Inertia page props (e.g. greeting on Home).

use fluent_templates::{fluent_bundle::FluentValue, Loader, static_loader};
use std::borrow::Cow;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en-US",
    };
}

/// Look up the greeting string for the given locale. Fails if locale or translation is missing.
#[allow(dead_code)]
pub fn greeting(locale: &str) -> String {
    let lang: LanguageIdentifier = locale
        .parse()
        .unwrap_or_else(|_| panic!("invalid locale: {}", locale));
    let mut args = HashMap::new();
    args.insert(Cow::Borrowed("name"), FluentValue::String(Cow::Borrowed("World")));
    LOCALES
        .try_lookup_with_args(&lang, "greeting", &args)
        .unwrap_or_else(|| panic!("missing translation 'greeting' for locale {}", locale))
}
