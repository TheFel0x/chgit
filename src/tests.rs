use std::collections::HashMap;

use crate::{Config, Profile};

#[test]
fn config_round_trips() {
    let mut config = Config {
        profiles: HashMap::new(),
        auto_paths: HashMap::new(),
    };
    config.profiles.insert(
        "test".to_string(),
        Profile {
            user_name: "John".to_string(),
            email: "john@example.com".to_string(),
            ssh_key: "~/.ssh/id_rsa".to_string(),
        },
    );
    config
        .auto_paths
        .insert("/home/user/projects".to_string(), "test".to_string());

    let serialized = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&serialized).unwrap();

    assert_eq!(deserialized.profiles["test"].user_name, "John");
    assert_eq!(deserialized.profiles["test"].email, "john@example.com");
    assert_eq!(deserialized.profiles["test"].ssh_key, "~/.ssh/id_rsa");
    assert_eq!(deserialized.auto_paths["/home/user/projects"], "test");
}
