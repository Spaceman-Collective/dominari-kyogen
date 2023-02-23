use std::str::FromStr;
use registry::constant::SEEDS_REGISTRYSIGNER;
use wasm_bindgen::prelude::*;
use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use serde_wasm_bindgen::{to_value, from_value};
use anchor_lang::system_program::ID as system_program;

#[wasm_bindgen]
pub struct Registry {
    #[wasm_bindgen(skip)]
    pub registry_id:Pubkey,
    #[wasm_bindgen(skip)]
    pub payer: Pubkey,
    #[wasm_bindgen(skip)]
    pub coreds: Pubkey,
}

// Instructions
#[wasm_bindgen]
impl Registry {
    #[wasm_bindgen(constructor)]
    pub fn new(coreds:&str, registry_id:&str, payer: &str) -> Self {
        console_error_panic_hook::set_once();
        Registry {
            coreds: Pubkey::from_str(coreds).unwrap(),
            registry_id: Pubkey::from_str(registry_id).unwrap(),
            payer: Pubkey::from_str(payer).unwrap()
        }
    }

    // Initialize
    pub fn initialize(&self) -> JsValue {
        let payer = self.payer;
        let registry_config = Registry::get_registry_signer(&self.registry_id);

        let ix = Instruction {
            program_id: self.registry_id,
            accounts: registry::accounts::Initialize {
                payer, 
                registry_config,
                system_program,
            }.to_account_metas(None),
            data: registry::instruction::Initialize {
                core_ds: self.coreds
            }.data()
        };

        to_value(&ix).unwrap()
    }

    // Register Component
    pub fn register_component(&self, schema:String) -> JsValue {
        let payer = self.payer;
        let registry_config = Registry::get_registry_signer(&self.registry_id);

        let component = Pubkey::find_program_address(&[
            registry::constant::SEEDS_COMPONENTREGISTRATION,
            schema.as_bytes().as_ref(),
        ], &self.registry_id).0;

        let ix = Instruction {
            program_id: self.registry_id,
            accounts: registry::accounts::RegisterComponent {
                payer, 
                registry_config,
                system_program,
                component,
            }.to_account_metas(None),
            data: registry::instruction::RegisterComponent {
                schema,
            }.data()
        };

        to_value(&ix).unwrap()
    }

    // Register Action Bundle
    /**
     * @param ab_signer This is the AB Signer PDA, not the program address of the AB
     */
    pub fn register_action_bundle(&self, ab_signer: &str) -> JsValue {
        let payer = self.payer;
        let action_bundle_signer = Pubkey::from_str(ab_signer).unwrap();

        let action_bundle_registration = Pubkey::find_program_address(&[
            registry::constant::SEEDS_ACTIONBUNDLEREGISTRATION,
            action_bundle_signer.to_bytes().as_ref(),
        ], &self.registry_id).0;

        let ix = Instruction {
            program_id: self.registry_id,
            accounts: registry::accounts::RegisterAB {
                payer, 
                system_program,
                action_bundle_registration,
                action_bundle_signer,
            }.to_account_metas(None),
            data: registry::instruction::RegisterActionBundle {}.data()
        };

        to_value(&ix).unwrap()
    }
    
    // Add Components To Action Bundle Registration
    /**
     * @param component_list is a list of string schema urls or names
     */
    pub fn add_components_to_action_bundle_registration(&self, ab_signer:&str, component_list:JsValue) -> JsValue {
        let components_str: Vec<String> = from_value(component_list).unwrap();
        let components:Vec<Pubkey> = components_str.iter().map(|comp_str| {
            Pubkey::find_program_address(&[
                registry::constant::SEEDS_COMPONENTREGISTRATION,
                comp_str.as_bytes().as_ref(),
            ], &self.registry_id).0    
        }).collect();

        let payer = self.payer;
        let config = Registry::get_registry_signer(&self.registry_id);
        let action_bundle_signer = Pubkey::from_str(ab_signer).unwrap();
        let action_bundle_registration = Pubkey::find_program_address(&[
            registry::constant::SEEDS_ACTIONBUNDLEREGISTRATION,
            action_bundle_signer.to_bytes().as_ref(),
        ], &self.registry_id).0;

        let ix = Instruction {
            program_id: self.registry_id,
            accounts: registry::accounts::AddComponentsToActionBundleRegistration {
                payer, 
                system_program,
                config,
                action_bundle_registration,
            }.to_account_metas(None),
            data: registry::instruction::AddComponentsToActionBundleRegistration {
                components,
            }.data()
        };

        to_value(&ix).unwrap()
    }
}

#[wasm_bindgen]
impl Registry {
    pub fn get_registry_signer_str(registry_id:&str) -> String {
        Registry::get_registry_signer(
            &Pubkey::from_str(&registry_id).unwrap()
        ).to_string()
    }
}

impl Registry {
    pub fn get_registry_signer(registry_id:&Pubkey) -> Pubkey {
        return Pubkey::find_program_address(&[SEEDS_REGISTRYSIGNER], &registry_id).0;
    }
}