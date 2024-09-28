mod player_accessors;
mod player_reference;
mod player_send_packet;
mod player_reset;
mod player_helper_functions;

pub use player_reference::PlayerIndex;
pub use player_reference::PlayerReference;
use vec1::Vec1;

use std::collections::HashMap;
use std::collections::HashSet;

use crate::client_connection::ClientConnection;
use crate::{
    game::{
        role::{Role, RoleState}, 
        chat::ChatMessageVariant, 
        visit::Visit, 
        grave::GraveKiller, 
        verdict::Verdict, available_buttons::AvailableButtons
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
    notes: String,
    crossed_out_outlines: Vec<u8>,
    death_note: Option<String>,

    role_labels: HashSet<PlayerReference>,
    player_tags: HashMap<PlayerReference, Vec1<Tag>>,


    chat_messages: Vec<ChatMessage>,
    queued_chat_messages: Vec<ChatMessage>, // Not yet sent to the client

    win_condition: WinCondition,

    last_sent_buttons: Vec<AvailableButtons>,


    fast_forward_vote: bool,
    forfeit_vote: bool,

    voting_variables: PlayerVotingVariables,
    night_variables: PlayerNightVariables,
}
struct PlayerVotingVariables{
    chosen_vote:    Option<PlayerReference>,
    verdict:        Verdict,
}
struct PlayerNightVariables{
    died: bool,
    attacked: bool,
    jailed: bool,
    roleblocked: bool,
    upgraded_defense: Option<DefensePower>,

    appeared_visits: Option<Vec<Visit>>,
    framed: bool,

    silenced: bool,

    selection: Vec<PlayerReference>,
    visits: Vec<Visit>,

    messages: Vec<ChatMessageVariant>,

    grave_role: Option<Role>,
    grave_killers: Vec<GraveKiller>,
    grave_will: String,
    grave_death_notes: Vec<String>,
}
impl Player {
    pub fn new(name: String, sender: ClientSender, role: Role) -> Self {
        Self {
            connection: ClientConnection::Connected(sender),

            name,
            role_state: role.default_state(),
            alive: true,
            will: "".to_string(),
            notes: "".to_string(),
            crossed_out_outlines: vec![],
            death_note: None,

            role_labels: HashSet::new(),
            player_tags: HashMap::new(),

            win_condition: role.default_state().default_win_condition(),

            chat_messages: Vec::new(),
            queued_chat_messages: Vec::new(),
            
            last_sent_buttons: Vec::new(),

            fast_forward_vote: false,
            forfeit_vote: false,

            voting_variables: PlayerVotingVariables{
                chosen_vote : None,
                verdict : Verdict::Abstain,
            },
            night_variables: PlayerNightVariables{
                died: false,
                attacked: false,
                jailed: false,
                roleblocked: false,
                upgraded_defense: None,
                appeared_visits: None,
                framed: false,

                silenced: false,

                selection: vec![],
                visits: vec![],

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
    use std::{collections::{HashMap, HashSet}, time::Duration};

    use crate::{client_connection::ClientConnection, game::{role::Role, verdict::Verdict}};

    use super::{Player, PlayerVotingVariables, PlayerNightVariables};

    pub fn mock_player(name: String, role: Role) -> Player {
        Player {
            // Since `tick` is never called in tests, this will never decrement.
            connection: ClientConnection::CouldReconnect { disconnect_timer: Duration::from_secs(1) },

            name,
            role_state: role.default_state(),
            alive: true,
            will: "".to_string(),
            notes: "".to_string(),
            crossed_out_outlines: vec![],
            death_note: None,

            role_labels: HashSet::new(),
            player_tags: HashMap::new(),

            win_condition: role.default_state().default_win_condition(),

            chat_messages: Vec::new(),
            queued_chat_messages: Vec::new(),
            
            last_sent_buttons: Vec::new(),

            fast_forward_vote: false,
            forfeit_vote: false,

            voting_variables: PlayerVotingVariables{
                chosen_vote : None,
                verdict : Verdict::Abstain,
            },
            night_variables: PlayerNightVariables{
                died: false,
                attacked: false,
                jailed: false,
                roleblocked: false,
                upgraded_defense: None,
                appeared_visits: None,
                framed:      false,

                silenced: false,

                selection: vec![],
                visits: vec![],

                messages: vec![],

                grave_role: None,
                grave_killers: vec![],
                grave_will: "".to_string(),
                grave_death_notes: vec![],
            },
        }
    }
}
