use raikou_window::config::WindowConfig;

#[test]
fn default_title_is_non_empty() {
    assert!(!WindowConfig::default().title.is_empty());
}

#[test]
fn default_initial_size_is_positive() {
    let config = WindowConfig::default();
    assert!(config.initial_size.0 > 0.0);
    assert!(config.initial_size.1 > 0.0);
}

#[test]
fn default_minimum_size_is_none_or_not_larger_than_initial_size() {
    let config = WindowConfig::default();
    if let Some((min_width, min_height)) = config.minimum_size {
        assert!(min_width <= config.initial_size.0);
        assert!(min_height <= config.initial_size.1);
    }
}

#[test]
fn default_flags_match_expected_bootstrap_defaults() {
    let config = WindowConfig::default();
    assert!(config.resizable);
    assert!(config.decorations);
    assert!(!config.transparency);
}

#[test]
fn builder_round_trip_sets_all_fields() {
    let config = WindowConfig::default()
        .title("Chained")
        .initial_size(800.0, 600.0)
        .minimum_size(200.0, 150.0)
        .resizable(false)
        .decorations(false)
        .transparency(true);

    assert_eq!(config.title, "Chained");
    assert_eq!(config.initial_size, (800.0, 600.0));
    assert_eq!(config.minimum_size, Some((200.0, 150.0)));
    assert!(!config.resizable);
    assert!(!config.decorations);
    assert!(config.transparency);
}

#[test]
fn config_is_clone_and_debug() {
    let original = WindowConfig::default().title("Original");
    let cloned = original.clone();
    assert_eq!(original.title, cloned.title);
    let _ = format!("{original:?}");
}
