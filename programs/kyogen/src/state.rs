use anchor_lang::prelude::*;
use core_ds::account::MaxSize;

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
pub struct RelevantComponentKeys {
    pub metadata: Pubkey,       // All entities
    pub mapmeta: Pubkey,        // Map
    pub location: Pubkey,       // Tile
    pub occupant: Pubkey,       // Tile
    pub player_stats: Pubkey,   // Player
    pub owner: Pubkey,          // Troop
    pub last_used: Pubkey,      // Troop
    pub range: Pubkey,          // Troop
    pub health: Pubkey,         // Troop
    pub damage: Pubkey,         // Troop
    pub troop_class: Pubkey,    // Troop
    pub active: Pubkey,         // Troop
    pub offchain_metadata: Pubkey // Any
}

impl MaxSize for RelevantComponentKeys {
    fn get_max_size() -> u64 {
        return 32*13;
    }
}
