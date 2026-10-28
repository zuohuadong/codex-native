use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemePreference {
    Auto,
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub root_bg: u32,
    pub sidebar_bg: u32,
    pub text_primary: u32,
    pub card_bg: u32,
    /// Zai `--color-panel`. Light is white; dark is #202020, not the card gray.
    pub panel_bg: u32,
    /// Zai tag at 50%, used for inline code (`--color-markdown-inline-code` / 50).
    pub inline_code_bg: u32,
    /// Zai `--color-icon-blue` (`--color-terminal-bright-blue`).
    pub link: u32,
    /// Zai `--color-card-border`, an alias of the 10% `--color-border`.
    pub card_border: u32,
    pub card_user_bg: u32,
    pub dock_bg: u32,
    pub dock_border: u32,
    pub dock_border_active: u32,
    pub accent_green: u32,
    pub settings_border: u32,
    /// Reference `--color-hover`, distinct from the stronger selected surface.
    pub hover: u32,
    pub selected: u32,
    /// Reference `--color-menu-hover`, distinct from the alpha `selected` surface.
    pub menu_hover: u32,
    pub muted_foreground: u32,
    /// RGBA foreground tokens from the Zai theme CSS cascade.
    pub foreground_subtle: u32,
    pub foreground_subtlest: u32,
    pub workflow_bg: u32,
    /// RGBA surfaces, composited over the card rather than baked into gray.
    pub workflow_node_bg: u32,
    pub workflow_node_hover: u32,
    pub workflow_active: u32,
    /// Zai `--color-destructive` (`#e03131` light / `#ff5c5c` dark).
    pub workflow_error: u32,
    /// `--color-destructive-surface`: 10% light, 16% dark.
    pub destructive_surface: u32,
    /// `border-destructive/30` on the same destructive token.
    pub destructive_border: u32,
    pub workflow_connector: u32,
    pub workflow_avatars: [u32; 9],
}

impl ThemeColors {
    pub fn light() -> Self {
        Self {
            root_bg: 0xf8f8f8,
            sidebar_bg: 0xf0f0f0,
            text_primary: 0x262626,
            card_bg: 0xffffff,
            panel_bg: 0xffffff,
            inline_code_bg: 0xe6e6e680,
            link: 0x0066dd,
            card_border: 0x0d0d0d1a,
            card_user_bg: 0x0d0d0d08,
            dock_bg: 0xffffff,
            dock_border: 0x0d0d0d1a,
            dock_border_active: 0x0d0d0d26,
            accent_green: 0x1e8a3e,
            settings_border: 0x0d0d0d1a,
            hover: 0x0d0d0d0d,
            selected: 0x0d0d0d0d,
            menu_hover: 0xf0f0f0,
            muted_foreground: 0x26262699,
            foreground_subtle: 0x26262699,
            foreground_subtlest: 0x26262666,
            workflow_bg: 0xfdfdfd,
            workflow_node_bg: 0x0d0d0d08,
            workflow_node_hover: 0x0d0d0d0d,
            workflow_active: 0xe07b00,
            workflow_error: 0xe03131,
            destructive_surface: 0xe031311a,
            destructive_border: 0xe031314d,
            workflow_connector: 0xa7a7a7,
            workflow_avatars: [
                0x54b9a6, 0xf19d38, 0x6464ef, 0x885cf5, 0x3c82f6, 0xed712e, 0xeb4699, 0x5bc67a,
                0xea4045,
            ],
        }
    }

    pub fn dark() -> Self {
        Self {
            root_bg: 0x161616,
            sidebar_bg: 0x2b2b2b,
            text_primary: 0xd4d4d4,
            card_bg: 0x2b2b2b,
            panel_bg: 0x202020,
            inline_code_bg: 0x36363680,
            link: 0x80beff,
            card_border: 0xffffff1a,
            card_user_bg: 0xffffff0d,
            dock_bg: 0x2b2b2b,
            dock_border: 0xffffff1a,
            dock_border_active: 0xffffff26,
            accent_green: 0x46bf72,
            settings_border: 0xffffff1a,
            hover: 0xffffff0d,
            selected: 0xffffff1a,
            menu_hover: 0x363636,
            muted_foreground: 0xd4d4d499,
            foreground_subtle: 0xd4d4d499,
            foreground_subtlest: 0xd4d4d44d,
            workflow_bg: 0x252525,
            workflow_node_bg: 0xffffff0d,
            workflow_node_hover: 0xffffff1a,
            workflow_active: 0xff8a30,
            workflow_error: 0xff5c5c,
            destructive_surface: 0xff5c5c29,
            destructive_border: 0xff5c5c4d,
            workflow_connector: 0x5a5a5a,
            workflow_avatars: [
                0x54b9a6, 0xf19d38, 0x6464ef, 0x885cf5, 0x3c82f6, 0xed712e, 0xeb4699, 0x5bc67a,
                0xea4045,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThemeColors;

    #[test]
    fn destructive_tokens_follow_the_zai_cascade() {
        let light = ThemeColors::light();
        assert_eq!(light.card_border, 0x0d0d0d1a);
        assert_eq!(light.workflow_error, 0xe03131);
        assert_eq!(light.destructive_surface, 0xe031311a);
        assert_eq!(light.destructive_border, 0xe031314d);

        let dark = ThemeColors::dark();
        assert_eq!(dark.card_border, 0xffffff1a);
        assert_eq!(dark.workflow_error, 0xff5c5c);
        assert_eq!(dark.destructive_surface, 0xff5c5c29);
        assert_eq!(dark.destructive_border, 0xff5c5c4d);
    }
}
