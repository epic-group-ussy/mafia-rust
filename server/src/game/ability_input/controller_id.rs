use serde::{Deserialize, Serialize};

use crate::game::{player::PlayerReference, role::Role};

pub type RoleControllerID = u8;
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "camelCase")]
#[serde(tag="type")]
pub enum ControllerID{
    #[serde(rename_all = "camelCase")]
    Role{
        player: PlayerReference,
        role: Role,
        id: RoleControllerID
    },
    Nominate{
        player: PlayerReference
    },
    ForfeitVote{
        player: PlayerReference
    },
    PitchforkVote{
        player: PlayerReference
    },
    SyndicateGunItemShoot,
    SyndicateGunItemGive,
    SyndicateChooseBackup,
    SyndicateBackupAttack,

    WardenLiveOrDie{
        warden: PlayerReference,
        player: PlayerReference,
    }
}
impl ControllerID{
    pub fn role(player: PlayerReference, role: Role, id: RoleControllerID)->Self{
        Self::Role{player, role, id}
    }
    pub fn forfeit_vote(player: PlayerReference)->Self{
        Self::ForfeitVote{player}
    }
    pub fn pitchfork_vote(player: PlayerReference)->Self{
        Self::PitchforkVote{player}
    }
    pub fn syndicate_gun_item_shoot()->Self{
        Self::SyndicateGunItemShoot
    }
    pub fn syndicate_gun_item_give()->Self{
        Self::SyndicateGunItemGive
    }
    pub fn syndicate_choose_backup()->Self{
        Self::SyndicateChooseBackup
    }
    pub fn syndicate_backup_attack()->Self{
        Self::SyndicateBackupAttack
    }
}