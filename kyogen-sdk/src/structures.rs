use std::str::FromStr;

use core_ds::constant::SEEDS_REGISTRYINSTANCE_PREFIX;
use kyogen::constant::SEEDS_INSTANCEINDEX;
use registry::constant::{SEEDS_REGISTRYINDEX, SEEDS_ACTIONBUNDLEREGISTRATION};
use spl_associated_token_account::{get_associated_token_address, ID as associated_token_program};
use spl_token::ID as token_program;
use structures::constant::SEEDS_PREFIXINDEX;
use wasm_bindgen::prelude::*;
use anchor_lang::{prelude::*, system_program::ID as system_program, solana_program::instruction::Instruction, InstructionData};
use serde_wasm_bindgen::to_value;
use crate::coreds::get_key_from_id;
use crate::kyogen::*;

use crate::registry::*;
use crate::component_index::ComponentIndex;

#[wasm_bindgen]
#[derive(Default)]
pub struct Structures {
    pub core_id: Pubkey,
    pub registry_id: Pubkey,
    pub kyogen_id: Pubkey,
    pub structures_id: Pubkey,
    pub payer: Pubkey
}


// Instructions
#[wasm_bindgen]
impl Structures {
    // New
    #[wasm_bindgen(constructor)]
    pub fn new(core_id:&str, registry_id:&str, kyogen_id:&str, structures_id:&str, payer:&str) -> Self {
        console_error_panic_hook::set_once();
        Structures {
            core_id: Pubkey::from_str(core_id).unwrap(),
            registry_id: Pubkey::from_str(registry_id).unwrap(), 
            kyogen_id: Pubkey::from_str(kyogen_id).unwrap(),
            structures_id: Pubkey::from_str(structures_id).unwrap(),
            payer: Pubkey::from_str(payer).unwrap(),
        }
    }
    // Initialize
    pub fn initialize(&self, component_index: &ComponentIndex) -> JsValue {
        let payer = self.payer;
        let config = Structures::get_structures_signer(&self.structures_id);

        let ix = Instruction {
            program_id: self.structures_id,
            accounts: structures::accounts::Initialize {
                payer,
                system_program,
                config,
            }.to_account_metas(None),
            data: structures::instruction::Initialize {
                component_keys: component_index.get_structures_relevant_keys()
            }.data()
        };

        to_value(&ix).unwrap()
    }

    // Create Structures Index
    pub fn init_structure_index(&self, instance:u64, game_token_mint:&str) -> JsValue {
        let payer = self.payer;
        let config = Structures::get_structures_signer(&self.structures_id);

        // Registry
        let registry_program = self.registry_id;
        let registry_index = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINDEX,
            instance.to_be_bytes().as_ref(),
        ], &self.registry_id).0;
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;
        
        // Structures
        let structures_index = Pubkey::find_program_address(&[
            SEEDS_PREFIXINDEX,
            registry_instance.key().as_ref(),
        ], &self.structures_id).0;
        let ab_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        // Kyogen
        let kyogen_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // ATA
        let game_token = Pubkey::from_str(game_token_mint).unwrap();
        let structures_index_ata = get_associated_token_address(&structures_index, &game_token);

        let ix = Instruction {
            program_id: self.structures_id,
            accounts: structures::accounts::InitStructureIndex {
                payer,
                system_program,
                config, 
                registry_program,
                registry_index,
                registry_instance,
                structures_index,
                ab_registration,
                kyogen_index,
                structures_index_ata,
                token_program,
                associated_token_program,
                game_token
            }.to_account_metas(None),
            data: structures::instruction::InitIndex {
                _instance: instance,
            }.data()
        };

        to_value(&ix).unwrap()
    }

    // Init Structure
    pub fn init_structure(&self, instance: u64, entity_id: u64, tile_id: u64, x:u8, y:u8, structure_blueprint_key: &str) -> JsValue {
        let payer = self.payer;
        let config = Structures::get_structures_signer(&self.structures_id);

        // Registry
        let registry_program = self.registry_id;
        let registry_config = Registry::get_registry_signer(&self.registry_id);

        // Core DS
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;
        let coreds = self.core_id;
        let structure = get_key_from_id(&self.core_id, &registry_instance, entity_id);
        let tile = get_key_from_id(&self.core_id, &registry_instance, tile_id);


        // Structures
        let structure_index = Pubkey::find_program_address(&[
            SEEDS_PREFIXINDEX,
            registry_instance.key().as_ref(),
        ], &self.structures_id).0;
        let structure_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        // Kyogen
        let kyogen_config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let structure_blueprint = Pubkey::from_str(&structure_blueprint_key).unwrap();

        let ix = Instruction {
            program_id: self.structures_id,
            accounts: structures::accounts::InitStructure {
                payer, 
                system_program,
                config,
                structure_index,
                kyogen_config,
                structure_blueprint,
                registry_config,
                registry_instance,
                registry_program,
                structure_registration,
                coreds,
                structure,
                tile,
            }.to_account_metas(None),
            data: structures::instruction::InitStructure {
                entity_id,
                location: (x, y)
            }.data()
        };

        to_value(&ix).unwrap()
    }

    // Use Meteor
    pub fn use_meteor(&self, instance:u64, meteor_id:u64, tile_id: u64, unit_id:u64, player_id:u64, game_token_mint:&str) -> JsValue {
        let payer = self.payer;
        let structures_config = Structures::get_structures_signer(&self.structures_id);

        // Registry
        let registry_program = self.registry_id;
        let registry_config = Registry::get_registry_signer(&self.registry_id);

        // Core DS
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;
        let coreds = self.core_id;

        let meteor = get_key_from_id(&self.core_id, &registry_instance, meteor_id);
        let tile = get_key_from_id(&self.core_id, &registry_instance, tile_id);
        let unit = get_key_from_id(&self.core_id, &registry_instance, unit_id);
        let player = get_key_from_id(&self.core_id, &registry_instance, player_id);

        // Structures
        let structures_index = Pubkey::find_program_address(&[
            SEEDS_PREFIXINDEX,
            instance.to_be_bytes().as_ref(),
        ], &self.structures_id).0;
        let structures_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            structures_config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        // Kyogen
        let kyogen_config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let kyogen_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // SPL
        let game_token = Pubkey::from_str(&game_token_mint).unwrap();
        let user_ata = get_associated_token_address(&payer, &game_token);
        let structures_ata = get_associated_token_address(&structures_index, &game_token);
    
        let ix = Instruction {
            program_id: self.structures_id,
            accounts: structures::accounts::UseMeteor {
                payer, 
                system_program,
                structures_config,
                kyogen_config,
                registry_config,
                registry_instance,
                registry_program,
                coreds,
                tile,
                game_token,
                user_ata,
                structures_ata,
                token_program,
                associated_token_program,
                structures_index,
                kyogen_index,
                structures_registration,
                meteor,
                unit,
                player
            }.to_account_metas(None),
            data: structures::instruction::UseMeteor {}.data()
        };
        
        to_value(&ix).unwrap()   
    }

    // Use Portal
    pub fn use_portal(
        &self, 
        instance:u64,
        game_token_mint:&str,
        from_tile: u64,
        from_portal: u64,
        to_tile: u64,
        to_portal: u64,
        unit: u64,
    ) -> JsValue {
        let payer = self.payer;
        let structures_config = Structures::get_structures_signer(&self.structures_id);

        // Registry
        let registry_program = self.registry_id;
        let registry_config = Registry::get_registry_signer(&self.registry_id);

        // Core DS
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;
        let coreds = self.core_id;

        let from = get_key_from_id(&self.core_id, &registry_instance, from_tile);
        let from_portal = get_key_from_id(&self.core_id, &registry_instance, from_portal);
        let to = get_key_from_id(&self.core_id, &registry_instance, to_tile);
        let to_portal = get_key_from_id(&self.core_id, &registry_instance, to_portal);
        let unit = get_key_from_id(&self.core_id, &registry_instance, unit);

        // Structures
        let structures_index = Pubkey::find_program_address(&[
            SEEDS_PREFIXINDEX,
            instance.to_be_bytes().as_ref(),
        ], &self.structures_id).0;
        let structures_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            structures_config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        // Kyogen
        let kyogen_config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let kyogen_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // SPL
        let game_token = Pubkey::from_str(&game_token_mint).unwrap();
        let user_ata = get_associated_token_address(&payer, &game_token);
        let kyogen_ata = get_associated_token_address(&kyogen_index, &game_token);


        let ix = Instruction {
            program_id: self.structures_id,
            accounts: structures::accounts::UsePortal {
                payer, 
                system_program,
                structures_config,
                kyogen_config,
                registry_config,
                registry_instance,
                registry_program,
                coreds,
                game_token,
                user_ata,
                kyogen_ata,
                token_program,
                associated_token_program,
                structures_index,
                kyogen_index,
                structures_registration,
                from,
                from_portal,
                to,
                to_portal,
                unit,
            }.to_account_metas(None),
            data: structures::instruction::UsePortal {}.data()
        };

        to_value(&ix).unwrap()  
    }

    // Use Lootable
    pub fn use_lootable(
        &self, 
        instance:u64,
        game_token_mint:&str,
        tile_id:u64,
        unit_id:u64,
        lootable_id: u64,
        player_id: u64,
        pack_key: &str,
    ) -> JsValue {
        let payer = self.payer;
        let structures_config = Structures::get_structures_signer(&self.structures_id);

        // Registry
        let registry_program = self.registry_id;
        let registry_config = Registry::get_registry_signer(&self.registry_id);

        // Core DS
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;
        let coreds = self.core_id;

        let tile = get_key_from_id(&self.core_id, &registry_instance, tile_id);
        let unit = get_key_from_id(&self.core_id, &registry_instance, unit_id);
        let lootable = get_key_from_id(&self.core_id, &registry_instance, lootable_id);
        let player = get_key_from_id(&self.core_id, &registry_instance, player_id);


        // Structures
        let structures_index = Pubkey::find_program_address(&[
            SEEDS_PREFIXINDEX,
            instance.to_be_bytes().as_ref(),
        ], &self.structures_id).0;
        let structures_registration = Pubkey::find_program_address(&[
            SEEDS_ACTIONBUNDLEREGISTRATION,
            structures_config.to_bytes().as_ref(),
        ], &self.registry_id).0;

        // Kyogen
        let kyogen_config = Kyogen::get_kyogen_signer(&self.kyogen_id);
        let kyogen_index = Pubkey::find_program_address(&[
            SEEDS_INSTANCEINDEX,
            registry_instance.to_bytes().as_ref(),
        ], &self.kyogen_id).0;

        // SPL
        let game_token = Pubkey::from_str(&game_token_mint).unwrap();
        let user_ata = get_associated_token_address(&payer, &game_token);
        let kyogen_ata = get_associated_token_address(&kyogen_index, &game_token);


        let ix = Instruction {
            program_id: self.structures_id,
            accounts: structures::accounts::UseLootable {
                payer, 
                system_program,
                structures_config,
                kyogen_config,
                registry_config,
                registry_instance,
                registry_program,
                coreds,
                game_token,
                user_ata,
                kyogen_ata,
                token_program,
                associated_token_program,
                structures_index,
                kyogen_index,
                structures_registration,
                tile,
                unit,
                lootable,
                player,
                pack: Pubkey::from_str(pack_key).unwrap()
            }.to_account_metas(None),
            data: structures::instruction::UseLootable {}.data()
        };

        to_value(&ix).unwrap()  
    }


    // Use Healer
}


// WASM Helper Methods
#[wasm_bindgen]
impl Structures {
    pub fn get_structures_signer_str(structures_id:&str) -> String {
        Structures::get_structures_signer(
            &Pubkey::from_str(&structures_id).unwrap()
        ).to_string()
    }

    pub fn get_structures_index(&self, instance:u64) -> String {
        // Structures
        let registry_instance = Pubkey::find_program_address(&[
            SEEDS_REGISTRYINSTANCE_PREFIX,
            self.registry_id.to_bytes().as_ref(),
            instance.to_be_bytes().as_ref(),
        ], &self.core_id).0;

        let structure_index = Pubkey::find_program_address(&[
            SEEDS_PREFIXINDEX,
            registry_instance.key().as_ref(),
        ], &self.structures_id).0;

        structure_index.to_string()
    }
}

// Non WASM Methods
impl Structures {
    pub fn get_structures_signer(structures_id:&Pubkey) -> Pubkey {
        return Pubkey::find_program_address(&[
            structures::constant::SEEDS_STRUCTURESSIGNER,
        ], &structures_id).0;
    }
}