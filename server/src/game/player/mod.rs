mod player_accessors;
mod player_reference;
mod player_send_packet;
mod player_reset;
mod player_helper_functions;
mod player_event_listeners;

pub use player_reference::PlayerIndex;
pub use player_reference::PlayerReference;
use vec1::Vec1;

use crate::client_connection::ClientConnection;
use crate::vec_map::VecMap;
use crate::vec_set::VecSet;
use crate::{
    game::{
        role::{Role, RoleState}, 
        chat::ChatMessageVariant, 
        visit::Visit, 
        grave::GraveKiller, 
        verdict::Verdict,
    },
    websocket_connections::connection::ClientSender,
};

use super::attack_power::DefensePower;
use super::chat::ChatMessage;
use super::tag::Tag;
use super::win_condition::WinCondition;

pub struct PlayerInitializeParameters {
    pub connection: ClientConnection,
    pub name: String,
    pub host: bool,
}
pub struct Player {
    connection: ClientConnection,

    name: String,
    role_state: RoleState,
    alive: bool,
    will: String,
    notes: Vec<String>,
    crossed_out_outlines: Vec<u8>,
    death_note: Option<String>,

    role_labels: VecSet<PlayerReference>,
    player_tags: VecMap<PlayerReference, Vec1<Tag>>,


    chat_messages: Vec<ChatMessage>,
    queued_chat_messages: Vec<ChatMessage>, // Not yet sent to the client

    win_condition: WinCondition,

    fast_forward_vote: bool,
    forfeit_vote: bool,

    voting_variables: PlayerVotingVariables,
    night_variables: PlayerNightVariables,
}
struct PlayerVotingVariables{
    verdict:        Verdict,
}
struct PlayerNightVariables{
    died: bool,
    attacked: bool,
    roleblocked: bool,
    upgraded_defense: Option<DefensePower>,

    convert_role_to: Option<RoleState>,

    appeared_visits: Option<Vec<Visit>>,
    framed: bool,

    silenced: bool,

    messages: Vec<ChatMessageVariant>,

    grave_role: Option<Role>,
    grave_killers: Vec<GraveKiller>,
    grave_will: String,
    grave_death_notes: Vec<String>,
}
impl Player {
    pub fn new(name: String, sender: ClientSender, role: Role, win_condition: WinCondition) -> Self {
        Self {
            connection: ClientConnection::Connected(sender),

            name,
            role_state: role.default_state(),
            alive: true,
            will: "".to_string(),
            notes: vec![],
            crossed_out_outlines: vec![],
            death_note: None,

            role_labels: VecSet::new(),
            player_tags: VecMap::new(),

            win_condition,

            chat_messages: Vec::new(),
            queued_chat_messages: Vec::new(),
            
            fast_forward_vote: false,
            forfeit_vote: false,

            voting_variables: PlayerVotingVariables{
                verdict : Verdict::Abstain,
            },
            night_variables: PlayerNightVariables{
                died: false,
                attacked: false,
                roleblocked: false,
                upgraded_defense: None,
                appeared_visits: None,
                framed: false,

                convert_role_to: None,

                silenced: false,

                messages: vec![],

                grave_role: None,
                grave_killers: vec![],
                grave_will: "".to_string(),
                grave_death_notes: vec![],
            },
        }
    }
}

pub mod test {
    use std::time::Duration;

    use crate::{client_connection::ClientConnection, game::{role::Role, verdict::Verdict}, vec_map::VecMap, vec_set::VecSet};

    use super::{Player, PlayerVotingVariables, PlayerNightVariables};

    pub fn mock_player(name: String, role: Role) -> Player {
        Player {
            // Since `tick` is never called in tests, this will never decrement.
            connection: ClientConnection::CouldReconnect { disconnect_timer: Duration::from_secs(1) },

            name,
            role_state: role.default_state(),
            alive: true,
            will: "".to_string(),
            notes: vec![],
            crossed_out_outlines: vec![],
            death_note: None,

            role_labels: VecSet::new(),
            player_tags: VecMap::new(),

            win_condition: role.default_state().default_win_condition(),

            chat_messages: Vec::new(),
            queued_chat_messages: Vec::new(),

            fast_forward_vote: false,
            forfeit_vote: false,

            voting_variables: PlayerVotingVariables{
                verdict : Verdict::Abstain,
            },
            night_variables: PlayerNightVariables{
                died: false,
                attacked: false,
                roleblocked: false,
                upgraded_defense: None,
                appeared_visits: None,
                framed: false,

                convert_role_to: None,

                silenced: false,

                messages: vec![],

                grave_role: None,
                grave_killers: vec![],
                grave_will: "".to_string(),
                grave_death_notes: vec![],
            },
        }
    }
}
