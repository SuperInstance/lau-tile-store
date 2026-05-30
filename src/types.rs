use std::fmt;

use serde::{Deserialize, Serialize};

/// Tile type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Observation,
    Action,
    Thought,
    Delegation,
    Escalation,
    Artifact,
    System,
    Onboarding,
    StandDown,
}

impl TileType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Observation => "observation",
            Self::Action => "action",
            Self::Thought => "thought",
            Self::Delegation => "delegation",
            Self::Escalation => "escalation",
            Self::Artifact => "artifact",
            Self::System => "system",
            Self::Onboarding => "onboarding",
            Self::StandDown => "stand_down",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "observation" => Some(Self::Observation),
            "action" => Some(Self::Action),
            "thought" => Some(Self::Thought),
            "delegation" => Some(Self::Delegation),
            "escalation" => Some(Self::Escalation),
            "artifact" => Some(Self::Artifact),
            "system" => Some(Self::System),
            "onboarding" => Some(Self::Onboarding),
            "stand_down" => Some(Self::StandDown),
            _ => None,
        }
    }

    pub fn all() -> Vec<TileType> {
        vec![
            Self::Observation,
            Self::Action,
            Self::Thought,
            Self::Delegation,
            Self::Escalation,
            Self::Artifact,
            Self::System,
            Self::Onboarding,
            Self::StandDown,
        ]
    }
}

impl fmt::Display for TileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Tile lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TileStatus {
    #[default]
    Active,
    Complete,
    Deadband,
    Escalated,
    Archived,
    Orphaned,
}

impl TileStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "active",
            Self::Complete => "complete",
            Self::Deadband => "deadband",
            Self::Escalated => "escalated",
            Self::Archived => "archived",
            Self::Orphaned => "orphaned",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "complete" => Some(Self::Complete),
            "deadband" => Some(Self::Deadband),
            "escalated" => Some(Self::Escalated),
            "archived" => Some(Self::Archived),
            "orphaned" => Some(Self::Orphaned),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete | Self::Archived | Self::Orphaned)
    }
}

impl fmt::Display for TileStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
