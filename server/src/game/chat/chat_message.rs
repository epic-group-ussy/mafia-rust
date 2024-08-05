use serde::Serialize;

use super::{super::player_group::PlayerGroup, chat_message_variant::ChatMessageVariant};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage{
    pub variant: ChatMessageVariant,
    pub chat_group: Option<PlayerGroup>,
}
impl ChatMessage{
    pub fn new(variant: ChatMessageVariant, chat_group: Option<PlayerGroup>)->Self{
        Self{variant,chat_group}
    }
    pub fn new_private(variant: ChatMessageVariant)->Self{
        Self{variant, chat_group: None}
    }
    pub fn new_non_private(variant: ChatMessageVariant, chat_group: PlayerGroup)->Self{
        Self{variant, chat_group: Some(chat_group)}
    }
    pub fn get_variant(&self)->&ChatMessageVariant{
        &self.variant
    }
}