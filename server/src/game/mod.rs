pub mod grave;
pub mod phase;
pub mod player;
pub mod chat;
pub mod role;
pub mod visit;
pub mod verdict;
pub mod role_list;
pub mod settings;
pub mod game_conclusion;
pub mod components;
pub mod on_client_message;
pub mod tag;
pub mod event;
pub mod spectator;
pub mod game_listeners;
pub mod attack_power;
pub mod modifiers;
pub mod win_condition;
pub mod role_outline_reference;
pub mod ability_input;

use std::time::Duration;
use ability_input::saved_controllers_map::SavedControllersMap;
use ability_input::PlayerListSelection;
use components::confused::Confused;
use components::drunk_aura::DrunkAura;
use components::love_linked::LoveLinked;
use components::mafia::Mafia;
use components::night_visits::NightVisits;
use components::pitchfork::Pitchfork;
use components::mafia_recruits::MafiaRecruits;
use components::poison::Poison;
use components::detained::Detained;
use components::insider_group::InsiderGroupID;
use components::insider_group::InsiderGroups;
use components::syndicate_gun_item::SyndicateGunItem;
use components::synopsis::SynopsisTracker;
use components::verdicts_today::VerdictsToday;
use event::on_tick::OnTick;
use modifiers::ModifierType;
use modifiers::Modifiers;
use event::before_initial_role_creation::BeforeInitialRoleCreation;
use rand::seq::SliceRandom;
use role_list::RoleAssignment;
use role_list::RoleOutlineOptionInsiderGroups;
use role_list::RoleOutlineOptionWinCondition;
use role_outline_reference::RoleOutlineReference;
use serde::Serialize;
use win_condition::WinCondition;

use crate::client_connection::ClientConnection;
use crate::game::event::on_game_start::OnGameStart;
use crate::game::player::PlayerIndex;
use crate::packet::ToClientPacket;
use crate::vec_map::VecMap;
use crate::vec_set::VecSet;
use chat::{ChatMessageVariant, ChatGroup, ChatMessage};
use player::PlayerReference;
use player::Player;
use phase::PhaseStateMachine;
use settings::Settings;
use grave::Grave;
use self::components::{
    arsonist_doused::ArsonistDoused,
    cult::Cult,
    puppeteer_marionette::PuppeteerMarionette
};
use self::game_conclusion::GameConclusion;
use self::event::on_game_ending::OnGameEnding;
use self::event::on_grave_added::OnGraveAdded;
use self::grave::GraveReference;
use self::phase::PhaseState;
use self::player::PlayerInitializeParameters;
use self::spectator::{
    spectator_pointer::{
        SpectatorIndex, SpectatorPointer
    },
    Spectator,
    SpectatorInitializeParameters
};
use self::role::RoleState;
use self::verdict::Verdict;


pub struct Game {
    pub settings : Settings,

    pub spectators: Vec<Spectator>,
    pub spectator_chat_messages: Vec<ChatMessageVariant>,

    /// indexed by role outline reference
    pub assignments: Vec<(PlayerReference, RoleOutlineReference, RoleAssignment)>,

    pub players: Box<[Player]>,
    pub graves: Vec<Grave>,

    phase_machine : PhaseStateMachine,

    
    /// Whether the game is still updating phase times
    pub ticking: bool,
    
    
    //components with data
    pub saved_controllers: SavedControllersMap,
    night_visits: NightVisits,
    syndicate_gun_item: SyndicateGunItem,
    pub cult: Cult,
    pub mafia: Mafia,
    pub arsonist_doused: ArsonistDoused,
    pub puppeteer_marionette: PuppeteerMarionette,
    pub mafia_recruits: MafiaRecruits,
    pub love_linked: LoveLinked,
    pub verdicts_today: VerdictsToday,
    pub pitchfork: Pitchfork,
    pub poison: Poison,
    pub modifiers: Modifiers,
    pub revealed_groups: InsiderGroups,
    pub detained: Detained,
    pub confused: Confused,
    pub drunk_aura: DrunkAura,
    pub synopsis_tracker: SynopsisTracker
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum RejectStartReason {
    GameEndsInstantly,
    RoleListTooSmall,
    RoleListCannotCreateRoles,
    ZeroTimeGame,
    PlayerDisconnected
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum GameOverReason {
    ReachedMaxDay,
    Winner,
    Draw
}



impl Game {
    pub fn new(settings: Settings, players: Vec<PlayerInitializeParameters>, spectators: Vec<SpectatorInitializeParameters>) -> Result<Self, RejectStartReason>{
        //check settings are not completly off the rails
        if settings.phase_times.game_ends_instantly() {
            return Err(RejectStartReason::ZeroTimeGame);
        }
        

        let mut role_generation_tries = 0;
        const MAX_ROLE_GENERATION_TRIES: u8 = 250;
        let (mut game, assignments) = loop {

            if role_generation_tries >= MAX_ROLE_GENERATION_TRIES {
                return Err(RejectStartReason::RoleListCannotCreateRoles);
            }

            let settings = settings.clone();
            let role_list = settings.role_list.clone();

            let random_outline_assignments = match role_list.create_random_role_assignments(&settings.enabled_roles){
                Some(roles) => {roles},
                None => {return Err(RejectStartReason::RoleListCannotCreateRoles);}
            };

            let assignments = Self::assign_players_to_assignments(random_outline_assignments);            


            // Create list of players
            let mut new_players = Vec::new();
            for (player_index, player) in players.iter().enumerate() {

                let ClientConnection::Connected(ref sender) = player.connection else {
                    return Err(RejectStartReason::PlayerDisconnected)
                };
                let Some((_, _, assignment)) = assignments.iter().find(|(p,_,_)|p.index() == player_index as u8) else {
                    return Err(RejectStartReason::RoleListTooSmall)
                };

                // Set win condition & Insider group here so we can check if game ends
                let win_condition = match &assignment.win_condition {
                    RoleOutlineOptionWinCondition::RoleDefault => assignment.role.default_state().default_win_condition(),
                    RoleOutlineOptionWinCondition::GameConclusionReached { win_if_any } => {
                        WinCondition::GameConclusionReached { 
                            win_if_any: win_if_any.iter().cloned().collect()
                        }
                    },
                };

                let new_player = Player::new(
                    player.name.clone(),
                    sender.clone(),
                    assignment.role,
                    win_condition
                );
                
                new_players.push(new_player);
            }

            let num_players = new_players.len() as u8;

            let mut game = Self{
                pitchfork: Pitchfork::new(num_players),

                assignments: assignments.clone(),
                ticking: true,
                spectators: spectators.clone().into_iter().map(Spectator::new).collect(),
                spectator_chat_messages: Vec::new(),
                players: new_players.into_boxed_slice(),
                graves: Vec::new(),
                phase_machine: PhaseStateMachine::new(settings.phase_times.clone()),
                modifiers: Modifiers::default_from_settings(settings.enabled_modifiers.clone()),
                settings,

                saved_controllers: SavedControllersMap::default(),
                night_visits: NightVisits::default(),
                syndicate_gun_item: SyndicateGunItem::default(),
                cult: Cult::default(),
                mafia: Mafia,
                arsonist_doused: ArsonistDoused::default(),
                puppeteer_marionette: PuppeteerMarionette::default(),
                mafia_recruits: MafiaRecruits::default(),
                love_linked: LoveLinked::default(),
                verdicts_today: VerdictsToday::default(),
                poison: Poison::default(),

                revealed_groups: InsiderGroups::default(),
                detained: Detained::default(),
                confused: Confused::default(),
                drunk_aura: DrunkAura::default(),
                synopsis_tracker: SynopsisTracker::new(num_players)
            };

            // Just distribute insider groups, this is for game over checking (Keeps game running syndicate gun)
            for player in PlayerReference::all_players(&game){
                let Some((player, _, assignment)) = assignments
                    .iter()
                    .find(|(p,_,_)|*p == player) else {
                        return Err(RejectStartReason::RoleListTooSmall)
                    };
                
                let insider_groups = match &assignment.insider_groups {
                    RoleOutlineOptionInsiderGroups::RoleDefault => assignment.role.default_state().default_revealed_groups(),
                    RoleOutlineOptionInsiderGroups::Custom { insider_groups } => insider_groups.iter().cloned().collect(),
                };
                
                for group in insider_groups{
                    unsafe {
                        group.add_player_to_revealed_group_unchecked(&mut game, *player);
                    }
                }
            }


            if !game.game_is_over() {
                break (game, assignments);
            }
            role_generation_tries += 1;
        };

        if game.game_is_over() {
            return Err(RejectStartReason::RoleListCannotCreateRoles);
        }
        
        game.send_packet_to_all(ToClientPacket::StartGame);

        //set wincons and revealed groups
        for player in PlayerReference::all_players(&game){
            let role_data = player.role(&game).new_state(&game);

            let Some((_, _, assignment)) = assignments
                .iter()
                .find(|(p,_,_)|*p == player) else {
                    return Err(RejectStartReason::RoleListTooSmall)
                };

            // We already set this earlier, now we just need to call the on_convert event. Hope this doesn't end the game!
            let win_condition = player.win_condition(&game).clone();
            player.set_win_condition(&mut game, win_condition);

            let insider_groups = match &assignment.insider_groups {
                RoleOutlineOptionInsiderGroups::RoleDefault => role_data.clone().default_revealed_groups(),
                RoleOutlineOptionInsiderGroups::Custom { insider_groups } => insider_groups.iter().cloned().collect(),
            };
            
            InsiderGroupID::start_game_set_player_revealed_groups(
                insider_groups,
                &mut game,
                player
            );
        }

        BeforeInitialRoleCreation::invoke(&mut game);
        
        //on role creation needs to be called after all players roles are known
        //trigger role event listeners
        for player_ref in PlayerReference::all_players(&game){
            player_ref.initial_role_creation(&mut game);
        }

        for player_ref in PlayerReference::all_players(&game){
            player_ref.send_join_game_data(&mut game);
        }
        for spectator in SpectatorPointer::all_spectators(&game){
            spectator.send_join_game_data(&mut game);
        }

        //reveal groups
        for group in InsiderGroupID::all() {
            group.reveal_group_players(&mut game);
        }

        //on game start needs to be called after all players have joined
        OnGameStart::invoke(&mut game);

        Ok(game)
    }
    
    fn assign_players_to_assignments(initialization_data: Vec<RoleAssignment>)->Vec<(PlayerReference, RoleOutlineReference, RoleAssignment)>{
        let mut player_indices: Vec<PlayerIndex> = (0..initialization_data.len() as PlayerIndex).collect();
        player_indices.shuffle(&mut rand::rng());

        initialization_data
            .into_iter()
            .enumerate()
            .zip(player_indices.into_iter())
            .map(|((o_index, assignment), p_index)|
                // We are iterating through playerlist and outline list, so this unsafe should be fine
                unsafe {
                    (PlayerReference::new_unchecked(p_index), RoleOutlineReference::new_unchecked(o_index as u8), assignment)
                }
            )
            .collect()
    }

    pub fn num_players(&self) -> u8 {
        self.players.len() as u8
    }

    /// Returns a tuple containing the number of guilty votes and the number of innocent votes
    pub fn count_verdict_votes(&self, player_on_trial: PlayerReference)->(u8,u8){
        let mut guilty = 0;
        let mut innocent = 0;
        for player_ref in PlayerReference::all_players(self){
            if !player_ref.alive(self) || player_ref == player_on_trial {
                continue;
            }
            let mut voting_power = 1;
            if let RoleState::Mayor(mayor) = player_ref.role_state(self).clone(){
                if mayor.revealed {
                    voting_power += 2;
                }
            }
            if let RoleState::Politician(politician) = player_ref.role_state(self).clone(){
                if politician.revealed {
                    voting_power += 2;
                }
            }
            
            match player_ref.verdict(self) {
                Verdict::Innocent => innocent += voting_power,
                Verdict::Abstain => {},
                Verdict::Guilty => guilty += voting_power,
            }
        }
        (guilty, innocent)
    }
    
    /// this is sent to the players whenever this function is called
    fn create_voted_player_map(&self) -> VecMap<PlayerReference, u8> {
        let mut voted_player_votes: VecMap<PlayerReference, u8> = VecMap::new();

        for player in PlayerReference::all_players(self){
            if !player.alive(self) { continue }

            let Some(PlayerListSelection(voted_players)) = self
                .saved_controllers
                .get_controller_current_selection_player_list(ability_input::ControllerID::Nominate { player }) else {
                    continue;
                };
            let Some(&voted_player) = voted_players.first() else { continue };
            

            let mut voting_power = 1;
            if let RoleState::Mayor(mayor) = player.role_state(self).clone() {
                if mayor.revealed {
                    voting_power = 3;
                }
            }
            else if let RoleState::Politician(politician) = player.role_state(self).clone() {
                if politician.revealed {
                    voting_power = 3;
                }
            }

            if let Some(num_votes) = voted_player_votes.get_mut(&voted_player) {
                *num_votes += voting_power;
            } else {
                voted_player_votes.insert(voted_player, voting_power);
            }
        }

        self.send_packet_to_all(
            ToClientPacket::PlayerVotes { votes_for_player: 
                PlayerReference::ref_vec_map_to_index(voted_player_votes.clone())
            }
        );

        voted_player_votes
    }
    /// Returns the player who is meant to be put on trial
    /// None if its not nomination
    /// None if nobody has enough votes
    /// None if there is a tie
    pub fn count_nomination_and_start_trial(&mut self, start_trial_instantly: bool)->Option<PlayerReference>{

        let &PhaseState::Nomination { trials_left, .. } = self.current_phase() else {return None};

        let voted_player_votes = self.create_voted_player_map();

        let mut voted_player = None;

        if let Some(maximum_votes) = voted_player_votes.values().max() {
            if self.nomination_votes_is_enough(*maximum_votes){
                let max_votes_players: VecSet<PlayerReference> = voted_player_votes.iter()
                    .filter(|(_, votes)| **votes == *maximum_votes)
                    .map(|(player, _)| *player)
                    .collect();

                if max_votes_players.len() == 1 {
                    voted_player = max_votes_players.iter().next().cloned();
                }
            }
        }
        
        if start_trial_instantly {
            if let Some(player_on_trial) = voted_player {
                self.send_packet_to_all(ToClientPacket::PlayerOnTrial { player_index: player_on_trial.index() } );
                
                PhaseStateMachine::next_phase(self, Some(PhaseState::Testimony {
                    trials_left: trials_left.saturating_sub(1), 
                    player_on_trial, 
                    nomination_time_remaining: self.phase_machine.get_time_remaining()
                }));
            }
        }

        voted_player
    }

    
    pub fn nomination_votes_is_enough(&self, votes: u8)->bool{
        votes >= self.nomination_votes_required()
    }
    pub fn nomination_votes_required(&self)->u8{
        let eligible_voters = PlayerReference::all_players(self)
            .filter(|p| p.alive(self) && !p.forfeit_vote(self))
            .count() as u8;

        if Modifiers::modifier_is_enabled(self, ModifierType::TwoThirdsMajority) {
            (eligible_voters + 1) * 2 / 3
        } else {
            1 + eligible_voters / 2
        }
    }


    pub fn game_is_over(&self) -> bool {
        if let Some(_) = GameConclusion::game_is_over(self){
            true
        }else{
            false
        }
    }

    pub fn current_phase(&self) -> &PhaseState {
        &self.phase_machine.current_state
    }

    pub fn day_number(&self) -> u8 {
        self.phase_machine.day_number
    }

    pub fn tick(&mut self, time_passed: Duration){

        if !self.ticking { return }

        if let Some(conclusion) = GameConclusion::game_is_over(self) {
            OnGameEnding::new(conclusion).invoke(self);
        }

        if self.phase_machine.day_number == u8::MAX {
            self.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::GameOver { 
                synopsis: SynopsisTracker::get(self, GameConclusion::Draw)
            });
            self.send_packet_to_all(ToClientPacket::GameOver{ reason: GameOverReason::ReachedMaxDay });
            self.ticking = false;
            return;
        }

        while self.phase_machine.time_remaining <= Duration::ZERO {
            PhaseStateMachine::next_phase(self, None);
        }
        PlayerReference::all_players(self).for_each(|p|p.tick(self, time_passed));
        SpectatorPointer::all_spectators(self).for_each(|s|s.tick(self, time_passed));

        self.phase_machine.time_remaining = self.phase_machine.time_remaining.saturating_sub(time_passed);

        OnTick::new().invoke(self);
    }

    pub fn add_grave(&mut self, grave: Grave){
        self.graves.push(grave.clone());
        if let Some(grave_ref) = GraveReference::new(
            self, 
            self.graves.len()
                .saturating_sub(1)
                .try_into()
                .expect("There can not be more than u8::MAX graves"))
        {
            OnGraveAdded::new(grave_ref).invoke(self);
        }
    }

    pub fn add_message_to_chat_group(&mut self, group: ChatGroup, variant: ChatMessageVariant){
        let message = ChatMessage::new_non_private(variant.clone(), group.clone());

        for player_ref in group.all_players_in_group(self){
            player_ref.add_chat_message(self, message.clone());
            player_ref.send_chat_messages(self);
        }

        if group == ChatGroup::All {
            self.add_chat_message_to_spectators(variant);
        }
    }
    pub fn add_messages_to_chat_group(&mut self, group: ChatGroup, messages: Vec<ChatMessageVariant>){
        for message in messages.into_iter(){
            self.add_message_to_chat_group(group.clone(), message);
        }
    }
    pub fn add_chat_message_to_spectators(&mut self, message: ChatMessageVariant){
        for spectator in self.spectators.iter_mut(){
            spectator.queued_chat_messages.push(message.clone());
        }
        self.spectator_chat_messages.push(message);
    }

    pub fn add_spectator(&mut self, params: SpectatorInitializeParameters) -> SpectatorIndex {
        self.spectators.push(Spectator::new(params));
        let spectator_pointer = SpectatorPointer::new(self.spectators.len() as SpectatorIndex - 1);

        spectator_pointer.send_join_game_data(self);

        spectator_pointer.index
    }
    pub fn remove_spectator(&mut self, i: SpectatorIndex){
        self.spectators.remove(i as usize);
    }

    pub fn send_packet_to_all(&self, packet: ToClientPacket){
        for player_ref in PlayerReference::all_players(self){
            player_ref.send_packet(self, packet.clone());
        }
        for spectator in self.spectators.iter(){
            spectator.send_packet(packet.clone());
        }
    }
}

pub mod test {

    use super::{
        ability_input::saved_controllers_map::SavedControllersMap,
        components::{
            arsonist_doused::ArsonistDoused, cult::Cult, insider_group::InsiderGroupID,
            love_linked::LoveLinked, mafia::Mafia,
            mafia_recruits::MafiaRecruits, night_visits::NightVisits,
            pitchfork::Pitchfork, poison::Poison,
            puppeteer_marionette::PuppeteerMarionette, syndicate_gun_item::SyndicateGunItem,
            synopsis::SynopsisTracker, verdicts_today::VerdictsToday
        }, 
        event::{before_initial_role_creation::BeforeInitialRoleCreation, on_game_start::OnGameStart},
        phase::PhaseStateMachine, player::{test::mock_player, PlayerReference},
        role::Role, settings::Settings, Game, RejectStartReason
    };
    
    pub fn mock_game(settings: Settings, number_of_players: usize) -> Result<Game, RejectStartReason> {

        //check settings are not completly off the rails
        if settings.phase_times.game_ends_instantly() {
            return Err(RejectStartReason::ZeroTimeGame);
        }

        let settings = settings.clone();
        let role_list = settings.role_list.clone();
        
        let random_outline_assignments = match role_list.create_random_role_assignments(&settings.enabled_roles){
            Some(roles) => {roles},
            None => {return Err(RejectStartReason::RoleListCannotCreateRoles);}
        };

        let assignments = Game::assign_players_to_assignments(random_outline_assignments);
        
        let shuffled_roles = assignments.iter().map(|(_,_,r)|r.role).collect::<Vec<Role>>();


        let mut players = Vec::new();
        for player_index in 0..number_of_players {
            let new_player = mock_player(
                format!("{}",player_index),
                match shuffled_roles.get(player_index){
                    Some(role) => *role,
                    None => return Err(RejectStartReason::RoleListTooSmall),
                }
            );
            players.push(new_player);
        }

        let mut game = Game{
            pitchfork: Pitchfork::new(number_of_players as u8),
            
            assignments,
            ticking: true,
            spectators: Vec::new(),
            spectator_chat_messages: Vec::new(),
            players: players.into_boxed_slice(),
            graves: Vec::new(),
            phase_machine: PhaseStateMachine::new(settings.phase_times.clone()),
            settings,

            saved_controllers: SavedControllersMap::default(),
            night_visits: NightVisits::default(),
            syndicate_gun_item: SyndicateGunItem::default(),
            cult: Cult::default(),
            mafia: Mafia,
            arsonist_doused: ArsonistDoused::default(),
            puppeteer_marionette: PuppeteerMarionette::default(),
            mafia_recruits: MafiaRecruits::default(),
            love_linked: LoveLinked::default(),
            verdicts_today: VerdictsToday::default(),
            poison: Poison::default(),
            modifiers: Default::default(),
            revealed_groups: Default::default(),
            detained: Default::default(),
            confused: Default::default(),
            drunk_aura: Default::default(),
            synopsis_tracker: SynopsisTracker::new(number_of_players as u8)
        };

        //set wincons and revealed groups
        for player in PlayerReference::all_players(&game){
            let role_data = player.role(&game).new_state(&game);

            player.set_win_condition(&mut game, role_data.clone().default_win_condition());
        
            InsiderGroupID::start_game_set_player_revealed_groups(
                role_data.clone().default_revealed_groups(),
                &mut game,
                player
            );
        }
        
        BeforeInitialRoleCreation::invoke(&mut game);

        //on role creation needs to be called after all players roles are known
        for player_ref in PlayerReference::all_players(&game){
            player_ref.initial_role_creation(&mut game);
        }

        OnGameStart::invoke(&mut game);

        Ok(game)
    }
}