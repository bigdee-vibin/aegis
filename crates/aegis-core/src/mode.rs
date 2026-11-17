use serde::{Deserialize, Serialize};

/// Operating mode. `LocalOnly` must never produce a cloud route (invariant I2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingMode {
    /// Local inference only; no cloud fallback, ever.
    LocalOnly,
    /// Cloud inference only.
    OnlineOnly,
    /// Local for sensitive work, cloud for sanitised/low-risk work.
    Hybrid,
    /// Router decides per request, always with an explanation.
    Auto,
}

impl OperatingMode {
    /// Whether this mode may ever select a cloud provider.
    #[must_use]
    pub fn allows_cloud(self) -> bool {
        !matches!(self, Self::LocalOnly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_only_never_allows_cloud() {
        assert!(!OperatingMode::LocalOnly.allows_cloud());
        assert!(OperatingMode::Hybrid.allows_cloud());
    }
}
