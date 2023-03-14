use serde::{Serialize, Deserialize};
use kyogen::{component::TroopClass, constant::Clans};
use structures::component::StructureType;

#[derive(Deserialize, Debug)]
pub struct GameConfigJSON {
    pub max_players: u16,
    pub game_token: String,
    pub spawn_claim_multiplier: f64,
    pub max_score: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TroopJSON {
    // Metadata
    pub name: String,
    pub id: String,
    // Owner
    pub player_id: String, // u64 as String
    pub player_key: String,
    // Last Used
    pub last_used: String, //u64 as String
    pub recovery: String, //u64
    // Range
    pub movement: u8,
    pub attack_range: u8,
    // Health
    pub health: String, //u64
    pub max_health: String, //u64
    // Damage
    pub min_damage: String, //u64
    pub max_damage: String, //u64
    pub bonus_samurai: String, //u32
    pub bonus_sohei: String, //u32
    pub bonus_shinobi: String, //u32
    // Troop Class
    pub troop_class: TroopClass,
    // Active
    pub active: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerJSON {
    // Metadata
    pub name: String,
    pub id: String,
    // Player Stats
    pub owner: String, //pubkey
    pub solarite: String, //u64
    pub score: String, //u64
    pub cards: Vec<String>, // Vec<PubkeyStrings>
    pub clan: Clans
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TileJSON {
    pub id: String,
    pub x: u8, 
    pub y: u8,
    pub spawnable: bool,
    pub clan: Option<Clans>,
    pub troop: Option<TroopJSON>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StructureJSON {
    pub name: String,
    pub id: String, //u64

    // Location
    pub x: u8,
    pub y: u8,

    // Last Used
    pub last_used: String, //u64,
    pub recovery: String, //u64,
    
    // Active
    pub active: bool,

    // Structure
    pub structure: StructureType,
    pub cost: String, //u64
}


#[derive(Serialize, Deserialize, Debug)]
pub struct MapJSON {
    pub map_id: String,
    pub tiles: Vec<TileJSON>,
    pub portals: Vec<StructureJSON>,
    pub healers: Vec<StructureJSON>,
    pub lootables: Vec<StructureJSON>,
    pub meteors: Vec<StructureJSON>,
}