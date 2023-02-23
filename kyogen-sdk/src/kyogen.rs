use std::collections::BTreeMap;
use std::str::FromStr;
use core_ds::account::MaxSize;
use core_ds::state::SerializedComponent;
use kyogen::constant::SEEDS_BLUEPRINT;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{from_value, to_value};
use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use anchor_lang::system_program::ID as system_program;
use crate::blueprint::{BlueprintJson, StructureTypeJSON};
use crate::component_index::ComponentIndex;
use kyogen::component::*;
use structures::component::*;


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
        let structures_ref = component_index.get_structure_relevant_keys();

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
                data: blueprint_components.active.as_ref().unwrap().try_to_vec().unwrap()
            });
        }

        if blueprint_components.structure.is_some() {
            // Convert Structure string to Pubkey
            let structure;
            let structure_json = blueprint_components.structure.unwrap();
            match structure_json.structure {
                StructureTypeJSON::Lootable { pack } => {
                    structure = ComponentStructure {
                        cost: structure_json.cost,
                        structure: StructureType::Lootable { 
                            pack: Pubkey::from_str(pack.as_str()).unwrap() 
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
        let blueprint = Pubkey::find_program_address(&[
            SEEDS_BLUEPRINT,
            name.as_bytes()
        ], &self.kyogen_id).0;

        let ix = Instruction { 
            program_id: self.kyogen_id,
            accounts: kyogen::accounts::RegisterBlueprint {
                payer,
                system_program,
                config,
                blueprint
            }.to_account_metas(None),
            data: kyogen::instruction::RegisterBlueprint {
                name: String::from(name),
                blueprint: components,
            }.data()
        };

        to_value(&ix).unwrap()
    }

    // Register Pack
    // Create Game Instance
    // Change Game State
    // Init Map
    // Init Tile
    // Init Player
    // Claim Spawn
    // Spawn Unit
    // Move Unit
    // Attack Unit
}

// WASM Helper Methods
#[wasm_bindgen]
impl Kyogen {
    pub fn get_kyogen_signer_str(kyogen_id:&str) -> String {
        Kyogen::get_kyogen_signer(
            &Pubkey::from_str(&kyogen_id).unwrap()
        ).to_string()
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