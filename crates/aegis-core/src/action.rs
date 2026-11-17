use serde::{Deserialize, Serialize};

/// Policy outcome. Ordered: `Allow < Notify < Review < Block` (system architecture §7).
///
/// Combining outcomes always takes the maximum, so adding evidence can only
/// make the result stricter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Proceed silently.
    Allow,
    /// Proceed and tell the user.
    Notify,
    /// Hold until the user decides.
    Review,
    /// Never proceed.
    Block,
}

impl Action {
    /// Combine two outcomes; the stricter one wins.
    #[must_use]
    pub fn combine(self, other: Self) -> Self {
        self.max(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_strictness() {
        assert!(Action::Allow < Action::Notify);
        assert!(Action::Notify < Action::Review);
        assert!(Action::Review < Action::Block);
    }

    #[test]
    fn combine_takes_stricter() {
        assert_eq!(Action::Allow.combine(Action::Review), Action::Review);
        assert_eq!(Action::Block.combine(Action::Notify), Action::Block);
    }
}
