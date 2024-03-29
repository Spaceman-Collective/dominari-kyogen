use std::collections::BTreeMap;
use std::str::FromStr;
use core_ds::account::MaxSize;
use core_ds::constant::SEEDS_REGISTRYINSTANCE_PREFIX;
use core_ds::state::SerializedComponent;
use kyogen::account::GameConfig;
use kyogen::constant::*;
use registry::constant::{SEEDS_REGISTRYINDEX, SEEDS_ACTIONBUNDLEREGISTRATION};
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{from_value, to_value};
use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use anchor_lang::system_program::ID as system_program;
use crate::blueprint::{BlueprintJson, StructureTypeJSON};
use crate::component_index::ComponentIndex;
use crate::coreds::get_key_from_id;
use crate::json_wrappers::*;
use crate::registry::Registry;
use kyogen::component::*;
use structures::component::*;
use spl_associated_token_account::{get_associated_token_address, ID as associated_token_program};
use spl_token::ID as token_program;


#[wasm_bindgen]
#[derive(Default)]
pub struct Kyogen {
    pub core_id: Pubkey,
    pub registry_id: Pubkey,
    pub kyogen_id: Pubkey,
    pub payer: Pubkey
}

// Instructions
#[wasm_bindgen]
impl Kyogen {
    // New 
    #[wasm_bindgen(constructor)]
    pub fn new(core_id:&str, registry_id:&str, kyogen_id:&str, payer:&str) -> Self {
        console_error_panic_hook::set_once();
        Kyogen { 
            core_id: Pubkey::from_str(core_id).unwrap(), 
            registry_id: Pubkey::from_str(registry_id).unwrap(), 
            kyogen_id: Pubkey::from_str(kyogen_id).unwrap(),
            payer: Pubkey::from_str(payer).unwrap() 
        }
    }
    // Initialize
    pub fn initialize(&self, component_index: &ComponentIndex) -> JsValue {
        let payer = self.payer;
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::Initialize {
                payer,
                system_program,
                config,
            }.to_account_metas(None),
            data: kyogen::instruction::Initialize {
                component_keys: component_index.get_kyogen_relevant_keys()
            }.data()
        };

        to_value(&ix).unwrap()
    }

    // Register Blueprint
    pub fn register_blueprint(&self, name:&str, component_index:&ComponentIndex, blueprint_json: JsValue) -> JsValue {
        let blueprint_components: BlueprintJson = from_value(blueprint_json).unwrap();
        let mut components: BTreeMap<Pubkey, SerializedComponent> = BTreeMap::new();
        let kyogen_ref = component_index.get_kyogen_relevant_keys();
        let structures_ref = component_index.get_structures_relevant_keys();

        // Ignoring Blueprint.metadata cause it'll get overwritten anyway
        // also ignoring Mapmeta, Location, occupant, playerstats, and owner as they aren't used in blueprints

        if blueprint_components.spawn.is_some() {
            components.insert(kyogen_ref.spawn, SerializedComponent { 
                max_size: ComponentSpawn::get_max_size(), 
                data: blueprint_components.spawn.as_ref().unwrap().try_to_vec().unwrap()
            });
        }

        if blueprint_components.last_used.is_some() {
            components.insert(kyogen_ref.last_used, SerializedComponent { 
                max_size: ComponentLastUsed::get_max_size(), 
                data: blueprint_components.last_used.as_ref().unwrap().try_to_vec().unwrap()
            });
        }

        if blueprint_components.range.is_some() {
            components.insert(kyogen_ref.range, SerializedComponent { 
                max_size: ComponentRange::get_max_size(), 
                data: blueprint_components.range.as_ref().unwrap().try_to_vec().unwrap()
            });
        }

        if blueprint_components.health.is_some() {
            components.insert(kyogen_ref.health, SerializedComponent { 
                max_size: ComponentHealth::get_max_size(), 
                data: blueprint_components.health.as_ref().unwrap().try_to_vec().unwrap()
            });
        }

        if blueprint_components.damage.is_some() {
            components.insert(kyogen_ref.damage, SerializedComponent { 
                max_size: ComponentDamage::get_max_size(), 
                data: blueprint_components.damage.as_ref().unwrap().try_to_vec().unwrap()
            });
        }

        if blueprint_components.troop_class.is_some() {
            components.insert(kyogen_ref.troop_class, SerializedComponent { 
                max_size: ComponentTroopClass::get_max_size(), 
                data: blueprint_components.troop_class.as_ref().unwrap().try_to_vec().unwrap()
            });
        }

        if blueprint_components.active.is_some() {
            components.insert(kyogen_ref.active, SerializedComponent { 
                max_size: ComponentActive::get_max_size(), 
                data: blueprint_components.active.as_ref().unwrap().try_to_vec().unwrap()
            });
        }

        if blueprint_components.image.is_some() {
            components.insert(kyogen_ref.image, SerializedComponent { 
                max_size: ComponentActive::get_max_size(), 
                data: blueprint_components.image.as_ref().unwrap().try_to_vec().unwrap()
            });
        }

        if blueprint_components.structure.is_some() {
            // Convert Structure string to Pubkey
            let structure;
            let structure_json = blueprint_components.structure.unwrap();
            match structure_json.structure {
                StructureTypeJSON::Lootable { 
                    ancients_pack,
                    wildings_pack,
                    creepers_pack,
                    synths_pack
                } => {
                    structure = ComponentStructure {
                        cost: structure_json.cost,
                        structure: StructureType::Lootable { 
                            ancients_pack: Pubkey::find_program_address(&[
                                SEEDS_PACK,
                                ancients_pack.as_bytes().as_ref(),
                            ], &self.kyogen_id).0,
                            wildings_pack: Pubkey::find_program_address(&[
                                SEEDS_PACK,
                                wildings_pack.as_bytes().as_ref(),
                            ], &self.kyogen_id).0,
                            creepers_pack: Pubkey::find_program_address(&[
                                SEEDS_PACK,
                                creepers_pack.as_bytes().as_ref(),
                            ], &self.kyogen_id).0, 
                            synths_pack: Pubkey::find_program_address(&[
                                SEEDS_PACK,
                                synths_pack.as_bytes().as_ref(),
                            ], &self.kyogen_id).0,
                        }
                    }
                }
                StructureTypeJSON::Healer { heal_amt }=> {
                    structure = ComponentStructure {
                        cost: structure_json.cost,
                        structure: StructureType::Healer { heal_amt }
                    }
                }
                StructureTypeJSON::Portal { channel }=> {
                    structure = ComponentStructure {
                        cost:structure_json.cost,
                        structure: StructureType::Portal { channel }
                    }
                }
                StructureTypeJSON::Meteor { solarite_per_use }=> {
                    structure = ComponentStructure {
                        cost: structure_json.cost,
                        structure: StructureType::Meteor { solarite_per_use }
                    }
                }
            }

            components.insert(structures_ref.structure, SerializedComponent { 
                max_size: ComponentStructure::get_max_size(), 
                data: structure.try_to_vec().unwrap()
            });
        }

        let payer = self.payer;
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let blueprint_acc = Pubkey::find_program_address(&[
            SEEDS_BLUEPRINT,
            name.as_bytes()
        ], &self.kyogen_id).0;

        let ix = Instruction { 
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::RegisterBlueprint {
                payer,
                system_program,
                config,
                blueprint_acc
            }.to_account_metas(None),
            data: kyogen::instruction::RegisterBlueprint {
                name: String::from(name),
                blueprint: components,
            }.data()
        };

        to_value(&ix).unwrap()
    }

    // Register Pack
    /**
     * Pass in a pubkey strings of the blueprints
     */
    pub fn register_pack(&self, name:String, blueprints_list: JsValue) -> JsValue {
        let pubkey_list:Vec<String> = from_value(blueprints_list).unwrap();
        let blueprints:Vec<Pubkey> = pubkey_list.iter().map(|key| {
            Pubkey::from_str(key.as_str()).unwrap()
        }).collect();

        let payer = self.payer;
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        
        let pack = Pubkey::find_program_address(&[
            SEEDS_PACK,
            name.as_str().as_bytes().as_ref(),
        ], &self.kyogen_id).0;

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::RegisterPack {
                payer,
                system_program,
                config,
                pack
            }.to_account_metas(None),
            data: kyogen::instruction::RegisterPack {
                name,
                blueprints
            }.data()
        };
        to_value(&ix).unwrap()
    }

    // Create Game Instance
    pub fn create_game_instance(&self, instance: u64, game_config_json: JsValue) -> JsValue {
        let game_config: GameConfigJSON = from_value(game_config_json).unwrap();
        let payer = self.payer;

        // CoreDS
        let coreds = self.core_id;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        // Registry
        let registry_config = Registry::get_registry_signer(&self.registry_id);
        let registry_program = self.registry_id;
        let registry_index = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINDEX,
            instance.to_be_bytes().as_ref(),
        ], &self.registry_id).0;

        // Action Bundle
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let instance_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // ATA for Mint
        let game_token = Pubkey::from_str(game_config.game_token.as_str()).unwrap();
        let instance_token_account = get_associated_token_address(
            &instance_index, 
            &game_token
        );

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::CreateGameInstance {
                payer,
                system_program,
                coreds, 
                registry_instance,
                registry_config,
                registry_program,
                registry_index,
                config,
                instance_index,
                token_program,
                associated_token_program,
                game_token,
                instance_token_account
            }.to_account_metas(None),
            data: kyogen::instruction::CreateGameInstance {
                instance,
                game_config: GameConfig {
                    max_players: game_config.max_players,
                    max_score: game_config.max_score,
                    game_token,
                    spawn_claim_multiplier: game_config.spawn_claim_multiplier
                }
            }.data()
        };
        to_value(&ix).unwrap()
    }

    // Change Game State
    pub fn change_game_state(&self, instance: u64, map_id: u64, play_phase_str: &str) -> JsValue {
        let payer = self.payer;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        let new_game_state: PlayPhase;
        match play_phase_str {
            "Lobby" => new_game_state = PlayPhase::Lobby,
            "Play" => new_game_state = PlayPhase::Play,
            "Paused" => new_game_state = PlayPhase::Paused,
            "Finished" => new_game_state = PlayPhase::Finished,
            &_ => new_game_state = PlayPhase::Paused
        }

        // CoreDS
        let coreds = self.core_id;

        let map = get_key_from_id(&self.core_id, &registry_instance, map_id);

        // Action Bundle
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let instance_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // Registry
        let registry_config = Registry::get_registry_signer(&self.registry_id);
        let registry_program = self.registry_id;
        let kyogen_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::ChangeGameState {
                payer,
                config,
                instance_index,
                registry_instance,
                registry_config,
                registry_program,
                kyogen_registration,
                coreds,
                map,
            }.to_account_metas(None),
            data: kyogen::instruction::ChangeGameState {
                new_game_state,
            }.data()
        };
        to_value(&ix).unwrap()
    }

    // Init Map
    pub fn init_map(&self, instance:u64, entity_id: u64, max_x: u8, max_y:u8) -> JsValue {
        let payer = self.payer;

        // CoreDS
        let coreds = self.core_id;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        let map_entity = get_key_from_id(&self.core_id, &registry_instance, entity_id);

        // Action Bundle
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let instance_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // Registry
        let registry_config = Registry::get_registry_signer(&self.registry_id);
        let registry_program = self.registry_id;
        let kyogen_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            config.to_bytes().as_ref(),
        ], &self.registry_id).0;


        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::InitMap {
                payer,
                system_program,
                config, 
                instance_index,
                registry_config,
                registry_program,
                kyogen_registration,
                registry_instance,
                coreds,
                map_entity,
            }.to_account_metas(None),
            data: kyogen::instruction::InitMap {
                entity_id,
                max_x,
                max_y
            }.data()
        };
        to_value(&ix).unwrap()
    }

    // Init Tile
    pub fn init_tile(&self, instance: u64, entity_id: u64, x:u8, y:u8, spawnable: bool, spawn_cost:u64, clan_str: &str) -> JsValue {
        let payer = self.payer;

        // CoreDS
        let coreds = self.core_id;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        let tile_entity = get_key_from_id(&self.core_id, &registry_instance, entity_id);

        // Action Bundle
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let instance_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // Registry
        let registry_config = Registry::get_registry_signer(&self.registry_id);
        let registry_program = self.registry_id;
        let kyogen_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        let clan; 
        match clan_str {
            "Ancients" => {clan = Some(Clans::Ancients)},
            "Wildings" => {clan = Some(Clans::Wildings)},
            "Creepers" => {clan = Some(Clans::Creepers)},
            "Synths" => {clan = Some(Clans::Synths)},
            &_ => {clan = None}
        }

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::InitTile {
                payer,
                system_program,
                config, 
                instance_index,
                registry_config,
                registry_program,
                kyogen_registration,
                registry_instance,
                coreds,
                tile_entity,
            }.to_account_metas(None),
            data: kyogen::instruction::InitTile {
                entity_id,
                x,
                y,
                spawnable,
                spawn_cost,
                clan,
            }.data()
        };
        to_value(&ix).unwrap()
    }

    // Init Player
    pub fn init_player(&self, instance: u64, entity_id:u64, name:String, clan_str: &str) -> JsValue {
        let payer = self.payer;

        // CoreDS
        let coreds = self.core_id;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        let player_entity = get_key_from_id(&self.core_id, &registry_instance, entity_id);

        // Action Bundle
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let instance_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // Registry
        let registry_config = Registry::get_registry_signer(&self.registry_id);
        let registry_program = self.registry_id;
        let kyogen_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        let clan: Clans;
        let pack: Pubkey;

        match clan_str {
            "Ancients" => {
                clan = Clans::Ancients;
                pack = Pubkey::find_program_address(&[
                    SEEDS_PACK,
                    STARTING_CARDS_ANCIENTS_NAME.as_bytes().as_ref(),
                ], &self.kyogen_id).0;
            },
            "Creepers" => {
                clan = Clans::Creepers;
                pack = Pubkey::find_program_address(&[
                    SEEDS_PACK,
                    STARTING_CARDS_CREEPERS_NAME.as_bytes().as_ref(),
                ], &self.kyogen_id).0;
            },
            "Wildings" => {
                clan = Clans::Wildings;
                pack = Pubkey::find_program_address(&[
                    SEEDS_PACK,
                    STARTING_CARDS_WILDINGS_NAME.as_bytes().as_ref(),
                ], &self.kyogen_id).0;

            },
            "Synths" => {
                clan = Clans::Synths;
                pack = Pubkey::find_program_address(&[
                    SEEDS_PACK,
                    STARTING_CARDS_SYNTHS_NAME.as_bytes().as_ref(),
                ], &self.kyogen_id).0;
            },
            &_ => {
                clan = Clans::Ancients;
                pack = Pubkey::find_program_address(&[
                    SEEDS_PACK,
                    STARTING_CARDS_ANCIENTS_NAME.as_bytes().as_ref(),
                ], &self.kyogen_id).0;
            },
        };

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::InitPlayer {
                payer,
                system_program,
                config, 
                instance_index,
                registry_config,
                registry_program,
                kyogen_registration,
                registry_instance,
                coreds,
                player_entity,
                pack,
            }.to_account_metas(None),
            data: kyogen::instruction::InitPlayer {
                entity_id,
                name,
                clan,
            }.data()
        };
        to_value(&ix).unwrap()

    }

    // Claim Spawn
    pub fn claim_spawn(&self, instance:u64, map_id:u64, tile_id: u64, unit_id:u64, player_id:u64, game_token_str: String) -> JsValue {
        let payer = self.payer;

        // CoreDS
        let coreds = self.core_id;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        let unit_entity = get_key_from_id(&self.core_id, &registry_instance, unit_id);
        let tile_entity = get_key_from_id(&self.core_id, &registry_instance, tile_id);
        let player_entity = get_key_from_id(&self.core_id, &registry_instance, player_id);
        let map = get_key_from_id(&self.core_id, &registry_instance, map_id);

        // Action Bundle
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let instance_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // Registry
        let registry_config = Registry::get_registry_signer(&self.registry_id);
        let registry_program = self.registry_id;
        let kyogen_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        // ATA 
        let game_token = Pubkey::from_str(game_token_str.as_str()).unwrap();
        let from_ata = get_associated_token_address(&payer, &game_token);
        let to_ata = get_associated_token_address(&instance_index, &game_token);

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::ClaimSpawn {
                payer,
                system_program,
                config, 
                instance_index,
                registry_config,
                registry_program,
                kyogen_registration,
                registry_instance,
                coreds,
                player_entity,
                tile_entity,
                unit_entity,
                from_ata,
                to_ata,
                token_program,
                map,
            }.to_account_metas(None),
            data: kyogen::instruction::ClaimSpawn {}.data()
        };
        to_value(&ix).unwrap()
    }

    // Spawn Unit
    pub fn spawn_unit(&self, instance:u64, map_id: u64, unit_id:u64, tile_id: u64, player_id: u64, unit_blueprint_str: String) -> JsValue {
        let payer = self.payer;

        // CoreDS
        let coreds = self.core_id;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        let unit = get_key_from_id(&self.core_id, &registry_instance, unit_id);
        let tile = get_key_from_id(&self.core_id, &registry_instance, tile_id);
        let player = get_key_from_id(&self.core_id, &registry_instance, player_id);
        let map = get_key_from_id(&self.core_id, &registry_instance, map_id);

        // Action Bundle
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let instance_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;
        let unit_blueprint = Pubkey::from_str(&unit_blueprint_str.as_str()).unwrap();

        // Registry
        let registry_config = Registry::get_registry_signer(&self.registry_id);
        let registry_program = self.registry_id;
        let kyogen_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::SpawnUnit {
                payer,
                system_program,
                config, 
                instance_index,
                registry_config,
                registry_program,
                kyogen_registration,
                registry_instance,
                coreds,
                player,
                unit,
                tile,
                unit_blueprint,
                map
            }.to_account_metas(None),
            data: kyogen::instruction::SpawnUnit {
                unit_id
            }.data()
        };
        to_value(&ix).unwrap()
    }

    // Move Unit
    pub fn move_unit(&self, instance:u64, map_id:u64, unit_id:u64, player_id:u64, from_tile_id:u64, to_tile_id: u64) -> JsValue {
        let payer = self.payer;

        // CoreDS
        let coreds = self.core_id;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        let unit = get_key_from_id(&self.core_id, &registry_instance, unit_id);
        let player = get_key_from_id(&self.core_id, &registry_instance, player_id);

        let from = get_key_from_id(&self.core_id, &registry_instance, from_tile_id);
        let to = get_key_from_id(&self.core_id, &registry_instance, to_tile_id);
        
        let map = get_key_from_id(&self.core_id, &registry_instance, map_id);

        // Action Bundle
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let instance_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // Registry
        let registry_config = Registry::get_registry_signer(&self.registry_id);
        let registry_program = self.registry_id;
        let kyogen_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::MoveUnit {
                payer,
                system_program,
                config, 
                instance_index,
                registry_config,
                registry_program,
                kyogen_registration,
                registry_instance,
                coreds,
                player,
                unit,
                from,
                to,
                map
            }.to_account_metas(None),
            data: kyogen::instruction::MoveUnit {}.data()
        };
        to_value(&ix).unwrap()
    }

    // Attack Unit
    pub fn attack_unit(&self, instance:u64, map_id: u64, attacker_id: u64, defender_id: u64, defending_tile_id: u64) -> JsValue {
        let payer = self.payer;

        // CoreDS
        let coreds = self.core_id;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        let attacker = get_key_from_id(&self.core_id, &registry_instance, attacker_id);
        let defender = get_key_from_id(&self.core_id, &registry_instance, defender_id);
        let defending_tile = get_key_from_id(&self.core_id, &registry_instance, defending_tile_id);
        let map = get_key_from_id(&self.core_id, &registry_instance, map_id);


        // Action Bundle
        let config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let instance_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // Registry
        let registry_config = Registry::get_registry_signer(&self.registry_id);
        let registry_program = self.registry_id;
        let kyogen_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::AttackUnit {
                payer,
                system_program,
                config, 
                instance_index,
                registry_config,
                registry_program,
                kyogen_registration,
                registry_instance,
                coreds,
                attacker,
                defender,
                defending_tile,
                map
            }.to_account_metas(None),
            data: kyogen::instruction::AttackUnit {}.data()
        };
        to_value(&ix).unwrap()
    }

    // Close Entity
    pub fn close_entity(&self, instance:u64, entity_id:u64) -> JsValue {
        let receiver = self.payer;

        // CoreDS
        let coreds = self.core_id;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        let entity = get_key_from_id(&self.core_id, &registry_instance, entity_id);

        // Action Bundle
        let kyogen_config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let kyogen_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // Registry
        let registry_config = Registry::get_registry_signer(&self.registry_id);
        let registry_program = self.registry_id;
        let kyogen_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            kyogen_config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        let ix = Instruction {
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::CloseEntity {
                receiver,
                system_program,
                kyogen_config, 
                kyogen_index,
                registry_config,
                registry_program,
                kyogen_registration,
                registry_instance,
                coreds,
                entity
            }.to_account_metas(None),
            data: kyogen::instruction::CloseEntity {}.data()
        };
        to_value(&ix).unwrap()
    }
}

// WASM Helper Methods
#[wasm_bindgen]
impl Kyogen {
    pub fn get_kyogen_signer_str(kyogen_id:&str) -> String {
        Kyogen::get_kyogen_signer(
            &Pubkey::from_str(&kyogen_id).unwrap()
        ).to_string()
    }

    pub fn get_pack_key(kyogen_id:&str, name:&str) -> String {
        Pubkey::find_program_address(&[
            SEEDS_PACK,
            name.as_bytes().as_ref(),
        ], &Pubkey::from_str(&kyogen_id).unwrap()).0.to_string()
    }
}

// Non WASM Methods
impl Kyogen {
    pub fn get_kyogen_signer(kyogen_id:&Pubkey) -> Pubkey {
        return Pubkey::find_program_address(&[
            kyogen::constant::SEEDS_KYOGENSIGNER
        ], &kyogen_id).0;
    }

}