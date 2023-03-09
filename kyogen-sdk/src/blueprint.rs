use std::str::FromStr;
use anchor_lang::prelude::*;
use wasm_bindgen::prelude::*;
use kyogen::component::*;
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
        ancients_pack: String,
        wildings_pack: String,
        creepers_pack: String,
        synths_pack: String,
    },
    Meteor {
        solarite_per_use: u64
    }
}

#[wasm_bindgen]
pub struct BlueprintIndex {
    #[wasm_bindgen(skip)]
    pub dominari: Pubkey,
    #[wasm_bindgen(skip)]
    pub index: bimap::BiHashMap<String, Pubkey>
}

#[wasm_bindgen]
impl BlueprintIndex {
    pub fn new(dominari: &str) -> Self {
        BlueprintIndex { dominari: Pubkey::from_str(dominari).unwrap(), index: bimap::BiHashMap::new() }
    }

    pub fn insert_blueprint_name(&mut self, blueprint: String) {
        let pubkey = Pubkey::find_program_address(&[
            kyogen::constant::SEEDS_BLUEPRINT,
            blueprint.as_str().as_bytes().as_ref(),
        ], &self.dominari).0;

        self.index.insert(blueprint, pubkey);
    }

    /**
     * Returns the pubkey if no matching name is found
     * Basically "unkown" Blueprint
     */
    pub fn get_blueprint_name(&self, pubkey:String) -> String {
        let key = Pubkey::from_str(pubkey.as_str()).unwrap();
        let name = self.index.get_by_right(&key);
        if name.is_none() {
            return pubkey;
        } else {
            return name.unwrap().to_owned();
        }
    }

    pub fn get_blueprint_key(&self, blueprint: String) -> String {
        let key = self.index.get_by_left(&blueprint);
        if key.is_none() {
            let pubkey = Pubkey::find_program_address(&[
                kyogen::constant::SEEDS_BLUEPRINT,
                blueprint.as_str().as_bytes().as_ref(),
            ], &self.dominari).0;
            return pubkey.to_string();
        } else {
            return key.unwrap().to_string();
        }
    }
}