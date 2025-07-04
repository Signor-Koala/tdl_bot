use indexmap::IndexMap;
use serde::Deserialize;
use serenity::all::{ChannelId, RoleId};

#[derive(Deserialize, Debug, PartialEq)]
pub struct RoleConfig {
    pub choices: Vec<RoleChoice>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct RoleChoice {
    pub message: String,
    pub options: IndexMap<String, RoleButton>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct RoleButton {
    pub emoji: String,
    pub label: String,
    pub role_id: RoleId,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PurgeTimerConfig {
    pub channel_id: ChannelId,
    pub time: toml::value::Datetime,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ModMailConfig {
    pub channel_id: ChannelId,
    pub mod_role: RoleId,
}

impl RoleConfig {
    pub fn from_config(config: &str) -> Self {
        toml::from_str(config).unwrap()
    }
}

impl PurgeTimerConfig {
    pub fn from_config(config: &str) -> Self {
        toml::from_str(config).unwrap()
    }
}

impl ModMailConfig {
    pub fn from_config(config: &str) -> Self {
        toml::from_str(config).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_test() {
        let config = "
[[choices]]
message = \"Choose type 1\"
[choices.options]
    t1_one = { emoji = \"emoji_1\", label = \"label_1\", role_id = 1 }
    t1_two = { emoji = \"emoji_2\", label = \"label_2\", role_id = 2 }

[[choices]]
message = \"Choose type 2\"
[choices.options]
    t2_one = { emoji = \"emoki_1\", label = \"label_3\", role_id = 3}
";

        assert_eq!(
            RoleConfig::from_config(config),
            RoleConfig {
                choices: vec![
                    RoleChoice {
                        message: String::from("Choose type 1"),
                        options: IndexMap::from([
                            (
                                String::from("t1_one"),
                                RoleButton {
                                    emoji: String::from("emoji_1"),
                                    label: String::from("label_1"),
                                    role_id: RoleId::from(1),
                                }
                            ),
                            (
                                String::from("t1_two"),
                                RoleButton {
                                    emoji: String::from("emoji_2"),
                                    label: String::from("label_2"),
                                    role_id: RoleId::from(2),
                                }
                            )
                        ]),
                    },
                    RoleChoice {
                        message: String::from("Choose type 2"),
                        options: IndexMap::from([(
                            String::from("t2_one"),
                            RoleButton {
                                emoji: String::from("emoki_1"),
                                label: String::from("label_3"),
                                role_id: RoleId::from(3),
                            }
                        ),]),
                    }
                ]
            }
        )
    }
}
