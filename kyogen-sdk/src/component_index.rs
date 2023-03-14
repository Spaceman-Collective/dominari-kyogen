use std::str::FromStr;

use wasm_bindgen::prelude::*;
use anchor_lang::prelude::*;
use registry::constant::SEEDS_COMPONENTREGISTRATION;
use bimap::BiHashMap;
use kyogen::state::KyogenComponentKeys;
use structures::state::StructuresComponentKeys;

#[wasm_bindgen]
pub struct ComponentIndex {
    #[wasm_bindgen(skip)]
    pub registry_id: Pubkey,
    #[wasm_bindgen(skip)]
    pub index: BiHashMap<String, Pubkey>
}

#[wasm_bindgen]
impl ComponentIndex {
    #[wasm_bindgen(constructor)]
    pub fn new(registry_id:&str) -> Self {
        console_error_panic_hook::set_once();
        ComponentIndex { 
            registry_id: Pubkey::from_str(registry_id).unwrap(), 
            index: ComponentIndex::get_initial_hashmap(Pubkey::from_str(registry_id).unwrap()) 
        }
    }

    pub fn insert_component_url(&mut self, schema:&str) {
        let pubkey = Pubkey::find_program_address(&[
            SEEDS_COMPONENTREGISTRATION,
            schema.as_bytes().as_ref(),
        ], &self.registry_id).0;

        self.index.insert(String::from(schema), pubkey);
    }

    pub fn get_component_pubkey_as_str(&self, schema:&str) -> String {
        let pubkey = Pubkey::find_program_address(&[
            SEEDS_COMPONENTREGISTRATION,
            schema.as_bytes().as_ref(),
        ], &self.registry_id).0;
        
        return pubkey.to_string();
    }

    pub fn get_component_pubkey(&self, schema:&str) -> Pubkey {
        self.index.get_by_left(&String::from(schema)).unwrap().clone()
    }

    pub fn get_component_url(&self, pubkey:&str) -> String {
        self.index.get_by_right(&Pubkey::from_str(pubkey).unwrap()).unwrap().clone()
    }
}

impl ComponentIndex {
    pub fn get_initial_hashmap(registry_id:Pubkey) -> BiHashMap<String, Pubkey> {
        let mut map = bimap::BiHashMap::<String, Pubkey>::new();
        let components_urls = vec![
            "metadata",       // All entities
            "mapmeta",        // Map
            "location",       // Tile, Structure
            "spawn",          // Tile,
            "occupant",       // Tile
            "player_stats",   // Player
            "owner",          // Troop
            "last_used",      // Troop
            "range",          // Troop
            "health",         // Troop
            "damage",         // Troop
            "troop_class",    // Troop
            "active",         // Troop
            "image",          // All
            "structure"  // Structure       
        ];

        for url in components_urls {
            let pubkey = Pubkey::find_program_address(&[
                SEEDS_COMPONENTREGISTRATION,
                url.as_bytes().as_ref(),
            ], &registry_id).0;
            map.insert(String::from(url), pubkey);
        }
        return map;
    } 

    pub fn get_kyogen_relevant_keys(&self) -> KyogenComponentKeys {
        KyogenComponentKeys { 
            metadata: self.get_component_pubkey(&"metadata".to_string()), 
            mapmeta: self.get_component_pubkey(&"mapmeta".to_string()), 
            location: self.get_component_pubkey(&"location".to_string()), 
            spawn: self.get_component_pubkey(&"spawn".to_string()), 
            occupant: self.get_component_pubkey(&"occupant".to_string()), 
            player_stats: self.get_component_pubkey(&"player_stats".to_string()), 
            owner: self.get_component_pubkey(&"owner".to_string()), 
            last_used: self.get_component_pubkey(&"last_used".to_string()), 
            range: self.get_component_pubkey(&"range".to_string()), 
            health: self.get_component_pubkey(&"health".to_string()), 
            damage: self.get_component_pubkey(&"damage".to_string()), 
            troop_class: self.get_component_pubkey(&"troop_class".to_string()),
            active: self.get_component_pubkey(&"active".to_string()), 
            image: self.get_component_pubkey(&"image".to_string()) 
        }
    }

    pub fn get_structures_relevant_keys(&self) -> StructuresComponentKeys {
        StructuresComponentKeys { 
            structure: self.get_component_pubkey(&"structure".to_string())
        }
    }

}