#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    Allow,
    Default,
    Deny,
}

impl TriState {
    pub fn is_allowed(self) -> bool {
        matches!(self, TriState::Allow | TriState::Default)
    }

    pub fn is_denied(self) -> bool {
        matches!(self, TriState::Deny)
    }
}

impl From<bool> for TriState {
    fn from(value: bool) -> Self {
        if value {
            TriState::Allow
        } else {
            TriState::Deny
        }
    }
}

impl From<TriState> for bool {
    fn from(value: TriState) -> bool {
        value.is_allowed()
    }
}
