use indexmap::IndexMap;
use serde::Deserialize;
use serenity::all::RoleId;

#[derive(Deserialize, Debug, PartialEq)]
pub struct RoleChoices {
    pub message: String,
    pub options: IndexMap<String, RoleButton>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct RoleButton {
    pub emoji: String,
    pub label: String,
    pub role_id: RoleId,
}

pub fn get_role_choices(config: &str) -> Vec<RoleChoices> {
    let role_choices_map: IndexMap<String, RoleChoices> = toml::from_str(config).unwrap();
    role_choices_map.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_test() {
        let config = "
[Type1]
message = \"Choose type 1\"
[Type1.options]
    t1_one = { emoji = \"emoji_1\", label = \"label_1\", role_id = 1 }
    t1_two = { emoji = \"emoji_2\", label = \"label_2\", role_id = 2 }

[Type2]
message = \"Choose type 2\"
[Type2.options]
    t2_one = { emoji = \"emoki_1\", label = \"label_3\", role_id = 3}
";

        assert_eq!(
            get_role_choices(config),
            vec![
                RoleChoices {
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
                RoleChoices {
                    message: String::from("Choose type 2"),
                    options: IndexMap::from([(
                        String::from("t2_one"),
                        RoleButton {
                            emoji: String::from("emoki_1"),
                            label: String::from("label_3"),
                            role_id: RoleId::new(3),
                        }
                    ),]),
                }
            ]
        )
    }
}
