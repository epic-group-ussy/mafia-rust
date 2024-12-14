use std::{collections::HashSet, vec};

use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};
use vec1::{
    vec1,
    Vec1
};

use crate::vec_set::VecSet;

use super::role::Role;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleList(pub Vec<RoleOutline>);
impl RoleList {
    pub fn create_random_roles(&self, enabled_roles: &VecSet<Role>) -> Option<Vec<Role>> {
        let mut taken_roles = Vec::new();
        for entry in self.0.iter(){
            if let Some(role) = entry.get_random_role(enabled_roles, &taken_roles){
                taken_roles.push(role);
            }else{
                return None;
            }
        }
        Some(taken_roles)
    }
    pub fn simplify(&mut self){
        for entry in self.0.iter_mut(){
            entry.simplify();
        }
    }
    pub fn sort(&mut self){
        self.0.sort_by_key(|r| r.get_roles().len());
    }
}



#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RoleOutline{
    #[default]
    Any,
    RoleOutlineOptions{
        options: Vec1<RoleOutlineOption>
    },
}
impl RoleOutline{
    pub fn new_exact(role: Role)->RoleOutline{
        RoleOutline::RoleOutlineOptions{options: vec1![RoleOutlineOption::Role{role}]}
    }
    pub fn get_roles(&self) -> Vec<Role> {
        match self {
            RoleOutline::RoleOutlineOptions{options} => 
                options.iter().flat_map(|r| r.get_roles()).collect(),
            RoleOutline::Any => 
                Role::values(),
        }
    }
    pub fn get_random_role(&self, enabled_roles: &VecSet<Role>, taken_roles: &[Role]) -> Option<Role> {
        let options = self.get_roles().into_iter().filter(|r|role_can_generate(*r, enabled_roles, taken_roles)).collect::<Vec<_>>();
        options.choose(&mut rand::thread_rng()).cloned()
    }
    pub fn simplify(&mut self){
        if let RoleOutline::RoleOutlineOptions{options} = self {
            let mut new_options = options.to_vec();

            new_options = new_options.into_iter().collect::<HashSet<_>>().into_iter().collect();

            for option_a in options.iter(){
                for option_b in options.iter(){
                    if option_a.is_subset(option_b) && option_a != option_b{
                        new_options.retain(|r| r != option_a);
                    }
                }
            }

            let mut new_options = Vec1::try_from_vec(new_options)
                .expect("It is impossible to have two sets that are not equal but are subsets of each other, role_list.rs: RoleOutline::simplify");

            new_options.sort();

            *self = RoleOutline::RoleOutlineOptions{options: new_options};
        }
    }
}



#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RoleOutlineOption {
    #[serde(rename_all = "camelCase")]
    RoleSet{role_set: RoleSet},
    #[serde(rename_all = "camelCase")]
    Role{role: Role},
}
impl RoleOutlineOption{
    pub fn get_roles(&self) -> Vec<Role> {
        match self {
            RoleOutlineOption::RoleSet { role_set } => {
                role_set.get_roles()
            }
            RoleOutlineOption::Role { role } => 
                vec![*role]
        }
    }
    pub fn is_subset(&self, other: &RoleOutlineOption) -> bool {
        self.get_roles().iter().all(|r|other.get_roles().contains(r))
    }
}
impl PartialOrd for RoleOutlineOption {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RoleOutlineOption {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.get_roles().len().cmp(&self.get_roles().len())
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum RoleSet {
    Town,
    TownCommon,
    TownInvestigative,
    TownProtective,
    TownKilling,
    TownSupport,

    Mafia,
    MafiaSupport,
    MafiaKilling,

    Cult,
    Fiends,
    
    Neutral,
    Minions
}
impl RoleSet{
    pub fn get_roles(&self) -> Vec<Role> {
        match self {
            RoleSet::Town => 
                vec![
                    Role::Jailor, Role::Villager, Role::Drunk
                ].into_iter().chain(
                    RoleSet::TownCommon.get_roles().into_iter()
                ).collect(),
            RoleSet::TownCommon => {
                RoleSet::TownInvestigative.get_roles().into_iter()
                .chain(
                    RoleSet::TownProtective.get_roles().into_iter()
                ).chain(
                    RoleSet::TownKilling.get_roles().into_iter()
                ).chain(
                    RoleSet::TownSupport.get_roles().into_iter()
                ).collect()
            },
            RoleSet::TownInvestigative => 
                vec![
                    Role::Detective, Role::Philosopher, Role::Gossip, 
                    Role::Psychic, Role::Auditor, Role::Spy, 
                    Role::Lookout, Role::Tracker, Role::Snoop,
                    Role::TallyClerk
                ],
            RoleSet::TownProtective => 
                vec![
                    Role::Bodyguard, Role::Cop, Role::Doctor,
                    Role::Bouncer, Role::Engineer, Role::Armorsmith,
                    Role::Steward
                ],
            RoleSet::TownKilling => 
                vec![
                    Role::Vigilante, Role::Veteran, Role::Deputy, Role::Marksman, Role::Rabblerouser
                ],
            RoleSet::TownSupport => 
                vec![Role::Medium, Role::Retributionist, Role::Transporter, Role::Escort, Role::Mayor, Role::Reporter],
            RoleSet::Mafia =>
                vec![
                    Role::Goon, Role::MafiaSupportWildcard, Role::MafiaKillingWildcard
                ].into_iter().chain(
                    RoleSet::MafiaKilling.get_roles().into_iter()
                ).chain(
                    RoleSet::MafiaSupport.get_roles().into_iter()
                ).collect(),
            RoleSet::MafiaKilling => 
                vec![
                    Role::Godfather, Role::Counterfeiter,
                    Role::Impostor, Role::Recruiter,
                    Role::Mafioso
                ],
            RoleSet::MafiaSupport => 
                vec![
                    Role::Blackmailer, Role::Informant, Role::Hypnotist, Role::Consort,
                    Role::Forger, Role::Framer, Role::Mortician, Role::Disguiser,
                    Role::MafiaWitch, Role::Necromancer, Role::Cupid, Role::Reeducator,
                ],
            RoleSet::Minions => 
                vec![
                    Role::Witch, Role::Scarecrow, Role::Warper, Role::Kidnapper
                ],
            RoleSet::Neutral =>
                vec![
                    Role::Jester, Role::Revolutionary, Role::Geist, Role::Politician, Role::Doomsayer,
                    Role::Martyr, Role::Chronokaiser
                ],
            RoleSet::Fiends =>
                vec![
                    Role::Arsonist, Role::Werewolf, Role::Ojo,
                    Role::Puppeteer, Role::Pyrolisk, Role::Kira,
                    Role::SerialKiller, Role::FiendsWildcard,
                    Role::Spiral
                ],
            RoleSet::Cult =>
                vec![
                    Role::Apostle, Role::Disciple, Role::Zealot
                ],
        }
    }
}



pub fn role_can_generate(role: Role, enabled_roles: &VecSet<Role>, taken_roles: &[Role]) -> bool {
    if !enabled_roles.contains(&role) {
        return false;
    }

    match role.maximum_count() {
        Some(max) => taken_roles.iter().filter(|r|**r==role).count() < max.into(),
        None => true,
    }
}