use anchor_lang::prelude::*;
use core_ds::{account::{MaxSize, RegistryInstance, Entity}, program::CoreDs};
use registry::{constant::SEEDS_REGISTRYSIGNER, account::{RegistryConfig, ActionBundleRegistration, RegistryIndex}, program::Registry};
use kyogen::{account::{Config as KyogenConfig, Blueprint}, constant::ENTITY_ID_SIZE};

use crate::account::*;
use crate::constant::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    #[account(
        init,
        payer=payer,
        seeds=[SEEDS_STRUCTURESSIGNER],
        bump,
        space= 8 + Config::get_max_size() as usize,
    )]
    pub config: Account<'info, Config>
}

#[derive(Accounts)]
#[instruction(instance:u64)]
pub struct InitStructureIndex<'info>{
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    #[account(
        // Only admin can create new indexs
        constraint = payer.key() == config.authority.key(),
        seeds=[SEEDS_STRUCTURESSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        init,
        payer=payer,
        seeds=[
            SEEDS_PREFIXINDEX,
            instance.to_be_bytes().as_ref(),
        ],
        bump,
        space= 8 + StructureIndex::get_max_size() as usize,
    )]
    pub structure_index: Account<'info, StructureIndex>,

    // CPI into Registry to Register this AB
    //Registry
    pub registry_program: Program<'info, Registry>,
    pub registry_index: Box<Account<'info, RegistryIndex>>,
    pub ab_registration: Box<Account<'info, ActionBundleRegistration>>,
}

#[derive(Accounts)]
pub struct InitStructure<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Structures
    #[account(
        // Only admin can create new games
        constraint = payer.key() == config.authority.key(),
        seeds=[SEEDS_STRUCTURESSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
        mut,
        realloc = structure_index.to_account_info().data_len() + ENTITY_ID_SIZE,
        realloc::payer = payer,
        realloc::zero = false,
    )]
    pub structure_index: Box<Account<'info, StructureIndex>>,

    // Kyogen
    pub kyogen_config: Box<Account<'info, KyogenConfig>>,
    pub structure_blueprint: Box<Account<'info, Blueprint>>,

    // Registry
    #[account(
        seeds = [SEEDS_REGISTRYSIGNER.as_slice()],
        bump,
        seeds::program = registry::id()
    )]
    pub registry_config: Account<'info, RegistryConfig>,
    pub registry_program: Program<'info, Registry>,
    pub structure_registration: Box<Account<'info, ActionBundleRegistration>>,
    
    // Core DS
    pub coreds: Program<'info, CoreDs>, 
    pub registry_instance: Account<'info, RegistryInstance>,

    /// CHECK: Initalized through CPI
    #[account(mut)]
    pub structure_entity: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UseSpawn<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Structures
    
    // Kyogen
    
    // Registry
    
    // CoreDS 

    pub spawn_entity: Box<Account<'info, Entity>>,
}


#[derive(Accounts)]
pub struct MeteorKickOff<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Structures
    #[account(
        // Only admin can kick off meteors
        constraint = payer.key() == config.authority.key(),
        seeds=[SEEDS_STRUCTURESSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,
    // Kyogen
    pub kyogen_config: Box<Account<'info, KyogenConfig>>, // need for "active" component key

    // Registry
    #[account(
        seeds = [SEEDS_REGISTRYSIGNER.as_slice()],
        bump,
        seeds::program = registry::id()
    )]
    pub registry_config: Account<'info, RegistryConfig>,
    pub registry_program: Program<'info, Registry>,
    pub structure_registration: Box<Account<'info, ActionBundleRegistration>>,

    // CoreDS
    pub coreds: Program<'info, CoreDs>, 
    pub registry_instance: Account<'info, RegistryInstance>,
    #[account(mut)]
    pub meteor_entity: Box<Account<'info, Entity>>,

    // Clockwork

}

