use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryPolicy {
    pub group_threshold: u8,
    pub groups: Vec<RecoveryGroupPolicy>,
}

impl RecoveryPolicy {
    pub fn standard_cloud_plus_physical() -> Self {
        Self {
            group_threshold: 2,
            groups: vec![
                RecoveryGroupPolicy {
                    kind: RecoveryGroupKind::Cloud,
                    member_threshold: 2,
                    member_count: 2,
                },
                RecoveryGroupPolicy {
                    kind: RecoveryGroupKind::LocalPhysical,
                    member_threshold: 1,
                    member_count: 1,
                },
                RecoveryGroupPolicy {
                    kind: RecoveryGroupKind::RemotePhysical,
                    member_threshold: 1,
                    member_count: 1,
                },
            ],
        }
    }

    pub fn can_recover_groups<I>(&self, satisfied_groups: I) -> bool
    where
        I: IntoIterator<Item = RecoveryGroupKind>,
    {
        let satisfied: BTreeSet<_> = satisfied_groups.into_iter().collect();
        let count = self
            .groups
            .iter()
            .filter(|group| satisfied.contains(&group.kind))
            .count();

        count >= usize::from(self.group_threshold)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryGroupPolicy {
    pub kind: RecoveryGroupKind,
    pub member_threshold: u8,
    pub member_count: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryGroupKind {
    Cloud,
    LocalPhysical,
    RemotePhysical,
}

impl RecoveryGroupKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cloud => "cloud",
            Self::LocalPhysical => "local_physical",
            Self::RemotePhysical => "remote_physical",
        }
    }
}
