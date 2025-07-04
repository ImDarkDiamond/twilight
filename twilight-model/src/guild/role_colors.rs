use serde::{Deserialize, Serialize};

/// [`Role`] colors
///
/// [`RoleColors::primary_color`] will always be filled.
///
/// Other fields can only be set to a non-null value if the guild has the [`ENHANCED_ROLE_COLORS`] guild feature.
///
/// See [Discord Docs/Role Colors]
///
/// [`Role`]: super::Role
/// [`ENHANCED_ROLE_COLORS`]: super::GuildFeature::EnhancedRoleColors
/// [Discord Docs/Role Colors]: https://discord.com/developers/docs/topics/permissions#role-object-role-colors-object
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RoleColors {
    /// Primary color of the role.
    ///
    /// This must be a valid hexadecimal RGB value. `0x000000` is ignored and
    /// doesn't count towards the final computed color in the user list. Refer
    /// to [`COLOR_MAXIMUM`] for the maximum acceptable value.
    ///
    /// [`COLOR_MAXIMUM`]: twilight_validate::embed::COLOR_MAXIMUM
    pub primary_color: u32,
    /// Secondary color for the role, this will make the role a gradient between the other provided colors.
    ///
    /// This must be a valid hexadecimal RGB value.
    /// Refer to [`COLOR_MAXIMUM`] for the maximum acceptable value.
    ///
    /// [`COLOR_MAXIMUM`]: twilight_validate::embed::COLOR_MAXIMUM
    pub secondary_color: Option<u32>,
    /// Tertiary color for the role, this will turn the gradient into a holographic style.
    ///
    /// When sending `tertiary_color` the API enforces the role color to be a holographic style with values of:
    /// - `primary_color` = 11127295
    /// - `secondary_color` = 16759788
    /// - `tertiary_color` = 16761760
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
