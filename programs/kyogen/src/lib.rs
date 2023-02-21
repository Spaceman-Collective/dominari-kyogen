use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::*;
use std::collections::BTreeMap;
use anchor_spl::token::{Transfer, transfer};

pub mod account;
pub mod context;
pub mod constant;
pub mod error;
pub mod event;
pub mod component;
pub mod state;

use account::*;
use context::*;
use constant::*;
use error::*;
use event::*;
use component::*;
use state::*;

use core_ds::account::MaxSize;
use core_ds::state::SerializedComponent;

declare_id!("3YdayPtujByJ1g1DWEUh7vpg78gZL49FWyD5rDGyof9T");

#[program]
pub mod kyogen {
    use super::*;

    // Initialize: Set authority & relevant component keys
    pub fn initialize(ctx: Context<Initialize>, component_keys: KyogenComponentKeys) -> Result<()> {
        ctx.accounts.config.authority = ctx.accounts.payer.key();
        ctx.accounts.config.components = component_keys;
        Ok(())
    }
    
    /**
     * Registers a new pack with the given name and pubkeys of blueprints
     * @param name 
     */
    pub fn register_pack(ctx: Context<RegisterPack>, name: String, blueprints: Vec<Pubkey>) -> Result<()> {
        ctx.accounts.pack.name = name;
        ctx.accounts.pack.blueprints = blueprints;
        Ok(())
    }

    /** Adds a new blueprint
     * @param name
     * @param blueprint BTreeMap of Pubkey to Serialized Component that gets auto loaded onto the new entity
     */
    pub fn register_blueprint(ctx: Context<RegisterBlueprint>, name: String, blueprint: BTreeMap<Pubkey, SerializedComponent>) -> Result<()> {
        ctx.accounts.blueprint.name = name;
        ctx.accounts.blueprint.components = blueprint;
        Ok(())
    }

    /**
     * Creates a registry instance with Core Ds Program.
     */
    pub fn create_game_instance(ctx: Context<CreateGameInstance>, instance: u64, game_config: GameConfig) -> Result<()> {
        let config_seeds:&[&[u8]] = &[
            SEEDS_KYOGENSIGNER,
            &[*ctx.bumps.get("config").unwrap()]
        ];
        let signer_seeds = &[config_seeds];

        // Instance the World
        let instance_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),
            registry::cpi::accounts::InstanceRegistry {
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                registry_instance: ctx.accounts.registry_instance.to_account_info(),
                registry_index: ctx.accounts.registry_index.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),
                ab_signer: ctx.accounts.config.to_account_info(),
            },
            signer_seeds
        );

        registry::cpi::instance_registry(instance_ctx, instance)?;
        // Set up Instance Index
        ctx.accounts.instance_index.config = game_config; 
        ctx.accounts.instance_index.authority = ctx.accounts.payer.key();
        Ok(()) 
    }

    /**
     * Only admin is allowed to change the game states for Kyogen Clash games.
     */
    pub fn change_game_state(ctx:Context<ChangeGameState>, new_game_state:PlayPhase) -> Result<()> {
        ctx.accounts.instance_index.play_phase = new_game_state.clone();
        emit!(GameStateChanged{
            instance: ctx.accounts.registry_instance.instance,
            new_state: new_game_state
        });
        Ok(())
    }

    // Init Map
    pub fn init_map(ctx: Context<InitMap>, entity_id:u64, max_x:u8, max_y:u8) -> Result<()> {
        let reference = &ctx.accounts.config.components;
        let config_seeds:&[&[u8]] = &[
            SEEDS_KYOGENSIGNER,
            &[*ctx.bumps.get("config").unwrap()]
        ];
        let signer_seeds = &[config_seeds];

        let init_entity_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),
            registry::cpi::accounts::InitEntity{
                entity: ctx.accounts.map_entity.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                registry_instance: ctx.accounts.registry_instance.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),
            },
            signer_seeds
        );

        let mut components: BTreeMap<Pubkey, SerializedComponent> = BTreeMap::new();
        // Map has Metadata and MapMeta Components
        let metadata_component = ComponentMetadata {
            name: format!("Map ({:#})", ctx.accounts.registry_instance.instance),
            registry_instance: ctx.accounts.registry_instance.key(),
        }.try_to_vec().unwrap();
        components.insert(reference.metadata.key(), SerializedComponent { 
            max_size: ComponentMetadata::get_max_size(), 
            data:  metadata_component
        });

        let mapmeta_component = ComponentMapMeta {
            max_x,
            max_y,
        }.try_to_vec().unwrap();
        components.insert(reference.mapmeta.key(), SerializedComponent { 
            max_size: ComponentMapMeta::get_max_size(), 
            data: mapmeta_component 
        });

        // Mint Map Entity
        registry::cpi::init_entity(init_entity_ctx, entity_id, components)?;
        ctx.accounts.instance_index.map = entity_id; //ctx.accounts.map_entity.key();
        Ok(())
    }

    // Init Tile
    pub fn init_tile(ctx:Context<InitTile>, entity_id:u64, x:u8, y:u8, spawnable: bool, spawn_cost: u64) -> Result<()> {
        // Tile can only be instanced by Admin
        // So we can trust in the input for x,y isn't duplicated
        let reference = &ctx.accounts.config.components;

        // Tile has Metadata, Location, Feature, Occupant, Owner and Cost components
        // Tile also has a Spawnable Component
        let mut components: BTreeMap<Pubkey, SerializedComponent> = BTreeMap::new();
        let metadata = ComponentMetadata {
            name: format!("Tile ({x}, {y})"),
            registry_instance: ctx.accounts.registry_instance.key(),
        }.try_to_vec().unwrap();
        components.insert(reference.metadata.key(), SerializedComponent { 
            max_size: ComponentMetadata::get_max_size(),
            data: metadata
        });

        let location = ComponentLocation {
            x,
            y,
        }.try_to_vec().unwrap();
        components.insert(reference.location.key(), SerializedComponent { 
            max_size: ComponentLocation::get_max_size(),
            data: location
        });

        let occupant = ComponentOccupant {
            occupant_id: None,
        }.try_to_vec().unwrap();
        components.insert(reference.occupant.key(), SerializedComponent { 
            max_size: ComponentOccupant::get_max_size(),
            data: occupant
        });

        let spawnable = ComponentSpawn {
            spawnable,
            clan: None,
            cost: spawn_cost
        }.try_to_vec().unwrap();
        components.insert(reference.spawn.key(), SerializedComponent { 
            max_size: ComponentSpawn::get_max_size(),
            data: spawnable
        });

        let config_seeds:&[&[u8]] = &[
            SEEDS_KYOGENSIGNER,
            &[*ctx.bumps.get("config").unwrap()]
        ];
        let signer_seeds = &[config_seeds];

        let init_entity_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),
            registry::cpi::accounts::InitEntity{
                entity: ctx.accounts.tile_entity.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                registry_instance: ctx.accounts.registry_instance.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),
            },
            signer_seeds
        );

        registry::cpi::init_entity(init_entity_ctx, entity_id, components)?;
        ctx.accounts.instance_index.tiles.push(entity_id);
        Ok(())
    }

    // Init Player
    pub fn init_player(ctx:Context<InitPlayer>, entity_id:u64, name:String, clan: Clans) -> Result<()> {
        let reference = &ctx.accounts.config.components;
        // Optional: Fail if too many players already in the instance
        if ctx.accounts.instance_index.config.max_players == ctx.accounts.instance_index.players.len() as u16 {
            return err!(KyogenError::PlayerCountExceeded)
        }

        if name.len() > STRING_MAX_SIZE as usize {
            return err!(KyogenError::StringTooLong)
        }

        // Create Player Entity
        // Player has: Metadata and Player Stats
        let mut components: BTreeMap<Pubkey, SerializedComponent> = BTreeMap::new();
        // Feature has Metadata, Location, Owner, Active, and ..Blueprint Components
        let metadata_component = ComponentMetadata {
            name: ctx.accounts.payer.key().to_string(),
            registry_instance: ctx.accounts.registry_instance.key(),
        }.try_to_vec().unwrap();
        components.insert(reference.metadata.key(), SerializedComponent { 
            max_size: ComponentMetadata::get_max_size(), 
            data:  metadata_component
        });

        let pack = &ctx.accounts.pack;
        let starting_cards;
        match clan {
            Clans::Ancients => {
                if pack.name != STARTING_CARDS_ANCIENTS_NAME {
                    return err!(KyogenError::WrongPack)
                }
                starting_cards = pack.blueprints.clone();
            },
            Clans::Wildings => {
                if pack.name != STARTING_CARDS_WILDINGS_NAME {
                    return err!(KyogenError::WrongPack)
                }
                starting_cards = pack.blueprints.clone();
            },
            Clans::Creepers => {
                if pack.name != STARTING_CARDS_CREEPERS_NAME {
                    return err!(KyogenError::WrongPack)
                }
                starting_cards = pack.blueprints.clone();
            },
            Clans::Synths => {
                if pack.name != STARTING_CARDS_SYNTHS_NAME {
                    return err!(KyogenError::WrongPack)
                }
                starting_cards = pack.blueprints.clone();
            },
        }

        let player_stats_component = ComponentPlayerStats {
            name,
            key: ctx.accounts.payer.key(),
            score: 0,
            kills: 0,
            cards: starting_cards,
            clan: clan.clone(),
        }.try_to_vec().unwrap();
        components.insert(reference.player_stats.key(), SerializedComponent { 
            max_size: ComponentPlayerStats::get_max_size(), 
            data:  player_stats_component
        });

        let config_seeds:&[&[u8]] = &[
            SEEDS_KYOGENSIGNER,
            &[*ctx.bumps.get("config").unwrap()]
        ];
        let signer_seeds = &[config_seeds];

        let init_entity_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),
            registry::cpi::accounts::InitEntity{
                entity: ctx.accounts.player_entity.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                registry_instance: ctx.accounts.registry_instance.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),
            },
            signer_seeds
        );

        registry::cpi::init_entity(init_entity_ctx, entity_id, components)?;
        
        // Add player entity to instance index
        ctx.accounts.instance_index.players.push(entity_id);

        emit!(NewPlayer { 
            instance: ctx.accounts.registry_instance.instance, 
            player_id: entity_id, 
            authority: ctx.accounts.payer.key(), 
            clan: clan
        });
        Ok(())
    }
    
    // Change spawnable tile's clan affiliation
    pub fn claim_spawn(ctx:Context<ClaimSpawn>) -> Result<()> {
        let reference = &ctx.accounts.config.components;

        // Check the game isnt' paused
        if ctx.accounts.instance_index.play_phase != PlayPhase::Play {
            return err!(KyogenError::GamePaused)
        }

        // Check that the 
            // Tile has a Unit
        let unit_c = ctx.accounts.tile_entity.components.get(&reference.occupant).unwrap();
        let unit_component = ComponentOccupant::try_from_slice(unit_c.data.as_slice()).unwrap();
        if unit_component.occupant_id.is_none(){
            return err!(KyogenError::WrongTile)
        }
            // Unit that's passed is the Tile Unit
        if unit_component.occupant_id.unwrap() != ctx.accounts.unit_entity.entity_id {
            return err!(KyogenError::WrongUnit)
        }
            // Player is the Owner of the Tile Unit
        let owner_c = ctx.accounts.unit_entity.components.get(&reference.owner).unwrap();
        let owner_component = ComponentOwner::try_from_slice(&owner_c.data.as_slice()).unwrap();
        if owner_component.owner.unwrap() != ctx.accounts.payer.key() {
            return err!(KyogenError::WrongUnit)
        }

        // Check that tile is Spawnable
        let spawn_c = ctx.accounts.tile_entity.components.get(&reference.spawn).unwrap();
        let mut spawn_component = ComponentSpawn::try_from_slice(spawn_c.data.as_slice()).unwrap();
        if !spawn_component.spawnable {
            return err!(KyogenError::WrongTile)
        }
        // Check that Spawn isn't already Player's Clans'
        let player_stats_c = ctx.accounts.player_entity.components.get(&reference.player_stats).unwrap();
        let player_stats_component = ComponentPlayerStats::try_from_slice(&player_stats_c.data.as_slice()).unwrap();
        if player_stats_component.clan != spawn_component.clan.unwrap() {
            return err!(KyogenError::WrongTile)
        } 
        // Charge the Player GAME TOKEN to claim the spawn
        let transfer_accounts = Transfer{
            from: ctx.accounts.from_ata.to_account_info(),
            to: ctx.accounts.to_ata.to_account_info(),
            authority: ctx.accounts.payer.to_account_info()
        };

        transfer(CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            transfer_accounts,
        ), spawn_component.cost)?;

        
        // Change the cost of the Spawn by Spawn Multiplier
        spawn_component.cost = (spawn_component.cost as f64 * ctx.accounts.instance_index.config.spawn_claim_multiplier).floor() as u64;
        match player_stats_component.clan {
            Clans::Ancients => spawn_component.clan = Some(Clans::Ancients),
            Clans::Wildings => spawn_component.clan = Some(Clans::Wildings),
            Clans::Creepers => spawn_component.clan = Some(Clans::Creepers),
            Clans::Synths => spawn_component.clan = Some(Clans::Synths)
        }
        // Save changes to Tile
        let config_seeds:&[&[u8]] = &[
            SEEDS_KYOGENSIGNER,
            &[*ctx.bumps.get("config").unwrap()]
        ];
        let signer_seeds = &[config_seeds];
        let modify_tile_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.tile_entity.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_tile_ctx, vec![(reference.spawn, spawn_component.try_to_vec().unwrap())])?;
       
        emit!(SpawnClaimed { 
            instance: ctx.accounts.registry_instance.instance, 
            clan: player_stats_component.clan, 
            player: ctx.accounts.player_entity.entity_id
        });
        
        Ok(())
    }

    // Spawn Unit
    pub fn spawn_unit(ctx:Context<SpawnUnit>, unit_id:u64) -> Result<()> {
        let reference = &ctx.accounts.config.components;

        // Check if game is paused
        // Check the game isnt' paused
        if ctx.accounts.instance_index.play_phase != PlayPhase::Play {
            return err!(KyogenError::GamePaused)
        }

        // Check player belongs to payer
        let player_stats_component = ctx.accounts.player.components.get(&reference.player_stats).unwrap();
        let mut player_stats = ComponentPlayerStats::try_from_slice(&player_stats_component.data.as_slice()).unwrap();
        if player_stats.key.key() != ctx.accounts.payer.key() {
            return err!(KyogenError::WrongPlayer)
        }

        // Check the tile is empty
        let tile_occupant_component = ctx.accounts.tile.components.get(&reference.occupant).unwrap();
        let mut tile_occupant = ComponentOccupant::try_from_slice(&tile_occupant_component.data.as_slice()).unwrap();
        if tile_occupant.occupant_id.is_some() {
            return err!(KyogenError::WrongTile)
        }
        // Check that Tile is spawnable and belongs to player Clan
        let tile_spawnable_component = ctx.accounts.tile.components.get(&reference.spawn).unwrap();
        let tile_spawn = ComponentSpawn::try_from_slice(&tile_spawnable_component.data.as_slice()).unwrap();
        if !tile_spawn.spawnable ||
            tile_spawn.clan.is_none() || 
            tile_spawn.clan.unwrap() != player_stats.clan    
        {
            return err!(KyogenError::WrongTile)
        }

        // Check that blueprint is in player hand
            // Unwrap is fine here because if the Blueprint is not in player hand we just fail
        let card_idx = player_stats.cards.iter().position(|&card| card.key() == ctx.accounts.unit_blueprint.key()).unwrap();
        //Modify player hand to remove blueprint
        player_stats.cards.swap_remove(card_idx);

        // Create unit entity
        // Unit has Metadata, Owner, Location, Active + Blueprint components
        let mut components: BTreeMap<Pubkey, SerializedComponent> = BTreeMap::new();
        // Add Metadata, Owner, Location, Active + Blueprint components
        let metadata_component = ComponentMetadata {
            name: ctx.accounts.unit_blueprint.name.clone(),
            registry_instance: ctx.accounts.registry_instance.key()
        }.try_to_vec().unwrap();
        components.insert(reference.metadata.key(), SerializedComponent {
            max_size: ComponentMetadata::get_max_size(),
            data: metadata_component
        });
        let owner_component = ComponentOwner {  
            owner: Some(ctx.accounts.payer.key()),
            player: Some(ctx.accounts.player.entity_id)
        }.try_to_vec().unwrap();
        components.insert(reference.owner.key(), SerializedComponent {
            max_size: ComponentOwner::get_max_size(),
            data: owner_component
        });
        let active_component = ComponentActive {
            active: true
        }.try_to_vec().unwrap();
        components.insert(reference.active.key(), SerializedComponent{
            max_size: ComponentActive::get_max_size(),
            data: active_component
        });

        // Clone the Tile's location component to the Unit
        components.insert(
            reference.location.key(),
            ctx.accounts.tile.components.get(&reference.location).unwrap().clone()
        );
        components.extend(ctx.accounts.unit_blueprint.components.clone());

        // Add the new unit entity to instance index      
        let system_signer_seeds:&[&[u8]] = &[
            SEEDS_KYOGENSIGNER,
            &[*ctx.bumps.get("config").unwrap()]
        ];
        let signer_seeds = &[system_signer_seeds];

        let mint_entity_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),
            registry::cpi::accounts::InitEntity{
                entity: ctx.accounts.unit.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                registry_instance: ctx.accounts.registry_instance.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),
            },
            signer_seeds
        );
        registry::cpi::init_entity(mint_entity_ctx, unit_id, components)?;
        ctx.accounts.instance_index.units.push(unit_id);

        // Modify tile for occupant component to point to unit
        tile_occupant.occupant_id = Some(unit_id);
        let modify_tile_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.tile.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_tile_ctx, vec![(reference.occupant, tile_occupant.try_to_vec().unwrap())])?;
        // Update player stats to no longer have the card
        let modify_player_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.player.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_player_ctx, vec![(reference.player_stats, player_stats.try_to_vec().unwrap())])?;
        // Emit UnitSpawn event
        emit!(UnitSpawned{
            instance: ctx.accounts.registry_instance.instance,
            tile: ctx.accounts.tile.entity_id,
            player: ctx.accounts.player.entity_id,
            unit: unit_id
        });
        Ok(())
    }

    // Move Unit
    // Attack Unit
    // Widraw Money from Instance Index

    // Reclaim Sol from a Game
        // Close Map, Tile, Player
}

pub fn get_random_u64(max: u64) -> u64 {
    let clock = Clock::get().unwrap();
    let slice = &hash(&clock.slot.to_be_bytes()).to_bytes()[0..8];
    let num: u64 = u64::from_be_bytes(slice.try_into().unwrap());
    let target = num/(u64::MAX/max);
    return target;
}

/* MOVE TO CARD LAYER */
// Use Card
/* MOVE TO CARD LAYER */