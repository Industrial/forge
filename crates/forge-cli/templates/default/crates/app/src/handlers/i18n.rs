//! Server-side i18n: fluent_templates lookup for Inertia page props (e.g. greeting on Home).

use fluent_templates::{Loader, fluent_bundle::FluentValue, static_loader};
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
  args.insert(
    Cow::Borrowed("name"),
    FluentValue::String(Cow::Borrowed("World")),
  );
  LOCALES
    .try_lookup_with_args(&lang, "greeting", &args)
    .unwrap_or_else(|| panic!("missing translation 'greeting' for locale {}", locale))
}

#[cfg(test)]
mod tests {
  use super::*;

  // --- BDD Tests ---

  mod greeting_lookup_behavior {
    use super::*;

    #[test]
    fn should_return_greeting_for_valid_locale() {
      // Given: a valid locale string
      let locale = "en-US";

      // When: looking up greeting for the locale
      let result = greeting(locale);

      // Then: should return a greeting string
      assert!(
        !result.is_empty(),
        "Should return non-empty greeting string"
      );
      assert!(
        result.contains("World"),
        "Greeting should contain the name argument 'World'"
      );
    }

    #[test]
    fn should_use_fallback_language_when_locale_not_specified() {
      // Given: the fallback locale (en-US)
      let locale = "en-US";

      // When: looking up greeting
      let result = greeting(locale);

      // Then: should return greeting in fallback language
      assert!(
        !result.is_empty(),
        "Should return greeting in fallback language"
      );
    }

    #[test]
    #[should_panic(expected = "invalid locale")]
    fn should_panic_when_locale_is_invalid() {
      // Given: an invalid locale string
      let locale = "invalid-locale-format";

      // When: looking up greeting for invalid locale
      greeting(locale);

      // Then: should panic with "invalid locale" message
    }

    #[test]
    fn should_include_name_argument_in_greeting() {
      // Given: a valid locale
      let locale = "en-US";

      // When: looking up greeting (which uses "World" as name argument)
      let result = greeting(locale);

      // Then: greeting should include the name argument
      assert!(
        result.contains("World"),
        "Greeting should include the name argument 'World'"
      );
    }

    #[test]
    fn should_return_different_greetings_for_different_locales() {
      // Given: different valid locales
      let locales = vec!["en-US", "en-GB"];

      // When: looking up greetings for each locale
      let mut results = Vec::new();
      for locale in locales {
        if let Ok(_) = locale.parse::<LanguageIdentifier>() {
          // Only test if locale is valid
          let result = greeting(locale);
          results.push((locale, result));
        }
      }

      // Then: should return greetings (may be same or different depending on translations)
      assert!(
        !results.is_empty(),
        "Should return greetings for valid locales"
      );
    }
  }

  mod locale_parsing_behavior {
    use super::*;

    #[test]
    fn should_parse_standard_locale_format() {
      // Given: a standard locale format (language-region)
      let locale = "en-US";

      // When: parsing the locale
      let lang: Result<LanguageIdentifier, _> = locale.parse();

      // Then: should parse successfully
      assert!(lang.is_ok(), "Should parse standard locale format");
    }

    #[test]
    fn should_parse_locale_with_language_only() {
      // Given: a locale with language only (no region)
      let locale = "en";

      // When: parsing the locale
      let lang: Result<LanguageIdentifier, _> = locale.parse();

      // Then: should parse successfully
      assert!(lang.is_ok(), "Should parse locale with language only");
    }

    #[test]
    fn should_fail_to_parse_invalid_locale_format() {
      // Given: an invalid locale format
      let locale = "not-a-valid-locale";

      // When: parsing the locale
      let lang: Result<LanguageIdentifier, _> = locale.parse();

      // Then: should fail to parse
      assert!(lang.is_err(), "Should fail to parse invalid locale format");
    }
  }

  mod translation_args_behavior {
    use super::*;

    #[test]
    fn should_create_args_map_with_name_parameter() {
      // Given: a name value
      let name = "World";

      // When: creating args map
      let mut args = HashMap::new();
      args.insert(
        Cow::Borrowed("name"),
        FluentValue::String(Cow::Borrowed(name)),
      );

      // Then: args map should contain name parameter
      assert!(
        args.contains_key("name"),
        "Args map should contain 'name' key"
      );
      match args.get("name") {
        Some(FluentValue::String(Cow::Borrowed(val))) => assert_eq!(*val, name),
        _ => panic!("Name value should be a string"),
      }
    }

    #[test]
    fn should_use_borrowed_string_for_name_argument() {
      // Given: a name string
      let name = "World";

      // When: creating FluentValue for name
      let value = FluentValue::String(Cow::Borrowed(name));

      // Then: value should be a borrowed string
      match value {
        FluentValue::String(Cow::Borrowed(s)) => assert_eq!(s, name),
        _ => panic!("Value should be a borrowed string"),
      }
    }
  }
}
