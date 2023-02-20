use anchor_lang::prelude::*;

use core_ds::account::MaxSize;

/*
 * Currently supported Structures
 * -> Spawn Point
 * -> Portal
 * -> Healer
 * -> Loot Box
 */

// Structures have "metadata", "location", "image", "structure", "last used", "active"


#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ComponentStructure {
    pub cost: u64,
    pub structure: StructureType
}

impl MaxSize for ComponentStructure {
    fn get_max_size() -> u64 {
        return 8 + 1 + 32;
    }
}

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum StructureType {
    Portal {
        channel: u8,
    },
    Healer {
        heal_amt: u64,
    },
    Lootable {
        pack: Pubkey
    },
    Meteor {
        solarite_per_use: u64
    }
}
