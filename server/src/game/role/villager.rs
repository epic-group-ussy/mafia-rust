use serde::Serialize;

use crate::game::role_list::Faction;


use super::RoleStateImpl;

pub(super) const FACTION: Faction = Faction::Town;
pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: u8 = 0;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Villager;

impl RoleStateImpl for Villager {}

