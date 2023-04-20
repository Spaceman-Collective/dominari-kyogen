use solana_client_wasm::WasmClient;
use wasm_bindgen::prelude::*;
use anchor_lang::prelude::*;
use std::str::FromStr;
use kyogen::account::InstanceIndex as KyogenIndex;
use structures::{account::StructureIndex, constant::SEEDS_PREFIXINDEX};

use crate::{coreds::{get_registry_instance, get_key_from_id}, gamestate::fetch_account, json_wrappers::AddressListJSON};

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
    
}

/* Passing in bytes and enum of account type, return JSON object of the thing */