use std::collections::BTreeMap;

use anchor_lang::prelude::*;
use core_ds::{account::{MaxSize, RegistryInstance, Entity}, state::SerializedComponent, program::CoreDs};
use registry::{constant::SEEDS_REGISTRYSIGNER, account::{RegistryConfig, ActionBundleRegistration}, program::Registry};
use crate::{constant::*, account::*, component::PlayPhase};
use anchor_spl::{associated_token::{get_associated_token_address, AssociatedToken}, token::{Token, TokenAccount, Mint}};

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
        space = 8 + Pack::get_max_size() as usize + (blueprints.len() * 32)
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
        space = 8 + Blueprint::get_max_size() as usize + compute_blueprint_size(&name, &blueprint) as usize
    )]
    pub blueprint_acc: Account<'info, Blueprint>
}

#[derive(Accounts)]
#[instruction(instance: u64, game_config: GameConfig)]
pub struct CreateGameInstance<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Associated Token Account for Mint
    #[account(
        init,
        payer = payer,
        associated_token::mint = game_token,
        associated_token::authority = instance_index
    )]
    pub instance_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(
        address = game_config.game_token
    )]
    pub game_token: Account<'info, Mint>,

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
    #[account(mut)]
    pub registry_index: AccountInfo<'info>,

    //CoreDs
    pub coreds: Program<'info, CoreDs>, 
    /// CHECK: Created via CPI in the coreds program
    #[account(mut)]
    pub registry_instance: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(new_game_state: PlayPhase)]
pub struct ChangeGameState<'info> {
    pub payer: Signer<'info>,

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

    #[account(mut)]
    pub map: Box<Account<'info, Entity>>,
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

#[derive(Accounts)]
pub struct ClaimSpawn<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // SPL Transfer
    #[account(
        mut,
        address = get_associated_token_address(&payer.key(), &instance_index.config.game_token)
    )]
    pub from_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        address = get_associated_token_address(&instance_index.key(), &instance_index.config.game_token)
    )]
    pub to_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,

    // Kyogen
    #[account(
        // Anyone can initialize themselves
        seeds=[SEEDS_KYOGENSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,
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
    
    // CoreDS
    pub coreds: Program<'info, CoreDs>, 
    pub registry_instance: Account<'info, RegistryInstance>,
    pub map: Box<Account<'info, Entity>>,
    #[account(mut)]
    pub tile_entity: Box<Account<'info, Entity>>,
    pub unit_entity: Box<Account<'info, Entity>>,
    pub player_entity: Box<Account<'info, Entity>>,
}

#[derive(Accounts)]
pub struct SpawnUnit <'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Kyogen
    pub unit_blueprint: Box<Account<'info, Blueprint>>,
    #[account(
        mut,
        realloc = instance_index.to_account_info().data_len() + ENTITY_ID_SIZE,
        realloc::payer = payer,
        realloc::zero = false,
    )]
    pub instance_index: Box<Account<'info, InstanceIndex>>,
    #[account(
        // Anyone can initialize themselves
        seeds=[SEEDS_KYOGENSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

    // Registry
    #[account(
        seeds = [SEEDS_REGISTRYSIGNER.as_slice()],
        bump,
        seeds::program = registry::id()
    )]
    pub registry_config: Account<'info, RegistryConfig>,
    pub registry_program: Program<'info, Registry>,
    pub kyogen_registration: Box<Account<'info, ActionBundleRegistration>>,
  
    // Core Ds
    pub coreds: Program<'info, CoreDs>, 
    pub registry_instance: Account<'info, RegistryInstance>,
    pub map: Box<Account<'info, Entity>>,
    /// CHECK: Created via CPI
    #[account(mut)]
    pub unit: AccountInfo<'info>,
    #[account(mut)]
    pub tile: Box<Account<'info, Entity>>,
    #[account(mut)]
    pub player: Box<Account<'info, Entity>>,
}

#[derive(Accounts)]
pub struct MoveUnit<'info>{
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Kyogen
    pub instance_index: Box<Account<'info, InstanceIndex>>,
    #[account(
        // Anyone can initialize themselves
        seeds=[SEEDS_KYOGENSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

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
   
    pub map: Box<Account<'info, Entity>>,
    #[account(mut)]
    pub from: Box<Account<'info, Entity>>,
    #[account(mut)]
    pub to: Box<Account<'info, Entity>>,
    #[account(mut)]
    pub unit: Box<Account<'info, Entity>>,
    pub player: Box<Account<'info, Entity>>,
}

#[derive(Accounts)]
pub struct AttackUnit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Kyogen
    pub instance_index: Box<Account<'info, InstanceIndex>>,
    #[account(
        // Anyone can initialize themselves
        seeds=[SEEDS_KYOGENSIGNER],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

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

    pub map: Box<Account<'info, Entity>>,
    #[account(mut)]
    pub attacker: Box<Account<'info, Entity>>,
    #[account(mut)]
    pub defender: Box<Account<'info, Entity>>,
    #[account(mut)]
    pub defending_tile: Box<Account<'info, Entity>>,
}

#[derive(Accounts)]
pub struct CloseEntity<'info>{
    #[account(mut)]
    pub receiver: Signer<'info>,
    pub system_program: Program<'info, System>,

    // Kyogen
    pub kyogen_config: Box<Account<'info, Config>>, // Need for component reference
    pub kyogen_index: Box<Account<'info, InstanceIndex>>, // Need for Tile entities

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

    #[account(mut)]
    pub entity: Box<Account<'info, Entity>>,
}

#[derive(Accounts)]
pub struct CloseKyogenIndex <'info> {
    #[account(mut)]
    pub receiver: Signer<'info>,
    pub system_program: Program<'info, System>,
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