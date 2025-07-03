use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RoleColors {
    pub primary_color: u32,
    pub secondary_color: Option<u32>,
    pub tertiary_color: Option<u32>,
}

#[cfg(test)]
mod tests {
    use crate::guild::RoleColors;

    use serde::{Deserialize, Serialize};
    use serde_test::Token;
    use static_assertions::{assert_fields, assert_impl_all};
    use std::{fmt::Debug, hash::Hash};

    assert_fields!(
        RoleColors: primary_color,
        secondary_color,
        tertiary_color
    );

    assert_impl_all!(
        RoleColors: Clone,
        Debug,
        Deserialize<'static>,
        Eq,
        Hash,
        PartialEq,
        Serialize
    );

    #[test]
    fn role_colors() {
        let value = RoleColors {
            primary_color: 11127295,
            secondary_color: Some(16759788),
            tertiary_color: Some(16761760),
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "RoleColors",
                    len: 3,
                },
                Token::Str("primary_color"),
                Token::U32(11127295),
                Token::Str("secondary_color"),
                Token::Some,
                Token::U32(16759788),
                Token::Str("tertiary_color"),
                Token::Some,
                Token::U32(16761760),
                Token::StructEnd,
            ],
        );
    }
}
