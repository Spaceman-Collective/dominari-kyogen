use std::str::FromStr;
use anchor_lang::prelude::*;
use kyogen::{account::{InstanceIndex as KyogenIndex, PlayPhase}, component::*};
use structures::component::ComponentStructure;
use wasm_bindgen::{prelude::*, throw_str};
use solana_client_wasm::WasmClient;
use web_sys::console;

use crate::{component_index::ComponentIndex, blueprint::BlueprintIndex, coreds::{get_registry_instance, get_key_from_id}, json_wrappers::*};
use core_ds::account::Entity;
use std::collections::HashMap;

#[wasm_bindgen]
pub struct GameState {
    pub kyogen_id: Pubkey,
    pub registry_id: Pubkey,
    pub coreds_id: Pubkey,
    pub instance: u64, 
    #[wasm_bindgen(skip)]
    pub component_index: ComponentIndex,
    #[wasm_bindgen(skip)]
    pub client: WasmClient,
    #[wasm_bindgen(skip)]
    pub kyogen_index: Option<KyogenIndex>,
    #[wasm_bindgen(skip)]
    pub entities: HashMap<u64, Entity>,
    #[wasm_bindgen(skip)]
    pub blueprint_index: BlueprintIndex,
    is_state_loaded: bool,
}

#[wasm_bindgen]
impl GameState {
    #[wasm_bindgen(constructor)]
    pub fn new(
        rpc:&str,
        kyogen_str: &str,
        registry_str: &str,
        coreds_str: &str,
        instance: u64,
    ) -> Self {
        console_error_panic_hook::set_once();
        GameState {
            kyogen_id: Pubkey::from_str(kyogen_str).unwrap(),
            registry_id: Pubkey::from_str(registry_str).unwrap(),
            coreds_id: Pubkey::from_str(coreds_str).unwrap(),
            instance,
            component_index: ComponentIndex::new(registry_str),
            client: WasmClient::new(rpc),
            kyogen_index: None,
            entities: HashMap::new(),
            blueprint_index: BlueprintIndex::new(kyogen_str),
            is_state_loaded: false,
        }
    }

    pub fn add_blueprints(&mut self, blueprints_json: JsValue) {
        let blueprints: Vec<String> = serde_wasm_bindgen::from_value(blueprints_json).unwrap();
        for blueprint in blueprints {
            self.blueprint_index.insert_blueprint_name(blueprint);
        }
    }

    pub fn get_blueprint_name(&self, pubkey:String) -> String {
        self.blueprint_index.get_blueprint_name(pubkey)
    }

    pub fn get_blueprint_key(&self, name:String) -> String {
        self.blueprint_index.get_blueprint_key(name)
    }
    
    pub fn get_play_phase(&self) -> String  {
        match self.kyogen_index.as_ref().unwrap().play_phase {
            PlayPhase::Lobby => return "Lobby".to_string(),
            PlayPhase::Play => return "Play".to_string(),
            PlayPhase::Paused => return "Paused".to_string(),
            PlayPhase::Finished => return "Finished".to_string(),
        }
    }

    pub async fn update_kyogen_index(&mut self) {
        let registry_instance = get_registry_instance(&self.coreds_id, &self.registry_id, self.instance);

        let kyogen_index = Pubkey::find_program_address(&[
            kyogen::constant::SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        let index:KyogenIndex = fetch_account(&self.client, &kyogen_index).await.unwrap();
        self.kyogen_index = Some(index.clone());
    }

    pub async fn load_state(&mut self) {
        let registry_instance = get_registry_instance(&self.coreds_id, &self.registry_id, self.instance);
        self.update_kyogen_index().await;
        
        let mut entities: HashMap<u64, Entity> = HashMap::new();
        entities.insert(
            self.kyogen_index.as_ref().unwrap().map,
            fetch_accounts::<Entity>(
                &self.client,
                &vec![get_key_from_id(
                    &self.coreds_id,
                    &registry_instance,
                    self.kyogen_index.as_ref().unwrap().map
                )]
            ).await.get(0).unwrap().1.to_owned()
        );        
        
        let tile_entity_keys:Vec<Pubkey> = self.kyogen_index.as_ref().unwrap().tiles
                                                                            .clone()
                                                                            .iter()
                                                                            .map(|id| {
                                                                                return get_key_from_id(
                                                                                    &self.coreds_id,
                                                                                    &registry_instance,
                                                                                    id.clone()
                                                                                )
                                                                            })
                                                                            .collect();

        let tile_entities:Vec<(Pubkey, Entity)> = fetch_accounts::<Entity>(&self.client, &tile_entity_keys).await;
        for (i, e) in tile_entities.iter().enumerate() {
            entities.insert(*self.kyogen_index.as_ref().unwrap().tiles.get(i).unwrap(), e.1.to_owned());
        }


        let unit_entity_keys:Vec<Pubkey> = self.kyogen_index.as_ref().unwrap().units
                                                                            .clone()   
                                                                            .iter()
                                                                            .map(|id| {
                                                                                return get_key_from_id(
                                                                                    &self.coreds_id,
                                                                                    &registry_instance,
                                                                                    id.clone()
                                                                                )
                                                                            })
                                                                            .collect();

        let unit_entities:Vec<(Pubkey, Entity)> = fetch_accounts::<Entity>(&self.client, &unit_entity_keys).await;
        for (i, e) in unit_entities.iter().enumerate() {
            entities.insert(*self.kyogen_index.as_ref().unwrap().units.get(i).unwrap(), e.1.to_owned());
        }

        let player_entity_keys:Vec<Pubkey> = self.kyogen_index.as_ref().unwrap().players
                                                                            .clone()   
                                                                            .iter()
                                                                            .map(|id| {
                                                                                return get_key_from_id(
                                                                                    &self.coreds_id,
                                                                                    &registry_instance,
                                                                                    id.clone()
                                                                                )
                                                                            })
                                                                            .collect();

        let player_entities:Vec<(Pubkey, Entity)> = fetch_accounts::<Entity>(&self.client, &player_entity_keys).await;
        for (i, e) in player_entities.iter().enumerate() {
            entities.insert(*self.kyogen_index.as_ref().unwrap().players.get(i).unwrap(), e.1.to_owned());
        }

        // TODO: Load Structures Index
        // TODO: Load Structures Entities

        self.is_state_loaded = true;
        self.entities = entities;
    }

    pub async fn update_entity(&mut self, entity_id:u64) {
        // Don't worry about finding this in index, just fetch the account and update the entities table
        let pubkey = get_key_from_id(
            &self.coreds_id,
            &get_registry_instance(
                &self.coreds_id,
                &self.registry_id,
                self.instance
            ),
            entity_id);
        let entity:Entity = fetch_account(&self.client, &pubkey).await.unwrap();
        self.entities.insert(entity_id, entity);
    }

    pub fn get_tile_id(&self, x:u8, y:u8) -> String {
        if self.kyogen_index.is_none() {
            throw_str("Index isn't built yet!");
        }

        for id in &self.kyogen_index.as_ref().unwrap().tiles {
            let location = self.get_location(id).unwrap_throw();
            if location.x == x && location.y == y {
                return id.clone().to_string();
            }
        }
        throw_str("Tile Not Found!");
    }

    pub  fn get_tile_json(&self, tile_id:u64) -> JsValue {
        return serde_wasm_bindgen::to_value(&self.get_tile_info(tile_id)).unwrap()
    }

    pub fn get_troop_json(&self, troop_id:u64) -> JsValue {
        return serde_wasm_bindgen::to_value(&self.get_troop_info(troop_id)).unwrap()
    }

    pub fn get_map(&self) -> JsValue {
        if self.kyogen_index.is_none() {
            throw_str("Load state first!")
        }
        let mut tiles: Vec<TileJSON> = vec![];

        for tile_id in self.kyogen_index.as_ref().unwrap().tiles.iter() {
            tiles.push(self.get_tile_info(*tile_id));
        }

        let structures: Vec<StructureJSON> = vec![];
        // TODO fetch from Structures Index

        serde_wasm_bindgen::to_value(&MapJSON{
            map_id: self.kyogen_index.as_ref().unwrap().map.to_string(),
            tiles,
            structures,
        }).unwrap()
    }

    pub fn get_players(&self) -> JsValue {
        if self.kyogen_index.is_none() {
            throw_str("Load state first!")
        }
        let mut players: Vec<PlayerJSON> = vec![];

        for player_id in self.kyogen_index.as_ref().unwrap().players.iter() {
            players.push(self.get_player_info(*player_id));
        };

        serde_wasm_bindgen::to_value(&players).unwrap()
    }

    pub fn get_playerjson_by_key(&self, player_key:String) -> JsValue {
        for player_id in self.kyogen_index.as_ref().unwrap().players.iter() {
            let player = self.get_player_info(*player_id);
            if player.owner.eq(&player_key) {
                return serde_wasm_bindgen::to_value(&player).unwrap();
            }
        };

        return serde_wasm_bindgen::to_value(&{}).unwrap();
    }
}

// Non WASM Functions
impl GameState {
    pub fn get_tile_info(&self, id: u64) -> TileJSON {
        let location = self.get_location(&id).unwrap();
        let spawn = self.get_spawn(&id).unwrap();
        let occupant = self.get_occupant(&id).unwrap();

        let mut tile = TileJSON {
            id: id.to_string(),
            x: location.x,
            y: location.y,
            spawnable: spawn.spawnable,
            clan: spawn.clan,
            troop: None,
        };

        if occupant.occupant_id.is_some() {
            tile.troop = Some(self.get_troop_info(occupant.occupant_id.unwrap()));
        }

        tile
    }

    pub fn get_troop_info(&self, id:u64) -> TroopJSON {
        let metadata = self.get_metadata(&id).unwrap();
        let owner = self.get_owner(&id).unwrap();
        let last_used = self.get_last_used(&id).unwrap();
        let range = self.get_range(&id).unwrap();
        let health = self.get_health(&id).unwrap();
        let damage = self.get_damage(&id).unwrap();
        let troop_class = self.get_troop_class(&id).unwrap();
        let active = self.get_active(&id).unwrap();

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

    pub fn get_player_info(&self, id: u64) -> PlayerJSON {
        let player_stats = self.get_player_stats(&id).unwrap();
        
        PlayerJSON {
            name: player_stats.name,
            id: id.to_string(),
            owner: player_stats.key.to_string(),
            solarite: player_stats.solarite.to_string(),
            score: player_stats.score.to_string(),
            cards: player_stats.cards.iter().map(|key| {return key.to_string()}).collect(),
            clan: player_stats.clan,
        }
    }

    //pub fn get_structure_info(&self, id:u64) -> StructureJSON {}
}

// Component Getters
impl GameState {

    /*** Kyogen Component Getters ***/
    pub fn get_metadata(&self, id:&u64) -> Option<ComponentMetadata> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().metadata.key());
        if sc.is_none() { return None };
        Some(ComponentMetadata::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_mapmeta(&self, id:&u64) -> Option<ComponentMapMeta> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&&self.component_index.get_kyogen_relevant_keys().mapmeta.key());
        if sc.is_none() { return None };
        Some(ComponentMapMeta::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_location(&self, id:&u64) -> Option<ComponentLocation> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().location.key());
        if sc.is_none() { return None };
        Some(ComponentLocation::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_spawn(&self, id:&u64) -> Option<ComponentSpawn> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().spawn.key());
        if sc.is_none() { return None };
        Some(ComponentSpawn::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_occupant(&self, id:&u64) -> Option<ComponentOccupant> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().occupant.key());
        if sc.is_none() { return None };
        Some(ComponentOccupant::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_player_stats(&self, id:&u64) -> Option<ComponentPlayerStats> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().player_stats.key());
        if sc.is_none() { return None };
        Some(ComponentPlayerStats::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_owner(&self, id:&u64) -> Option<ComponentOwner> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().owner.key());
        if sc.is_none() { return None };
        Some(ComponentOwner::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_last_used(&self, id:&u64) -> Option<ComponentLastUsed> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().last_used.key());
        if sc.is_none() { return None };
        Some(ComponentLastUsed::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_range(&self, id:&u64) -> Option<ComponentRange> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().range.key());
        if sc.is_none() { return None };
        Some(ComponentRange::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_health(&self, id:&u64) -> Option<ComponentHealth> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().health.key());
        if sc.is_none() { return None };
        Some(ComponentHealth::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_damage(&self, id:&u64) -> Option<ComponentDamage> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().damage.key());
        if sc.is_none() { return None };
        Some(ComponentDamage::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_troop_class(&self, id:&u64) -> Option<ComponentTroopClass> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().troop_class.key());
        if sc.is_none() { return None };
        Some(ComponentTroopClass::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_active(&self, id:&u64) -> Option<ComponentActive> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().active.key());
        if sc.is_none() { return None };
        Some(ComponentActive::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    pub fn get_image(&self, id:&u64) -> Option<ComponentImage> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_kyogen_relevant_keys().image.key());
        if sc.is_none() { return None };
        Some(ComponentImage::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }

    /*** Structure Component Getters ***/

    pub fn get_structure(&self, id:&u64) -> Option<ComponentStructure> {
        let serialized_components = &self.entities.get(&id).unwrap().components;
        let sc = serialized_components.get(&self.component_index.get_structure_relevant_keys().structure.key());
        if sc.is_none() { return None };
        Some(ComponentStructure::try_from_slice(&sc.unwrap().data.as_slice()).unwrap())
    }


}

pub async fn fetch_account<T: AccountDeserialize>(client: &WasmClient, pubkey: &Pubkey) -> Result<T> {
    let mut data:&[u8] = &client.get_account(pubkey).await.unwrap().data;
    let result: Result<T> = deserialize_account(&mut data).await;
    return result;
}

/**
 * Makes the assumption that the accounts returned are in the same order as the keys passed in
 * This is because the accounts returned don't have the pubkey attached to them.
 */
pub async fn fetch_accounts<T: AccountDeserialize>(client: &WasmClient, pubkeys: &Vec<Pubkey>) -> Vec<(Pubkey,T)> {
    let accounts = &client.get_multiple_accounts(pubkeys).await.unwrap();
    let mut results = vec![];
    for (i, account) in accounts.iter().enumerate() {
        let result: Result<T> = deserialize_account(&account.as_ref().unwrap().data).await;
        results.push((pubkeys.get(i).unwrap().clone(), result.unwrap()));
    }
    return results;
}

pub async fn deserialize_account<T: AccountDeserialize>(mut data: &[u8]) -> Result<T> {
    let result = T::try_deserialize(&mut data).map_err(Into::into);
    return result;
}