
use kyogen::component::*;
use structures::component::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BlueprintJson {
    // Kyogen Components
    pub metadata: Option<ComponentMetadataJSON>, // Uses Pubkey
    pub mapmeta: Option<ComponentMapMeta>, // Not used in blueprints
    pub location: Option<ComponentLocation>, // Not used in blueprints
    pub spawn: Option<ComponentSpawn>,
    pub occupant: Option<ComponentOccupant>, // Not used in blueprints
    pub player_stats: Option<ComponentPlayerStats>, // Uses Pubkey but won't ever be in a blueprint
    pub owner: Option<ComponentOwner>, // Uses pubkey but not used in blueprints
    pub last_used: Option<ComponentLastUsed>,
    pub range: Option<ComponentRange>,
    pub health: Option<ComponentHealth>,
    pub damage: Option<ComponentDamage>,
    pub troop_class: Option<TroopClass>,
    pub active: Option<ComponentActive>,
    pub image: Option<ComponentImage>,
    // Stuctures Components
    pub structure: Option<ComponentStructureJSON>, // Uses Pubkey
}

#[derive(Deserialize, Debug)]
pub struct ComponentMetadataJSON {
    pub name: String
}

#[derive(Deserialize, Debug)]
pub struct ComponentStructureJSON {
    pub cost: u64,
    pub structure: StructureTypeJSON
}

#[derive(Deserialize, Debug)]
pub enum StructureTypeJSON {
    Portal {
        channel: u8,
    },
    Healer {
        heal_amt: u64,
    },
    Lootable {
        pack: String
    },
    Meteor {
        solarite_per_use: u64
    }
}