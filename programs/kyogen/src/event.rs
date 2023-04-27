use anchor_lang::prelude::*;
use crate::{component::PlayPhase, constant::Clans};

#[event]
pub struct GameStateChanged{
    pub instance: u64,
    pub new_state: PlayPhase
}

#[event]
pub struct NewPlayer{
    pub instance: u64,
    pub player_id: u64,
    pub authority: Pubkey,
    pub clan: Clans
}

#[event]
pub struct SpawnClaimed{
    pub instance: u64,
    pub clan: Clans,
    pub tile: u64,
    pub player: u64,
}

#[event]
pub struct UnitSpawned {
    pub instance: u64,
    pub tile: u64,
    pub player: u64,
    pub unit: u64,
}

#[event]
pub struct UnitMoved {
    pub instance: u64,
    pub unit: u64,
    pub from: u64,
    pub to: u64,
}

#[event]
pub struct UnitAttacked{
    pub instance: u64,
    pub attacker: u64,
    pub defender: u64,
    pub tile: u64,
}