use anchor_lang::prelude::*;

use crate::constant::*;
use core_ds::account::MaxSize;

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentMetadata{
    pub name: String,
    pub entity_type: EntityType,
    pub registry_instance: Pubkey
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub enum EntityType {
    Map,
    Unit,
    Feature,
    Tile,
    Player
}

impl MaxSize for ComponentMetadata {
    fn get_max_size() -> u64 {
        return STRING_MAX_SIZE + STRING_MAX_SIZE + 32
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentMapMeta{
    pub max_x: u8,
    pub max_y: u8,
}

impl MaxSize for ComponentMapMeta {
    fn get_max_size() -> u64 {
        return 1 + 1
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentLocation {
    pub x: u8,
    pub y: u8
}

impl MaxSize for ComponentLocation {
    fn get_max_size() -> u64 {
        return 1 + 1
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentOwner{
    pub owner: Option<Pubkey>,    // Keypair for Tile Owner
    pub player: Option<u64>    // Entity ID for Tile Owner's Player
}

impl MaxSize for ComponentOwner {
    fn get_max_size() -> u64 {
        return 1+32+1+8
    }
}


#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentOccupant{
    pub occupant_id: Option<u64>
}

impl MaxSize for ComponentOccupant {
    fn get_max_size() -> u64 {
        return 1+8
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentPlayerStats{
    pub name: String,
    pub key: Pubkey, //owner key
    pub score: u64,
    pub kills: u64,
    // Blueprints for Cards, PDA for Pack, String for Blueprint name
    pub cards: Vec<Pubkey>, 
}

impl MaxSize for ComponentPlayerStats {
    fn get_max_size() -> u64 {
        return STRING_MAX_SIZE+32+8+8+4+(32*PLAYER_MAX_CARDS)
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentLastUsed{
    pub last_used: u64, // Slot last used in
    pub recovery: u64 // How many slots til it can be used again
}

impl MaxSize for ComponentLastUsed {
    fn get_max_size() -> u64 {
        return 8+8
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentRange{
    pub movement: u8,
    pub attack_range: u8,
}

impl MaxSize for ComponentRange {
    fn get_max_size() -> u64 {
        return 1+1
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentHealth{
    pub health: u64,
}

impl MaxSize for ComponentHealth {
    fn get_max_size() -> u64 {
        return 8
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentDamage{
    pub min_damage: u64,
    pub max_damage: u64,
    pub bonus_samurai: u32,
    pub bonus_sohei: u32,
    pub bonus_shinobi: u32,
}

impl MaxSize for ComponentDamage {
    fn get_max_size() -> u64 {
        return 8 + 8 + 4 + 4 + 4
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentTroopClass{
    pub class: TroopClass,
}

impl MaxSize for ComponentTroopClass {
    fn get_max_size() -> u64 {
        return 1+1
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum TroopClass {
    Samurai,
    Sohei,
    Shinobi,
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentActive{
    pub active: bool,
}

impl MaxSize for ComponentActive {
    fn get_max_size() -> u64 {
        return 1
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentOffchainMetadata{
    pub link: String,
}

impl MaxSize for ComponentOffchainMetadata {
    fn get_max_size() -> u64 {
        return STRING_MAX_SIZE*2 //can be 2 times regular string for long url links
    }
}
