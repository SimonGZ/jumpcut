use super::wrapping::InterruptionDashWrap;

/// The named pagination contracts supported by JumpCut.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaginationProfile {
    /// JumpCut's opinionated, configurable pagination behavior.
    #[default]
    Jumpcut,
    /// Stable compatibility behavior for conventional screenplay pagination.
    Industry,
}

/// Behavior that affects pagination decisions without changing page geometry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationPolicy {
    pub profile: PaginationProfile,
    pub break_dialogue_and_action_at_sentences: bool,
    pub interruption_dash_wrap: InterruptionDashWrap,
    pub dual_dialogue_counts_for_contd: bool,
    pub automatic_character_continueds: bool,
    pub orphan_limit: usize,
    pub widow_limit: usize,
    pub avoid_page_start_parenthetical: bool,
}

impl PaginationPolicy {
    pub fn jumpcut() -> Self {
        Self {
            profile: PaginationProfile::Jumpcut,
            break_dialogue_and_action_at_sentences: true,
            interruption_dash_wrap: InterruptionDashWrap::KeepTogether,
            dual_dialogue_counts_for_contd: false,
            automatic_character_continueds: true,
            orphan_limit: 2,
            widow_limit: 2,
            avoid_page_start_parenthetical: true,
        }
    }

    pub fn industry() -> Self {
        Self {
            profile: PaginationProfile::Industry,
            break_dialogue_and_action_at_sentences: true,
            interruption_dash_wrap: InterruptionDashWrap::Industry,
            dual_dialogue_counts_for_contd: true,
            automatic_character_continueds: true,
            orphan_limit: 2,
            widow_limit: 2,
            avoid_page_start_parenthetical: false,
        }
    }
}

impl Default for PaginationPolicy {
    fn default() -> Self {
        Self::jumpcut()
    }
}
