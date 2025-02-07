use std::collections::HashSet;
use rand::seq::SliceRandom;

use crate::{game::{
    ability_input::{AbilitySelection, ControllerID, ControllerParametersMap, PlayerListSelection, SavedControllersMap},
    attack_power::{AttackPower, DefensePower},
    chat::{ChatGroup, ChatMessage, ChatMessageVariant},
    components::{
        arsonist_doused::ArsonistDoused,
        drunk_aura::DrunkAura,
        insider_group::InsiderGroupID
    }, event::{
        before_role_switch::BeforeRoleSwitch, on_any_death::OnAnyDeath, on_role_switch::OnRoleSwitch
    }, game_conclusion::GameConclusion, grave::{Grave, GraveKiller}, modifiers::{ModifierType, Modifiers}, phase::PhaseType, role::{chronokaiser::Chronokaiser, Priority, Role, RoleState}, visit::{Visit, VisitTag}, win_condition::WinCondition, Game
}, packet::ToClientPacket, vec_map::VecMap, vec_set::VecSet};

use super::PlayerReference;

impl PlayerReference{
    pub fn roleblock(&self, game: &mut Game, send_messages: bool) {
        let roleblock_immune = self.role(game).roleblock_immune();

        if !roleblock_immune {
            self.set_night_roleblocked(game, true);
            self.set_night_visits(game, vec![]);
        }

        if send_messages {
            self.push_night_message(game,
                ChatMessageVariant::RoleBlocked { immune: roleblock_immune }
            );
        }
    }
    pub fn ward(&self, game: &mut Game) -> Vec<PlayerReference> {
        let mut wardblocked = vec![];
        for visitor in self.all_night_visitors_cloned(game){
            if !visitor.role(game).wardblock_immune() {
                visitor.set_night_wardblocked(game, true);
                visitor.set_night_visits(game, vec![]);
                visitor.push_night_message(game, ChatMessageVariant::Wardblocked);
                wardblocked.push(visitor);
            }
        }
        wardblocked
    }
    pub fn night_blocked(&self, game: &Game)->bool{
        self.night_roleblocked(game) || self.night_wardblocked(game)
    }



    /// Returns true if attack overpowered defense
    pub fn try_night_kill_single_attacker(&self, attacker_ref: PlayerReference, game: &mut Game, grave_killer: GraveKiller, attack: AttackPower, should_leave_death_note: bool) -> bool {
        self.try_night_kill(
            &vec![attacker_ref].into_iter().collect(),
            game,
            grave_killer,
            attack,
            should_leave_death_note
        )
    }
    pub fn try_night_kill(&self, attacker_refs: &VecSet<PlayerReference>, game: &mut Game, grave_killer: GraveKiller, attack: AttackPower, should_leave_death_note: bool) -> bool {
        self.set_night_attacked(game, true);

        if self.night_defense(game).can_block(attack){
            self.push_night_message(game, ChatMessageVariant::YouSurvivedAttack);
            for attacker in attacker_refs.iter() {
                attacker.push_night_message(game,ChatMessageVariant::SomeoneSurvivedYourAttack);
            }
            return false;
        }
        
        self.push_night_message(game, ChatMessageVariant::YouWereAttacked);
        for attacker in attacker_refs.iter() {
            attacker.push_night_message(game,ChatMessageVariant::YouAttackedSomeone);
        }

        self.push_night_grave_killers(game, grave_killer);
            
        if should_leave_death_note {
            for attacker in attacker_refs.iter() {
                if let Some(note) = attacker.death_note(game) {
                    self.push_night_grave_death_notes(game, note.clone());
                }
            }
        }
        

        if !self.alive(game) { return true }

        self.set_night_died(game, true);

        true
    }
    pub fn try_night_kill_no_attacker(&self, game: &mut Game, grave_killer: GraveKiller, attack: AttackPower) -> bool {
        self.try_night_kill(
            &VecSet::new(),
            game,
            grave_killer,
            attack,
            false
        )
    }

    /**
    ### Example use in witch case
        
    fn do_night_action(self, game: &mut Game, actor_ref: PlayerReference, priority: Priority) {
        if let Some(currently_used_player) = actor_ref.possess_night_action(game, priority, self.currently_used_player){
            actor_ref.set_role_state(game, RoleState::Witch(Witch{
                currently_used_player: Some(currently_used_player)
            }))
        }
    }
    */
    pub fn possess_night_action(&self, game: &mut Game, priority: Priority, currently_used_player: Option<PlayerReference>)->Option<PlayerReference>{
        match priority {
            Priority::Possess => {
                let untagged_possessor_visits = self.untagged_night_visits_cloned(game);
                let Some(possessed_visit) = untagged_possessor_visits.get(0) else {return None};
                let Some(possessed_into_visit) = untagged_possessor_visits.get(1) else {return None};
                
                possessed_visit.target.push_night_message(game,
                    ChatMessageVariant::YouWerePossessed { immune: possessed_visit.target.possession_immune(game) }
                );
                if possessed_visit.target.possession_immune(game) {
                    self.push_night_message(game,
                        ChatMessageVariant::TargetIsPossessionImmune
                    );
                    return None;
                }


                //change all controller inputs to be selecting this player as well
                for (controller_id, controller_data) in game.saved_controllers.all_controllers().clone().iter() {
                    match controller_data.selection() {
                        AbilitySelection::Unit => {},
                        AbilitySelection::Boolean { .. } => {
                            if possessed_visit.target == possessed_into_visit.target {
                                SavedControllersMap::set_selection_in_controller(
                                    game,
                                    possessed_visit.target,
                                    controller_id.clone(),
                                    AbilitySelection::new_boolean(true),
                                    true
                                );
                            }
                        },
                        AbilitySelection::TwoPlayerOption { selection } => {

                            let mut selection = selection.0.clone();
                            if let Some((_, second)) = selection {
                                selection = Some((possessed_into_visit.target, second));
                            }

                            SavedControllersMap::set_selection_in_controller(
                                game,
                                possessed_visit.target,
                                controller_id.clone(),
                                AbilitySelection::new_two_player_option(selection),
                                true
                            );
                        },
                        AbilitySelection::PlayerList { selection } => {

                            let mut selection = selection.0.clone();
                            if let Some(first) = selection.first_mut(){
                                *first = possessed_into_visit.target;
                            }else{
                                selection = vec![possessed_into_visit.target];
                            }


                            SavedControllersMap::set_selection_in_controller(
                                game,
                                possessed_visit.target,
                                controller_id.clone(),
                                AbilitySelection::new_player_list(selection),
                                true
                            );
                        },
                        AbilitySelection::RoleOption { .. } => {},
                        AbilitySelection::TwoRoleOption { .. } => {},
                        AbilitySelection::TwoRoleOutlineOption { .. } => {},
                        AbilitySelection::String { .. } => {},
                        AbilitySelection::Integer { .. } => {},
                        AbilitySelection::Kira { .. } => {},
                    }
                }

                possessed_visit.target.set_night_visits(game,
                    possessed_visit.target.convert_selection_to_visits(game)
                );

                //remove the second role visit
                let mut found_first = false;
                let mut new_witch_visits = vec![];
                for visit in self.all_night_visits_cloned(game){
                    if !found_first || visit.tag != VisitTag::Role {
                        new_witch_visits.push(visit);
                    }
                    if visit.tag == VisitTag::Role {
                        found_first = true;
                    }
                }

                self.set_night_visits(game, new_witch_visits);
                return Some(possessed_visit.target);
            },
            Priority::Investigative => {
                if let Some(currently_used_player) = currently_used_player {
                    self.push_night_message(game,
                        ChatMessageVariant::TargetHasRole { role: currently_used_player.role(game) }
                    );
                }
                return None;
            },
            Priority::StealMessages => {
                if let Some(currently_used_player) = currently_used_player {
                    for message in currently_used_player.night_messages(game).clone() {
                        self.push_night_message(game,
                            ChatMessageVariant::TargetsMessage { message: Box::new(message.clone()) }
                        );
                    }
                }
                return None;
            },
            _ => {
                return None;
            }
        }
    }

    /// ### Pre condition:
    /// self.alive(game) == false
    pub fn die(&self, game: &mut Game, grave: Grave){
        if let Some(event) = self.die_return_event(game, grave){
            event.invoke(game);
        }
    }
    /// if the player is already dead, this does nothing and returns none
    pub fn die_return_event(&self, game: &mut Game, grave: Grave)->Option<OnAnyDeath>{
        if !self.alive(game) { return None }
        self.set_alive(game, false);
        self.add_private_chat_message(game, ChatMessageVariant::YouDied);
        game.add_grave(grave.clone());

        Some(OnAnyDeath::new(*self))
    }
    pub fn initial_role_creation(&self, game: &mut Game){
        let new_role_data = self.role(&game).new_state(&game);
        self.set_role_state(game, new_role_data.clone());
        self.on_role_creation(game);    //this function can change role state
        if new_role_data.role() == self.role(game) {
            self.add_private_chat_message(game, ChatMessageVariant::RoleAssignment{role: self.role(game)});
        }
        self.set_win_condition(game, self.win_condition(game).clone());
        InsiderGroupID::set_player_revealed_groups(
            InsiderGroupID::all_insider_groups_with_player(game, *self), 
            game, *self
        );
    }
    pub fn set_role(&self, game: &mut Game, new_role_data: impl Into<RoleState>) {
        let new_role_data = new_role_data.into();
        let old = self.role_state(game).clone();
        BeforeRoleSwitch::new(*self, old.clone(), new_role_data.clone()).invoke(game);

        self.set_role_state(game, new_role_data.clone());
        self.on_role_creation(game);    //this function can change role state
        if new_role_data.role() == self.role(game) {
            self.add_private_chat_message(game, ChatMessageVariant::RoleAssignment{role: self.role(game)});
        }

        OnRoleSwitch::new(*self, old, self.role_state(game).clone()).invoke(game);
    }
    /// Swaps this persons role, sends them the role chat message, and makes associated changes
    pub fn set_role_and_win_condition_and_revealed_group(&self, game: &mut Game, new_role_data: impl Into<RoleState>){
        let new_role_data = new_role_data.into();
        
        self.set_role(game, new_role_data);
    
        self.set_win_condition(game, self.role_state(game).clone().default_win_condition());
        
        InsiderGroupID::set_player_revealed_groups(
            self.role_state(game).clone().default_revealed_groups(), 
            game, *self
        );
        
    }
    pub fn increase_defense_to(&self, game: &mut Game, defense: DefensePower){
        if defense.is_stronger(self.night_defense(game)) {
            self.set_night_upgraded_defense(game, Some(defense));
        }
    }
    
    
    pub fn tracker_seen_visits(self, game: &Game) -> Vec<Visit> {
        if let Some(v) = self.night_appeared_visits(game) {
            v.clone()
        } else {
            self.all_night_visits_cloned(game)
        }
    }
    pub fn all_appeared_visitors(self, game: &Game) -> Vec<PlayerReference> {
        PlayerReference::all_players(game).filter(|player_ref|{
            player_ref.tracker_seen_visits(game).iter().any(|other_visit| 
                other_visit.target == self
            )
        }).collect()
    }

    pub fn push_night_messages_to_player(&self, game: &mut Game){
        let mut messages = self.night_messages(game).to_vec();
        messages.shuffle(&mut rand::rng());
        messages.sort();
        self.send_packet(game, ToClientPacket::NightMessages { chat_messages: 
            messages.iter().map(|msg|ChatMessage::new_private(msg.clone())).collect()
        });
        self.add_private_chat_messages(game, messages);
    }

    pub fn chosen_vote(&self, game: &Game) -> Option<PlayerReference> {
        if let Some(PlayerListSelection(players)) =game.saved_controllers.get_controller_current_selection_player_list(ControllerID::nominate(*self)) {
            Some(players.first().cloned()).flatten()
        }else{
            None
        }
    }

    pub fn role_label_map(&self, game: &Game) -> VecMap<PlayerReference, Role> {
        let mut map = VecMap::new();
        for player in self.role_labels(game).iter() {
            map.insert(*player, player.role(game));
        }
        map
    }

    pub fn ability_deactivated_from_death(&self, game: &Game) -> bool {
        !(
            self.alive(game) ||
            (
                PlayerReference::all_players(game).any(|p|
                    if let RoleState::Coxswain(c) = p.role_state(game) {
                        c.targets.contains(self)
                    }else{
                        false
                    }
                )
            )
        )
    }
    pub fn defense(&self, game: &Game) -> DefensePower {
        if game.current_phase().is_night() {
            self.night_defense(game)
        }else{
            self.role(game).defense()
        }
    }
    pub fn possession_immune(&self, game: &Game) -> bool {
        self.role(game).possession_immune()
    }
    pub fn has_innocent_aura(&self, game: &Game) -> bool {
        PlayerReference::all_players(game).into_iter().any(|player_ref| 
            match player_ref.role_state(game) {
                RoleState::Disguiser(r) => 
                    r.current_target.is_some_and(|player|player == *self),
                _ => false
            }
        ) ||
        self.role(game).has_innocent_aura(game)
    }
    pub fn has_suspicious_aura(&self, game: &Game) -> bool {
        self.role(game).has_suspicious_aura(game) || 
        self.night_framed(game) ||
        DrunkAura::has_drunk_aura(game, *self) ||
        ArsonistDoused::has_suspicious_aura_douse(game, *self)
    }
    pub fn get_won_game(&self, game: &Game) -> bool {
        match self.win_condition(game){
            WinCondition::GameConclusionReached { win_if_any } => {
                if let Some(resolution) = GameConclusion::game_is_over(game) {
                    win_if_any.contains(&resolution)
                } else {
                    false
                }
            },
            WinCondition::RoleStateWon => {
                match self.role_state(game) {
                    RoleState::Jester(r) => r.won(),
                    RoleState::Doomsayer(r) => r.won(),
                    RoleState::Revolutionary(r) => r.won(),
                    RoleState::Chronokaiser(_) => Chronokaiser::won(game, *self),
                    RoleState::Martyr(r) => r.won(),
                    _ => false
                }
            },
        }
    }
    /// If they can consistently kill then they keep the game running
    /// Town kills by voting
    /// Mafia kills with MK or gun
    /// Cult kills / converts
    pub fn keeps_game_running(&self, game: &Game) -> bool {
        if InsiderGroupID::Mafia.is_player_in_revealed_group(game, *self) {return true;}
        if InsiderGroupID::Cult.is_player_in_revealed_group(game, *self) {return true;}
        if self.win_condition(game).is_loyalist_for(GameConclusion::Town) {return true;}
        
        GameConclusion::keeps_game_running(self.role(game))
    }

    /*
        Role functions
    */

    pub fn controller_parameters_map(&self, game: &Game) -> ControllerParametersMap {
        self.role_state(game).clone().controller_parameters_map(game, *self)
    }
    pub fn do_night_action(&self, game: &mut Game, priority: Priority) {
        self.role_state(game).clone().do_night_action(game, *self, priority)
    }
    pub fn on_role_creation(&self, game: &mut Game) {
        self.role_state(game).clone().on_role_creation(game, *self)
    }
    pub fn get_current_send_chat_groups(&self, game: &Game) -> HashSet<ChatGroup> {
        if Modifiers::modifier_is_enabled(game, ModifierType::NoChat)
            || (
                Modifiers::modifier_is_enabled(game, ModifierType::NoNightChat) 
                && self.alive(game)
                && matches!(game.current_phase().phase(), PhaseType::Night | PhaseType::Obituary)
            )
        {
            return HashSet::new()
        }
        self.role_state(game).clone().get_current_send_chat_groups(game, *self)
    }
    pub fn get_current_receive_chat_groups(&self, game: &Game) -> HashSet<ChatGroup> {
        self.role_state(game).clone().get_current_receive_chat_groups(game, *self)
    }
    pub fn convert_selection_to_visits(&self, game: &Game) -> Vec<Visit> {
        self.role_state(game).clone().convert_selection_to_visits(game, *self)
    }
}



