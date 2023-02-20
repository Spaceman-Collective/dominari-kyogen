use std::collections::BTreeMap;

use anchor_lang::prelude::*;
use core_ds::{account::{MaxSize, RegistryInstance}, state::SerializedComponent, program::CoreDs};
use registry::{constant::SEEDS_REGISTRYSIGNER, account::{RegistryConfig, ActionBundleRegistration}, program::Registry};
use crate::{constant::*, account::*};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    #[account(
        init,
        payer=payer,
        seeds=[SEEDS_KYOGENSIGNER],
        bump,
        space= 8 + Config::get_max_size() as usize,
    )]
    pub config: Account<'info, Config>
}

#[derive(Accounts)]
#[instruction(name:String, blueprints: Vec<Pubkey>)]
pub struct RegisterPack<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(
        constraint = payer.key() == config.authority.key()
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        init,
        payer=payer,
        seeds=[
            SEEDS_PACK,
            name.as_bytes()
        ],
        bump,
        space = 8 + Pack::get_max_size() as usize
    )]
    pub pack: Account<'info, Pack>
}

#[derive(Accounts)]
#[instruction(name:String, blueprint: BTreeMap<Pubkey, SerializedComponent>)]
pub struct RegisterBlueprint<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(
        constraint = payer.key() == config.authority.key()
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        init,
        payer=payer,
        seeds=[
            SEEDS_BLUEPRINT,
            name.as_bytes()
        ],
        bump,
        space = 8 + Pack::get_max_size() as usize
    )]
    pub blueprint: Account<'info, Blueprint>
}

#[derive(Accounts)]
#[instruction(instance: u64, game_config: GameConfig)]
pub struct CreateGameInstance<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Action Bundle
    #[account(
        // Only admin can create new games
        constraint = payer.key() == config.authority.key(),
        seeds=[SEEDS_KYOGENSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,
    
    #[account(
        init,
        payer=payer,
        seeds=[
            SEEDS_INSTANCEINDEX,
            registry_instance.key().as_ref()
        ],
        bump,
        space= 8 + InstanceIndex::get_max_size() as usize + GameConfig::get_max_size() as usize
    )]
    pub instance_index: Box<Account<'info, InstanceIndex>>,    

    //Registry
    #[account(
        seeds = [SEEDS_REGISTRYSIGNER.as_slice()],
        bump,
        seeds::program = registry::id()
    )]
    pub registry_config: Account<'info, RegistryConfig>,
    pub registry_program: Program<'info, Registry>,

    /// CHECK: Created via CPI in the registry program
    pub registry_index: AccountInfo<'info>,

    //CoreDs
    pub coreds: Program<'info, CoreDs>, 
    /// CHECK: Created via CPI in the coreds program
    #[account(mut)]
    pub registry_instance: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ChangeGameState<'info> {
    pub payer: Signer<'info>,
    #[account(
        // Only admin can create new games
        constraint = payer.key() == config.authority.key(),
    )]
    pub config: Box<Account<'info, Config>>,

    pub instance_index: Box<Account<'info, InstanceIndex>>,
}

#[derive(Accounts)]
pub struct InitMap<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Kyogen
    #[account(
        // Only admin can create new games
        constraint = payer.key() == config.authority.key(),
        seeds=[SEEDS_KYOGENSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(mut)]
    pub instance_index: Box<Account<'info, InstanceIndex>>,
    
    // Registry
    #[account(
        seeds = [SEEDS_REGISTRYSIGNER.as_slice()],
        bump,
        seeds::program = registry::id()
    )]
    pub registry_config: Account<'info, RegistryConfig>,
    pub registry_program: Program<'info, Registry>,
    pub kyogen_registration: Box<Account<'info, ActionBundleRegistration>>,
    
    // Core DS
    pub coreds: Program<'info, CoreDs>, 
    pub registry_instance: Account<'info, RegistryInstance>,

    /// CHECK: Initalized through CPI
    #[account(mut)]
    pub map_entity: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitTile<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Kyogen
    #[account(
        // Only admin can create new games
        constraint = payer.key() == config.authority.key(),
        seeds=[SEEDS_KYOGENSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
        mut,
        realloc = instance_index.to_account_info().data_len() + ENTITY_ID_SIZE,
        realloc::payer = payer,
        realloc::zero = false,
    )]
    pub instance_index: Box<Account<'info, InstanceIndex>>,
    
    // Registry
    #[account(
        seeds = [SEEDS_REGISTRYSIGNER.as_slice()],
        bump,
        seeds::program = registry::id()
    )]
    pub registry_config: Account<'info, RegistryConfig>,
    pub registry_program: Program<'info, Registry>,
    pub kyogen_registration: Box<Account<'info, ActionBundleRegistration>>,
    
    // Core DS
    pub coreds: Program<'info, CoreDs>, 
    pub registry_instance: Account<'info, RegistryInstance>,

    /// CHECK: Initalized through CPI
    #[account(mut)]
    pub tile_entity: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitPlayer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Kyogen
    #[account(
        // Anyone can initialize themselves
        seeds=[SEEDS_KYOGENSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
        mut,
        realloc = instance_index.to_account_info().data_len() + ENTITY_ID_SIZE,
        realloc::payer = payer,
        realloc::zero = false,
    )]
    pub instance_index: Box<Account<'info, InstanceIndex>>,

    /// CHECK: Pack Name checked inside the ix based on clan that's passed in
    pub pack: Box<Account<'info, Pack>>,

    // Registry
    #[account(
        seeds = [SEEDS_REGISTRYSIGNER.as_slice()],
        bump,
        seeds::program = registry::id()
    )]
    pub registry_config: Account<'info, RegistryConfig>,
    pub registry_program: Program<'info, Registry>,
    pub kyogen_registration: Box<Account<'info, ActionBundleRegistration>>,
    
    // Core DS
    pub coreds: Program<'info, CoreDs>, 
    pub registry_instance: Account<'info, RegistryInstance>,

    /// CHECK: Initalized through CPI
    #[account(mut)]
    pub player_entity: AccountInfo<'info>,
}


////////////////////////////////////////////////////
pub fn compute_blueprint_size(name:&String, map: &BTreeMap<Pubkey, SerializedComponent>) -> u64 {
    let mut size:u64 = name.as_bytes().len() as u64 + 4; // 4 is for empty btreemap
    for (_, value) in map.iter() {
        size += 32; //key size
        size += value.max_size;
    }
    return size;
}