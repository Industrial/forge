//! E2E test for Forge i18n (015): locales layout, GET / (Inertia Home with locale prop), Accept-Language.
//! Run via `bin/test-e2e`. One test: layout, locale files, en-US and de in Inertia page props.

use std::time::Duration;

use forge_e2e_lib::cli;

/// Single E2E test: prebuilt layout, locales dir with en-US and de, GET / returns Inertia HTML
/// with locale prop from Accept-Language.
#[tokio::test]
async fn e2e_prebuilt_i18n_layout_and_accept_language() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let app_root = project_root.join("crates/app");
  let locales_en = app_root.join("locales/en-US/main.ftl");
  let locales_de = app_root.join("locales/de/main.ftl");
  assert!(
    locales_en.exists(),
    "locales/en-US/main.ftl missing at {}",
    locales_en.display()
  );
  assert!(
    locales_de.exists(),
    "locales/de/main.ftl missing at {}",
    locales_de.display()
  );

  let en_content = std::fs::read_to_string(&locales_en).expect("read en-US main.ftl");
  let de_content = std::fs::read_to_string(&locales_de).expect("read de main.ftl");
  assert!(
    en_content.contains("greeting") && en_content.contains("Hello"),
    "en-US main.ftl should define greeting with Hello; got {:?}",
    en_content
  );
  assert!(
    de_content.contains("greeting") && de_content.contains("Hallo"),
    "de main.ftl should define greeting with Hallo; got {:?}",
    de_content
  );

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .unwrap();

  // GET / with Accept-Language en-US: Inertia shell with Pages/Home and locale "en-US" in props
  let en_resp = client
    .get(base.clone())
    .header("Accept-Language", "en-US,en;q=0.9")
    .send()
    .await
    .expect("GET / en");
  assert!(en_resp.status().is_success(), "GET / (en) must succeed");
  let en_body = en_resp.text().await.expect("body");
  assert!(
    en_body.contains("id=\"app\"") || en_body.contains("id='app'"),
    "en response should contain Inertia app root; got {:?}",
    en_body
  );
  assert!(
    en_body.contains("Pages/Home") && en_body.contains("en-US"),
    "en response should contain Pages/Home and locale en-US in props; got {:?}",
    en_body
  );
  // i18n: server-translated greeting in props (en-US -> "Hello, World!")
  assert!(
    en_body.contains("Hello") && en_body.contains("World"),
    "en response should contain translated greeting in props; got {:?}",
    en_body
  );

  // GET / with Accept-Language de: locale "de" in page props
  let de_resp = client
    .get(base.clone())
    .header("Accept-Language", "de,en;q=0.9")
    .send()
    .await
    .expect("GET / de");
  assert!(de_resp.status().is_success(), "GET / (de) must succeed");
  let de_body = de_resp.text().await.expect("body");
  // In HTML, page props are escaped (e.g. &quot;de&quot;) so check for locale value in context
  assert!(
    de_body.contains("Pages/Home")
      && (de_body.contains("\"de\"") || de_body.contains("locale&quot;:&quot;de")),
    "de response should contain Pages/Home and locale de in props; got {:?}",
    de_body
  );
  // i18n: server-translated greeting in props (de -> "Hallo, World!")
  assert!(
    de_body.contains("Hallo") && de_body.contains("World"),
    "de response should contain translated greeting (Hallo) in props; got {:?}",
    de_body
  );

  // Fallback: no Accept-Language -> default locale in props
  let fallback_resp = client.get(base).send().await.expect("GET / no header");
  assert!(
    fallback_resp.status().is_success(),
    "GET / (fallback) must succeed"
  );
  let fallback_body = fallback_resp.text().await.expect("body");
  assert!(
    fallback_body.contains("Pages/Home"),
    "fallback should return Inertia Home; got {:?}",
    fallback_body
  );
}
