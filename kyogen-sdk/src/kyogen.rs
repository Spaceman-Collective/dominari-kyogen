use std::str::FromStr;
use anchor_lang::prelude::Pubkey;
use wasm_bindgen::prelude::*;
use crate::component_index::ComponentIndex;

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
    // Register Blueprint
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