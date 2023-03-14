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

declare_id!("CTQCiB97LrAjAtHy1eqGwqGiy2mjefBXR762nrDhWYTL");

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
        ctx.accounts.blueprint_acc.name = name;
        ctx.accounts.blueprint_acc.components = blueprint;
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

        // Instance the Registry
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
        ctx.accounts.instance_index.instance = instance;
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
    pub fn init_tile(ctx:Context<InitTile>, entity_id:u64, x:u8, y:u8, spawnable: bool, spawn_cost: u64, clan: Option<Clans>) -> Result<()> {
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
            clan,
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
            name: format!("{}:{}", ctx.accounts.payer.key().to_string(), name),
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
            solarite: 0,
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
            return err!(KyogenError::NoOccupantOnTile)
        }
            // Unit that's passed is the Tile Unit
        if unit_component.occupant_id.unwrap() != ctx.accounts.unit_entity.entity_id {
            return err!(KyogenError::WrongUnit)
        }
            // Player is the Owner of the Tile Unit
        let owner_c = ctx.accounts.unit_entity.components.get(&reference.owner).unwrap();
        let owner_component = ComponentOwner::try_from_slice(&owner_c.data.as_slice()).unwrap();
        if owner_component.owner.unwrap() != ctx.accounts.payer.key() {
            return err!(KyogenError::PlayerDoesntOwnUnit)
        }

        // Check that tile is Spawnable
        let spawn_c = ctx.accounts.tile_entity.components.get(&reference.spawn).unwrap();
        let mut spawn_component = ComponentSpawn::try_from_slice(spawn_c.data.as_slice()).unwrap();
        if !spawn_component.spawnable {
            return err!(KyogenError::TileIsNotSpawnable)
        }
        // Check that Spawn isn't already Player's Clans'
        let player_stats_c = ctx.accounts.player_entity.components.get(&reference.player_stats).unwrap();
        let player_stats_component = ComponentPlayerStats::try_from_slice(&player_stats_c.data.as_slice()).unwrap();
        if player_stats_component.clan != spawn_component.clan.unwrap() {
            return err!(KyogenError::TileAlreadyClaimed)
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
            return err!(KyogenError::TileIsNotEmpty)
        }
        // Check that Tile is spawnable and belongs to player Clan
        let tile_spawnable_component = ctx.accounts.tile.components.get(&reference.spawn).unwrap();
        let tile_spawn = ComponentSpawn::try_from_slice(&tile_spawnable_component.data.as_slice()).unwrap();
        if !tile_spawn.spawnable ||
            tile_spawn.clan.is_none() || 
            tile_spawn.clan.unwrap() != player_stats.clan    
        {
            return err!(KyogenError::TileIsNotSpawnable)
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
    pub fn move_unit(ctx:Context<MoveUnit>) -> Result<()> {
        // Don't need to check if the tiles / units/ etc belong to the registry instance because the registry will do those checks for us
        let reference = &ctx.accounts.config.components;

        // Check if game is paused
        if ctx.accounts.instance_index.play_phase != PlayPhase::Play {
            return err!(KyogenError::GamePaused)
        }

        // From tile has an occupant that is owned by player
        let from_occupant_c = ctx.accounts.from.components.get(&reference.occupant).unwrap();
        let mut from_occupant = ComponentOccupant::try_from_slice(&from_occupant_c.data.as_slice()).unwrap();
        if from_occupant.occupant_id.is_none() ||
            from_occupant.occupant_id.unwrap() != ctx.accounts.unit.entity_id {
                return err!(KyogenError::WrongTile)
        }
        let unit_owner_c = ctx.accounts.unit.components.get(&reference.owner).unwrap();
        let unit_owner = ComponentOwner::try_from_slice(&unit_owner_c.data.as_slice()).unwrap();
        if unit_owner.owner.unwrap() != ctx.accounts.payer.key() ||
            unit_owner.player.unwrap() != ctx.accounts.player.entity_id {
            return err!(KyogenError::WrongPlayer)
        }

        // To tile occupant is empty
        let to_occupant_c = ctx.accounts.to.components.get(&reference.occupant).unwrap();
        let mut to_occupant = ComponentOccupant::try_from_slice(&to_occupant_c.data.as_slice()).unwrap();
        if to_occupant.occupant_id.is_some() {
            return err!(KyogenError::TileOccupied)
        }

        // Unit is recovered from last used
        let clock = Clock::get().unwrap();
        let unit_last_used_c = ctx.accounts.unit.components.get(&reference.last_used).unwrap();
        let mut unit_last_used = ComponentLastUsed::try_from_slice(&unit_last_used_c.data.as_slice()).unwrap();
        if unit_last_used.last_used != 0 && (unit_last_used.last_used + unit_last_used.recovery) >= clock.slot {
            return err!(KyogenError::UnitRecovering)
        }

        // Distance between from and to must be < Unit's movement
        let from_location_c = ctx.accounts.from.components.get(&reference.location).unwrap();
        let from_location = ComponentLocation::try_from_slice(&from_location_c.data.as_slice()).unwrap();

        let to_location_c = ctx.accounts.to.components.get(&reference.location).unwrap();
        let to_location = ComponentLocation::try_from_slice(&to_location_c.data.as_slice()).unwrap();
        
        let distance:f64 = (((to_location.x as f64 - from_location.x as f64).powf(2_f64) + (to_location.y as f64 - from_location.y as f64).powf(2_f64)) as f64).sqrt();
        let unit_range_component = ctx.accounts.unit.components.get(&reference.range).unwrap();
        let unit_range = ComponentRange::try_from_slice(&unit_range_component.data.as_slice()).unwrap();
        if unit_range.movement < distance.floor() as u8 {
            return err!(KyogenError::TileOutOfRange)
        }

        let system_signer_seeds:&[&[u8]] = &[
            SEEDS_KYOGENSIGNER,
            &[*ctx.bumps.get("config").unwrap()]
        ];
        let signer_seeds = &[system_signer_seeds];
        
        // Modify Unit Last Used & Location (copy from To Tile)
        unit_last_used.last_used = clock.slot;
        let modify_unit_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.unit.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_unit_ctx, vec![
            (reference.last_used, unit_last_used.try_to_vec().unwrap()), // Last Used
            (reference.location, to_location.try_to_vec().unwrap())  // Location
        ])?;

        // Modify From Tile
        from_occupant.occupant_id = None;

        let modify_from_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.from.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_from_ctx, vec![
            (reference.occupant, from_occupant.try_to_vec().unwrap()), // Last Used
        ])?;


        // Modify To Tile
        to_occupant.occupant_id = Some(ctx.accounts.unit.entity_id);
        
        let modify_to_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.to.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_to_ctx, vec![
            (reference.occupant, to_occupant.try_to_vec().unwrap()), // Last Used
        ])?;

        emit!(UnitMoved{
            instance: ctx.accounts.registry_instance.instance,
            unit: ctx.accounts.unit.entity_id,
            from: ctx.accounts.from.entity_id,
            to: ctx.accounts.to.entity_id
        });

        Ok(())
    }

    // Attack Unit
    pub fn attack_unit(ctx: Context<AttackUnit>) -> Result<()> {
        // Don't need to check if the tiles / units/ etc belong to the registry instance because the registry will do those checks for us
        let reference = &ctx.accounts.config.components;
        let attacker = &ctx.accounts.attacker;
        let defender = &ctx.accounts.defender;
        let tile = &ctx.accounts.defending_tile;

        let system_signer_seeds:&[&[u8]] = &[
            SEEDS_KYOGENSIGNER,
            &[*ctx.bumps.get("config").unwrap()]
        ];
        let signer_seeds = &[system_signer_seeds];

        // Check if game is paused
        if ctx.accounts.instance_index.play_phase != PlayPhase::Play {
            return err!(KyogenError::GamePaused)
        }

        // Check attacker is owned by payer
        let attacker_owner_c = attacker.components.get(&reference.owner).unwrap();
        let attacker_owner = ComponentOwner::try_from_slice(&attacker_owner_c.data.as_slice()).unwrap();
        if attacker_owner.owner != Some(ctx.accounts.payer.key()) {
            return err!(KyogenError::PlayerDoesntOwnUnit)
        }

        // Check defender is NOT owned by payer
        let defender_owner_c = defender.components.get(&reference.owner).unwrap();
        let defender_owner = ComponentOwner::try_from_slice(&defender_owner_c.data.as_slice()).unwrap();
        if defender_owner.player == attacker_owner.player {
            return err!(KyogenError::AttackingSelfOwnedUnit)
        }

        // Check that attacker and defender are active
        let attacker_active_c = attacker.components.get(&reference.active).unwrap();
        let attacker_active = ComponentActive::try_from_slice(&attacker_active_c.data.as_slice()).unwrap();
        let defender_active_c = defender.components.get(&reference.active).unwrap();
        let mut defender_active = ComponentActive::try_from_slice(&defender_active_c.data.as_slice()).unwrap();       
        if attacker_active.active == false || defender_active.active == false{
            return err!(KyogenError::UnitNotActive)
        }

        // Defending Tile unit is Defender
        let tile_occupant_c = tile.components.get(&reference.occupant).unwrap();
        let mut tile_occupant = ComponentOccupant::try_from_slice(&tile_occupant_c.data.as_slice()).unwrap();
        if tile_occupant.occupant_id.unwrap() != defender.entity_id {
            return err!(KyogenError::WrongUnit)
        }

        // Defender is in range of attacker
        let attacker_location_c = attacker.components.get(&reference.location).unwrap();
        let attacker_location = ComponentLocation::try_from_slice(&attacker_location_c.data.as_slice()).unwrap();
        let defender_location_c = defender.components.get(&reference.location).unwrap();
        let defender_location = ComponentLocation::try_from_slice(&defender_location_c.data.as_slice()).unwrap();
        
        let distance:f64 = (((defender_location.x as f64 - attacker_location.x as f64).powf(2_f64) + (defender_location.y as f64 - attacker_location.y as f64 ).powf(2_f64)) as f64).sqrt();
        let attacker_range_c = attacker.components.get(&reference.range).unwrap();
        let attacker_range = ComponentRange::try_from_slice(&attacker_range_c.data.as_slice()).unwrap();
        if distance.floor() as u8 > attacker_range.attack_range {
            return err!(KyogenError::TileOutOfRange)
        }

        // Attacker last used is valid
        let clock = Clock::get().unwrap();
        let attacker_last_used_c = attacker.components.get(&reference.last_used).unwrap();
        let mut attacker_last_used = ComponentLastUsed::try_from_slice(&attacker_last_used_c.data.as_slice()).unwrap();
        if attacker_last_used.last_used != 0 && (attacker_last_used.last_used + attacker_last_used.recovery) >= clock.slot {
            return err!(KyogenError::UnitRecovering)
        }

        attacker_last_used.last_used = clock.slot; 

        // Calculate damage with Attacker Damage component against Defender Health Component
        let attacker_damage_c = attacker.components.get(&reference.damage).unwrap();
        let attacker_damage = ComponentDamage::try_from_slice(&attacker_damage_c.data.as_slice()).unwrap();

        let defender_health_c = defender.components.get(&reference.health);
        let mut defender_health = ComponentHealth::try_from_slice(&defender_health_c.unwrap().data.as_slice()).unwrap();

        let mut dmg = get_random_u64(attacker_damage.max_damage); 

        let defender_troop_class_c = defender.components.get(&reference.troop_class).unwrap();
        let defender_troop_class = ComponentTroopClass::try_from_slice(&defender_troop_class_c.data.as_slice()).unwrap();
        match defender_troop_class.class {
            TroopClass::Samurai => dmg += attacker_damage.bonus_samurai as u64,
            TroopClass::Shinobi => dmg += attacker_damage.bonus_shinobi as u64,
            TroopClass::Sohei => dmg += attacker_damage.bonus_sohei as u64
        }

        if dmg < attacker_damage.min_damage {
            dmg = attacker_damage.min_damage;
        }
        // Modify defender health and active
        if dmg >= defender_health.health {
            // modify the defending tile occupant and set defender to not active
            defender_health.health = 0;
            defender_active.active = false;
            tile_occupant.occupant_id = None;

            let modify_tile_ctx = CpiContext::new_with_signer(
                ctx.accounts.registry_program.to_account_info(),            
                registry::cpi::accounts::ModifyComponent {
                    entity: ctx.accounts.defending_tile.to_account_info(),
                    registry_config: ctx.accounts.registry_config.to_account_info(),
                    action_bundle: ctx.accounts.config.to_account_info(),
                    action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                    core_ds: ctx.accounts.coreds.to_account_info(),                
                }, 
                signer_seeds
            );
            registry::cpi::req_modify_component(modify_tile_ctx, vec![
                (reference.occupant, tile_occupant.try_to_vec().unwrap()), // Last Used
            ])?;

        } else {
            // subtract defender health
            defender_health.health -= dmg;
        }
        // Modify defender
        let modify_defender_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.defender.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_defender_ctx, vec![
            (reference.health, defender_health.try_to_vec().unwrap()), 
            (reference.active, defender_active.try_to_vec().unwrap()), 
        ])?;

        // Modify attacker last used
        let modify_attacker_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.attacker.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.kyogen_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_attacker_ctx, vec![
            (reference.last_used, attacker_last_used.try_to_vec().unwrap()), // Last Used
        ])?;

        // emit unit attacked
        emit!(UnitAttacked{
            instance: ctx.accounts.registry_instance.instance,
            attacker: ctx.accounts.attacker.entity_id,
            defender: ctx.accounts.defender.entity_id,
            tile: ctx.accounts.defending_tile.entity_id,
        });

        Ok(())
    }

    // TODO: Widraw Money from Instance Index
    // TODO: Claim Victory
    // TODO: Reclaim Sol from a Game
        // Close Map, Tile, Player
}

pub fn get_random_u64(max: u64) -> u64 {
    let clock = Clock::get().unwrap();
    let slice = &hash(&clock.slot.to_be_bytes()).to_bytes()[0..8];
    let num: u64 = u64::from_be_bytes(slice.try_into().unwrap());
    let target = num/(u64::MAX/max);
    return target;
}