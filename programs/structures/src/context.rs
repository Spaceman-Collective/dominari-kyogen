use anchor_lang::prelude::*;
use core_ds::{account::{MaxSize, RegistryInstance, Entity}, program::CoreDs};
use registry::{constant::SEEDS_REGISTRYSIGNER, account::{RegistryConfig, ActionBundleRegistration, RegistryIndex}, program::Registry};
use kyogen::{account::{Config as KyogenConfig, Blueprint, InstanceIndex as KyogenIndex}, constant::ENTITY_ID_SIZE};
use anchor_spl::{associated_token::AssociatedToken, token::{Token, TokenAccount, Mint}};

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
            registry_instance.key().as_ref(),
        ],
        bump,
        space= 8 + StructureIndex::get_max_size() as usize,
    )]
    pub structures_index: Account<'info, StructureIndex>,

    // CPI into Registry to Register this AB
    //Registry
    pub registry_program: Program<'info, Registry>,
    #[account(mut)]
    pub registry_index: Box<Account<'info, RegistryIndex>>,
    #[account(mut)]
    pub ab_registration: Box<Account<'info, ActionBundleRegistration>>,

    // Kyogen
    #[account(
        constraint = kyogen_index.instance == instance
    )]
    pub kyogen_index:Account<'info, KyogenIndex>,

    // CoreDS
    pub registry_instance: Account<'info, RegistryInstance>,

    // SPL
    #[account(
        init,
        payer = payer,
        associated_token::mint = game_token,
        associated_token::authority = structures_index
    )]
    pub structures_index_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(
        address = kyogen_index.config.game_token
    )]
    pub game_token: Account<'info, Mint>,
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
    pub structure: AccountInfo<'info>,
    pub tile: Account<'info, Entity>
}

#[derive(Accounts)]
pub struct UseMeteor<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // SPL
    #[account(
        mut,
        associated_token::mint = game_token,
        associated_token::authority = payer,
    )]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = game_token,
        associated_token::authority = structures_index,
    )]
    pub structures_ata: Account<'info, TokenAccount>,
    #[account(
        address = kyogen_index.config.game_token
    )]
    pub game_token: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    // Structures
    #[account(
        constraint = structures_index.instance == kyogen_index.instance
    )]
    pub structures_index: Box<Account<'info, StructureIndex>>,
    pub structures_config: Box<Account<'info, Config>>,

    // Kyogen
    pub kyogen_config: Box<Account<'info, KyogenConfig>>,
    pub kyogen_index: Box<Account<'info, KyogenIndex>>,

    // Registry
    pub registry_config: Account<'info, RegistryConfig>,
    pub registry_program: Program<'info, Registry>,
    pub structures_registration: Box<Account<'info, ActionBundleRegistration>>,

    // CoreDS 
    pub coreds: Program<'info, CoreDs>, 
    pub registry_instance: Account<'info, RegistryInstance>,


    #[account(mut)]
    pub meteor: Box<Account<'info, Entity>>,
    pub tile: Box<Account<'info, Entity>>,
    pub unit: Box<Account<'info, Entity>>

}

#[derive(Accounts)]
pub struct UsePortal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // SPL
    #[account(
        mut,
        associated_token::mint = game_token,
        associated_token::authority = payer,
    )]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = game_token,
        associated_token::authority = kyogen_index,
    )]
    pub kyogen_ata: Account<'info, TokenAccount>,
    #[account(
        address = kyogen_index.config.game_token
    )]
    pub game_token: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    // Structures
    #[account(
        constraint = structures_index.instance == kyogen_index.instance
    )]
    pub structures_index: Box<Account<'info, StructureIndex>>,
    pub structures_config: Box<Account<'info, Config>>,

    // Kyogen
    pub kyogen_config: Box<Account<'info, KyogenConfig>>,
    pub kyogen_index: Box<Account<'info, KyogenIndex>>,

    // Registry
    pub registry_config: Account<'info, RegistryConfig>,
    pub registry_program: Program<'info, Registry>,
    pub structures_registration: Box<Account<'info, ActionBundleRegistration>>,

    // CoreDS 
    pub coreds: Program<'info, CoreDs>, 
    pub registry_instance: Account<'info, RegistryInstance>,


    #[account(mut)]
    pub from: Box<Account<'info, Entity>>, 
    #[account(mut)]
    pub from_portal: Box<Account<'info, Entity>>,
    #[account(mut)]
    pub to: Box<Account<'info, Entity>>,
    pub to_portal: Box<Account<'info, Entity>>,
    #[account(mut)]
    pub unit: Box<Account<'info, Entity>>,
}
