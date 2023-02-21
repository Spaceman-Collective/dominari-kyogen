use std::collections::BTreeMap;

use anchor_lang::prelude::*;
use core_ds::{account::MaxSize, state::SerializedComponent};

use crate::state::*;
use crate::constant::*;

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub components: KyogenComponentKeys,
}

impl MaxSize for Config {
    fn get_max_size() -> u64 {
        return 32 + KyogenComponentKeys::get_max_size();
    }
}

#[account]
pub struct Blueprint {
    pub name: String,
    pub components: BTreeMap<Pubkey, SerializedComponent> // Component => Data
}

impl MaxSize for Blueprint {
    fn get_max_size() -> u64 {
        return STRING_MAX_SIZE + 4;
    }
}

#[account]
pub struct Pack {
    pub name: String,
    pub blueprints: Vec<Pubkey>
}

impl MaxSize for Pack {
    fn get_max_size() -> u64 {
        return STRING_MAX_SIZE + 4;
    }
}

/**
 * Always needs a map for an instance
 * Init during Init Map
 * Then realloc+ on entity spawn
 */
#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
#[account]
pub struct InstanceIndex {
    pub authority: Pubkey,
    pub config: GameConfig,
    pub map: u64,
    pub tiles: Vec<u64>,
    pub units: Vec<u64>,
    pub players: Vec<u64>,
    pub play_phase: PlayPhase
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub enum PlayPhase {
    Lobby,
    Play,
    Paused,
    Finished
}

/**
 * DOES NOT INCLUDE GAME CONFIG SIZE
 * To fetch that, use the get_max_size() function on the config object
 * This is because it's dynamically allocated based on # of starting cards passed in
 */
impl MaxSize for InstanceIndex {
    fn get_max_size() -> u64 {
        return 32+8+4+4+4+2;
    }
}

/**
 * Starting Cards is a pointer to a PACK NAME in the "starting_cards" Pack
 */
#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
pub struct GameConfig {
    pub max_players: u16,
    pub game_token: Pubkey, //MINT for SPL token used to pay for things
    pub spawn_claim_multiplier: f64,
}

impl MaxSize for GameConfig {
    fn get_max_size() -> u64 {
        return 2 + 32 + 8;
    }
}
