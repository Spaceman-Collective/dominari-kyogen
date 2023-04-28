use solana_client_wasm::WasmClient;
use wasm_bindgen::prelude::*;
use anchor_lang::prelude::*;
use std::{str::FromStr};
use kyogen::account::InstanceIndex as KyogenIndex;
use structures::{account::StructureIndex, constant::SEEDS_PREFIXINDEX};
use core_ds::account::Entity;
use kyogen::component::*; 
use structures::component::*;
use crate::{coreds::{get_registry_instance, get_key_from_id}, gamestate::fetch_account, json_wrappers::{AddressListJSON, StructureJSON, TileJSON, TroopJSON, PlayerJSON}, component_index::ComponentIndex};

#[wasm_bindgen]
pub struct StatelessSDK {
    pub kyogen_id: Pubkey,
    pub registry_id: Pubkey,
    pub coreds_id: Pubkey,
    pub structures_id: Pubkey,
    #[wasm_bindgen(skip)]
    pub client: WasmClient,
}

#[wasm_bindgen]
impl StatelessSDK {
    #[wasm_bindgen(constructor)]
    pub fn new(
        rpc:&str,
        kyogen_str: &str,
        registry_str: &str,
        coreds_str: &str,
        structures_str: &str
    ) -> Self {
        console_error_panic_hook::set_once();
        StatelessSDK { 
            kyogen_id: Pubkey::from_str(kyogen_str).unwrap(),
            registry_id: Pubkey::from_str(registry_str).unwrap(),
            coreds_id: Pubkey::from_str(coreds_str).unwrap(),
            structures_id: Pubkey::from_str(structures_str).unwrap(),
            client: WasmClient::new(rpc)
        }
    }

    /*
        Given a instance (gameId) fetch the indexes (Kyogen and Structures)
        Then go through units and structures list to derive all their addresses
        Don't need to fetch all the accounts like in gamestate
     */
    pub async fn fetch_addresses(&self, instance: u64) -> JsValue {
        let registry_instance = get_registry_instance(&self.coreds_id, &self.registry_id, instance);
        let kyogen_index_key = Pubkey::find_program_address(&[
            kyogen::constant::SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;
        let structures_index_key = Pubkey::find_program_address(&[
            SEEDS_PREFIXINDEX,
            registry_instance.key().as_ref(),
        ], &self.structures_id).0;

        let k_idx:KyogenIndex = fetch_account(&self.client, &kyogen_index_key).await.unwrap();
        let s_idx:StructureIndex = fetch_account(&self.client, &structures_index_key).await.unwrap();
        let map_key = get_key_from_id(&self.coreds_id, &registry_instance, k_idx.map);
        
        let tiles = k_idx.tiles.iter().map(|&id| {
            get_key_from_id(&self.coreds_id, &registry_instance, id).to_string()
        }).collect();

        let units = k_idx.units.iter().map(|&id| {
            get_key_from_id(&self.coreds_id, &registry_instance, id).to_string()
        }).collect();

        let players = k_idx.players.iter().map(|&id| {
            get_key_from_id(&self.coreds_id, &registry_instance, id).to_string()
        }).collect();

        let portals = s_idx.portal.iter().map(|&id| {
            get_key_from_id(&self.coreds_id, &registry_instance, id).to_string()
        }).collect();

        let healers = s_idx.healer.iter().map(|&id| {
            get_key_from_id(&self.coreds_id, &registry_instance, id).to_string()
        }).collect();

        let lootables = s_idx.lootable.iter().map(|&id| {
            get_key_from_id(&self.coreds_id, &registry_instance, id).to_string()
        }).collect();

        let meteors = s_idx.meteor.iter().map(|&id| {
            get_key_from_id(&self.coreds_id, &registry_instance, id).to_string()
        }).collect();

        let address_json: AddressListJSON = AddressListJSON {
            kyogen_index: kyogen_index_key.to_string(),
            structures_index: structures_index_key.to_string(),
            map: map_key.to_string(),
            tiles,
            units,
            players,
            portals,
            healers,
            lootables,
            meteors
        };

        serde_wasm_bindgen::to_value(&address_json).unwrap()
    }
    
    pub fn fetch_address_by_id(&self, instance: u64, id: u64) -> String {
        let registry_instance = get_registry_instance(&self.coreds_id, &self.registry_id, instance);
        get_key_from_id(&self.coreds_id, &registry_instance, id).to_string()
    }

    pub fn get_player_json(&self, data:&str, player_id:u64) -> JsValue {
        let player = StatelessSDK::get_player_info(&self.registry_id.to_string(), data, player_id);
        serde_wasm_bindgen::to_value(&player).unwrap()
    }

    pub  fn get_tile_json(&self, data: &str, tile_id:u64) -> JsValue {
        serde_wasm_bindgen::to_value(&StatelessSDK::get_tile_info(&self.registry_id.to_string(), data, tile_id)).unwrap()
    }

    pub fn get_structure_json(&self, data:&str, structure_id:u64) -> JsValue {
        serde_wasm_bindgen::to_value(&StatelessSDK::get_structure_info(&self.registry_id.to_string(), data, structure_id)).unwrap()
    }

    pub fn get_troop_json(&self, data:&str, troop_id:u64) -> JsValue {
        serde_wasm_bindgen::to_value(&StatelessSDK::get_troop_info(&self.registry_id.to_string(), data, troop_id)).unwrap()
    }
}

// Non WASM Methods
impl StatelessSDK {
    pub fn get_structure_info(registry_str:&str, data: &str, id:u64) -> StructureJSON {
        let componet_index = ComponentIndex::new(registry_str);
        let metadata = StatelessSDK::get_metadata(data, &componet_index).unwrap();
        let location = StatelessSDK::get_location(data, &componet_index).unwrap();
        let last_used = StatelessSDK::get_last_used(data, &componet_index).unwrap();
        let active = StatelessSDK::get_active(data, &componet_index).unwrap();
        let structure = StatelessSDK::get_structure(data, &componet_index).unwrap();
   
        StructureJSON { 
            name: metadata.name, 
            id: id.to_string(), 
            x: location.x, 
            y: location.y, 
            last_used: last_used.last_used.to_string(), 
            recovery: last_used.recovery.to_string(), 
            active: active.active, 
            structure: structure.structure, 
            cost: structure.cost.to_string() 
        }
    }

    pub fn get_tile_info(registry_str:&str, data: &str, id:u64) -> TileJSON {
        let componet_index = ComponentIndex::new(registry_str);
        let location = StatelessSDK::get_location(data, &componet_index).unwrap();
        let spawn = StatelessSDK::get_spawn(data, &componet_index).unwrap();
        let occupant = StatelessSDK::get_occupant(data, &componet_index).unwrap();

        let mut tile = TileJSON {
            id: id.to_string(),
            x: location.x,
            y: location.y,
            spawnable: spawn.spawnable,
            clan: spawn.clan,
            troop: None,
        };

        if occupant.occupant_id.is_some() {
            tile.troop = Some(StatelessSDK::get_troop_info(registry_str, data, occupant.occupant_id.unwrap()));
        }

        tile
    }

    pub fn get_troop_info(registry_str:&str, data: &str, id:u64) -> TroopJSON {
        let componet_index = ComponentIndex::new(registry_str);
        let metadata = StatelessSDK::get_metadata(data, &componet_index).unwrap();
        let owner = StatelessSDK::get_owner(data, &componet_index).unwrap();
        let last_used = StatelessSDK::get_last_used(data, &componet_index).unwrap();
        let range = StatelessSDK::get_range(data, &componet_index).unwrap();
        let health = StatelessSDK::get_health(data, &componet_index).unwrap();
        let damage = StatelessSDK::get_damage(data, &componet_index).unwrap();
        let troop_class = StatelessSDK::get_troop_class(data, &componet_index).unwrap();
        let active = StatelessSDK::get_active(data, &componet_index).unwrap();

        TroopJSON {
            name: metadata.name,
            id: id.to_string(),
            player_id: owner.player.unwrap().to_string(),
            player_key: owner.owner.unwrap().to_string(),
            last_used: last_used.last_used.to_string(),
            recovery: last_used.recovery.to_string(),
            movement: range.movement,
            attack_range: range.attack_range,
            health: health.health.to_string(),
            max_health: health.max_health.to_string(),
            min_damage: damage.min_damage.to_string(),
            max_damage: damage.max_damage.to_string(),
            bonus_samurai: damage.bonus_samurai.to_string(),
            bonus_shinobi: damage.bonus_shinobi.to_string(),
            bonus_sohei: damage.bonus_sohei.to_string(),
            troop_class: troop_class.class,
            active: active.active
        }
    }

    pub fn get_player_info(registry_str:&str, data: &str, id:u64) -> PlayerJSON {
        let componet_index = ComponentIndex::new(registry_str);
        let player_stats = StatelessSDK::get_player_stats(data, &componet_index).unwrap();
        
        PlayerJSON {
            name: player_stats.name,
            id: id.to_string(),
            owner: player_stats.key.to_string(),
            solarite: player_stats.solarite.to_string(),
            score: player_stats.score.to_string(),
            cards: player_stats.cards.iter().map(|key| {key.to_string()}).collect(),
            clan: player_stats.clan,
        }
    }
}


/* Passing in bytes and enum of account type, return JSON object of the thing */
// Component Getters
impl StatelessSDK {
    /*** Kyogen Component Getters ***/
    pub fn get_metadata(data: &str, component_index: &ComponentIndex) -> Option<ComponentMetadata> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().metadata.key());
        sc?;
        Some(ComponentMetadata::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_mapmeta(data: &str, component_index: &ComponentIndex) -> Option<ComponentMapMeta> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().mapmeta.key());
        sc?;
        Some(ComponentMapMeta::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_location(data: &str, component_index: &ComponentIndex) -> Option<ComponentLocation> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().location.key());
        sc?;
        Some(ComponentLocation::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_spawn(data: &str, component_index: &ComponentIndex) -> Option<ComponentSpawn> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().spawn.key());
        sc?;
        Some(ComponentSpawn::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_occupant(data: &str, component_index: &ComponentIndex) -> Option<ComponentOccupant> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().occupant.key());
        sc?;
        Some(ComponentOccupant::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_player_stats(data: &str, component_index: &ComponentIndex) -> Option<ComponentPlayerStats> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().player_stats.key());
        sc?;
        Some(ComponentPlayerStats::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_owner(data: &str, component_index: &ComponentIndex) -> Option<ComponentOwner> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().owner.key());
        sc?;
        Some(ComponentOwner::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_last_used(data: &str, component_index: &ComponentIndex) -> Option<ComponentLastUsed> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().last_used.key());
        sc?;
        Some(ComponentLastUsed::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_range(data: &str, component_index: &ComponentIndex) -> Option<ComponentRange> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().range.key());
        sc?;
        Some(ComponentRange::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_health(data: &str, component_index: &ComponentIndex) -> Option<ComponentHealth> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().health.key());
        sc?;
        Some(ComponentHealth::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_damage(data: &str, component_index: &ComponentIndex) -> Option<ComponentDamage> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().damage.key());
        sc?;
        Some(ComponentDamage::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_troop_class(data: &str, component_index: &ComponentIndex) -> Option<ComponentTroopClass> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().troop_class.key());
        sc?;
        Some(ComponentTroopClass::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_active(data: &str, component_index: &ComponentIndex) -> Option<ComponentActive> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().active.key());
        sc?;
        Some(ComponentActive::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_image(data: &str, component_index: &ComponentIndex) -> Option<ComponentImage> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_kyogen_relevant_keys().image.key());
        sc?;
        Some(ComponentImage::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }

    /*** Structure Component Getters ***/

    pub fn get_structure(data: &str, component_index: &ComponentIndex) -> Option<ComponentStructure> {
        let entity: Entity = deserialize_account(&hex::decode(data).unwrap()).unwrap();
        let serialized_components = entity.components;
        let sc = serialized_components.get(&component_index.get_structures_relevant_keys().structure.key());
        sc?;
        Some(ComponentStructure::try_from_slice(sc.unwrap().data.as_slice()).unwrap())
    }
}

pub fn deserialize_account<T: AccountDeserialize>(mut data: &[u8]) -> Result<T> {
    T::try_deserialize(&mut data).map_err(Into::into)
}