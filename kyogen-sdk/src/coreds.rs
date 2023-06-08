use anchor_lang::prelude::Pubkey;
use core_ds::constant::{SEEDS_ENTITY_PREFIX, SEEDS_REGISTRYINSTANCE_PREFIX};

/**
 * Returns PDA of a given Entity ID in a given instance
 */
pub fn get_key_from_id(coreds_id: &Pubkey, registry_instance: &Pubkey, id: u64) -> Pubkey {
    Pubkey::find_program_address(&[
        SEEDS_ENTITY_PREFIX,
        id.to_be_bytes().as_ref(),
        registry_instance.to_bytes().as_ref(),
    ], coreds_id).0
}

pub fn get_registry_instance(coreds_id:&Pubkey, registry:&Pubkey, instance:u64) -> Pubkey {
    Pubkey::find_program_address(&[
        SEEDS_REGISTRYINSTANCE_PREFIX,
        registry.to_bytes().as_ref(),
        instance.to_be_bytes().as_ref()
    ], coreds_id).0
}