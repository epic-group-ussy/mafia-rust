use serde::{Serialize, Deserialize};

use crate::game::player::PlayerIndex;
use crate::game::role::Role;

use super::ChatMessage;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum NightInformation {
    RoleBlocked { immune : bool },  //you were roleblocked
    TargetSurvivedAttack,
    YouSurvivedAttack,
    YouDied,

    /* Role-specific */
    

    SpyMafiaVisit{players: Vec<PlayerIndex>},
    SpyCovenVisit{players: Vec<PlayerIndex>},
    SpyBug{message: Box<ChatMessage>},

    VeteranAttackedYou,
    VeteranAttackedVisitor,

    VigilanteSuicide,

    DoctorHealed,   //"Someone attacked you but a doctor nursed you back to health"
    DoctorHealedYou,

    BodyguardProtected,
    BodyguardProtectedYou,

    Transported,

    RetributionistBug{message: Box<ChatMessage>},
    NecromancerBug{message: Box<ChatMessage>},

    GodfatherForcedMafioso,
    GodfatherForcedYou,

    Silenced,

    JanitorResult { role: Role, will: String },
    ForgerResult { role: Role, will: String },
    ConsigliereResult{ role: Role },
    
    SheriffResult { suspicious: bool },
    LookoutResult{players: Vec<PlayerIndex>},
    InvestigatorResult{roles: Vec<Role>},
 
    WitchTargetImmune,
    WitchedYou { immune: bool },    //you were witched
    WitchBug{message: Box<ChatMessage>},

    ArsonistCleanedSelf,    //You cleaned the gas off yourself
    ArsonistWasDoused,  //you were doused in gas (only arsonists recieve this message)
}
