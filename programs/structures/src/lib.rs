use anchor_lang::prelude::*;

use kyogen::component::*;
use kyogen::constant::Clans;
use core_ds::state::SerializedComponent;
use std::collections::BTreeMap;
use core_ds::account::MaxSize;
use anchor_spl::token::{Transfer, transfer};
use anchor_lang::solana_program::hash::*;

pub mod context;
pub mod account;
pub mod state;
pub mod constant;
pub mod component;
pub mod error;
pub mod event;

use context::*;
//use account::*;
use state::*;
use constant::*;
use component::*;
use error::*;
use event::*;

declare_id!("4Bo4cgr4RhGpXJsQUV4KENCf3HJwPveFsPELJGGN9GkR");

#[program]
pub mod structures {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, component_keys: StructuresComponentKeys) -> Result<()> {
        ctx.accounts.config.authority = ctx.accounts.payer.key();
        ctx.accounts.config.components = component_keys;
        Ok(())
    }

    // Init Index & Register with Registry
    pub fn init_index(ctx:Context<InitStructureIndex>, instance:u64) -> Result<()> {
        // CTX Creates the Structure Index
        // Append the Registry Index to include this AB for the given instance
        
        // Instance the World
        let append_ctx = CpiContext::new(
            ctx.accounts.registry_program.to_account_info(),
            registry::cpi::accounts::AppendRegistryIndex {
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                registry_index: ctx.accounts.registry_index.to_account_info(),
                action_bundle_registration: ctx.accounts.ab_registration.to_account_info(),
            }
        );

        registry::cpi::append_registry_index(append_ctx)?;
        ctx.accounts.structures_index.instance = instance;
        Ok(())
    }

    // Init Structure
    pub fn init_structure(ctx: Context<InitStructure>, 
        entity_id: u64, 
        location: (u8, u8),
    ) -> Result<()> {
        // Define entity with components
        let kyogen_ref = &ctx.accounts.kyogen_config.components;

        let config_seeds:&[&[u8]] = &[
            SEEDS_STRUCTURESSIGNER,
            &[*ctx.bumps.get("config").unwrap()]
        ];
        let signer_seeds = &[config_seeds];

        // Structure cannot be init'd on a Tile that's spawnable
        let tile = &ctx.accounts.tile;
        let tile_location_sc = tile.components.get(&kyogen_ref.location).unwrap();
        let tile_location = ComponentLocation::try_from_slice(&tile_location_sc.data.as_slice()).unwrap();
        let tile_spawn_sc = tile.components.get(&kyogen_ref.spawn).unwrap();
        let tile_spawn = ComponentSpawn::try_from_slice(&tile_spawn_sc.data.as_slice()).unwrap();

        // Check tile location == structure location
        if tile_location.x != location.0 || tile_location.y != location.1 {
            return err!(StructureError::LocationMismatch)
        }

        // Check that the tile is NOT spawnable
        if tile_spawn.spawnable {
            return err!(StructureError::StructureCannotBeInitializedOnSpawn)
        }

        let mut components: BTreeMap<Pubkey, SerializedComponent> = BTreeMap::new();
        // Structures have "metadata", "location", "image", "structure", "last used", "active"
        // Only Metadata + Location are initialized here, everything else is pulled from a blueprint

        let metadata_component = ComponentMetadata {
            name: ctx.accounts.structure_blueprint.name.clone(),
            registry_instance: ctx.accounts.registry_instance.key(),
        }.try_to_vec().unwrap();
        components.insert(kyogen_ref.metadata, SerializedComponent { 
            max_size: ComponentMetadata::get_max_size(),
            data:  metadata_component
        });

        let location_component = ComponentLocation {
            x: location.0,
            y: location.1
        }.try_to_vec().unwrap();
        components.insert(kyogen_ref.location, SerializedComponent { 
            max_size: ComponentLocation::get_max_size(), 
            data: location_component 
        });

        components.extend(ctx.accounts.structure_blueprint.components.clone());

        // CPI into Registry to Create Entity
        let init_entity_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(), 
            registry::cpi::accounts::InitEntity {
                entity: ctx.accounts.structure.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                registry_instance: ctx.accounts.registry_instance.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.config.to_account_info(),
                action_bundle_registration: ctx.accounts.structure_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),
            }, 
            signer_seeds
        );
        registry::cpi::init_entity(init_entity_ctx, entity_id, components)?;

        // Add to Structure Index
        // Figure out what structure type it is by looking up it's Structure Type component
        let structure_ref = &ctx.accounts.config.components;
        let blueprint_components = &ctx.accounts.structure_blueprint.components;

        let structure_type_c = blueprint_components.get(&structure_ref.structure).unwrap();
        let structure_type = ComponentStructure::try_from_slice(structure_type_c.data.as_slice()).unwrap();

        if matches!(structure_type.structure, StructureType::Portal { .. }) {
            ctx.accounts.structure_index.portal.push(entity_id);
        } else if matches!(structure_type.structure, StructureType::Healer { .. }) {
            ctx.accounts.structure_index.healer.push(entity_id);
        } else if matches!(structure_type.structure, StructureType::Lootable { .. }) {
            ctx.accounts.structure_index.lootable.push(entity_id);
        } else if matches!(structure_type.structure, StructureType::Meteor { .. }) {
            ctx.accounts.structure_index.meteor.push(entity_id);
        }

        Ok(())
    }

    // Use Meteor    
        // Transfer Solarite from SI to User
    pub fn use_meteor(ctx:Context<UseMeteor>) -> Result<()> {
        // Check the game isnt' paused
        if ctx.accounts.kyogen_index.play_phase != kyogen::account::PlayPhase::Play {
            return err!(StructureError::GamePaused)
        }

        let kyogen_ref = &ctx.accounts.kyogen_config.components;
        let structures_ref = &ctx.accounts.structures_config.components;
        
        let tile = &ctx.accounts.tile;
        let unit = &ctx.accounts.unit;
        let meteor = &ctx.accounts.meteor;
        let player = &ctx.accounts.player;

        // Make sure there is a unit on tile & it's the same id as unit.id
        let occupant_sc = tile.components.get(&kyogen_ref.occupant).unwrap();
        let occupant = ComponentOccupant::try_from_slice(&occupant_sc.data.as_slice()).unwrap();
        if occupant.occupant_id.is_none() || occupant.occupant_id.unwrap() != unit.entity_id {
            return err!(StructureError::InvalidUnit)
        }

        // Make sure tile.location == structure.location
        let location_sc = tile.components.get(&kyogen_ref.location).unwrap();
        let location = ComponentLocation::try_from_slice(&location_sc.data.as_slice()).unwrap();
        let meteor_loc_sc = meteor.components.get(&kyogen_ref.location).unwrap();
        let meteor_loc = ComponentLocation::try_from_slice(&meteor_loc_sc.data.as_slice()).unwrap();
        if location.x != meteor_loc.x || location.y != meteor_loc.y {
            return err!(StructureError::InvalidLocation)
        }

        // Make sure unit.owner == payer
        let owner_sc = unit.components.get(&kyogen_ref.owner).unwrap();
        let owner = ComponentOwner::try_from_slice(&owner_sc.data.as_slice()).unwrap();
        if owner.owner.unwrap().key() != ctx.accounts.payer.key() {
            return err!(StructureError::InvalidOwner)
        }
        
        // Make sure structure.last_used isn't violated
        let last_used_sc = meteor.components.get(&kyogen_ref.last_used).unwrap();
        let mut last_used = ComponentLastUsed::try_from_slice(&last_used_sc.data.as_slice()).unwrap();
        let clock = Clock::get().unwrap();
        if last_used.last_used != 0 && (last_used.last_used + last_used.recovery) >= clock.slot {
            return err!(StructureError::StructureInCooldown)
        }

        // Set Last Used to current slot
        let system_signer_seeds:&[&[u8]] = &[
            SEEDS_STRUCTURESSIGNER,
            &[*ctx.bumps.get("structures_config").unwrap()]
        ];
        let signer_seeds = &[system_signer_seeds];

        last_used.last_used = clock.slot;
        let modify_structure_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.meteor.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.structures_config.to_account_info(),
                action_bundle_registration: ctx.accounts.structures_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_structure_ctx, vec![
            (kyogen_ref.last_used, last_used.try_to_vec().unwrap()), // Last Used
        ])?;

        // Grant solarite to user
        let structure_sc = meteor.components.get(&structures_ref.structure).unwrap();
        let structure_info = ComponentStructure::try_from_slice(&structure_sc.data.as_slice()).unwrap();

        match structure_info.structure {
            StructureType::Meteor { solarite_per_use } => {
                let transfer_accounts = Transfer{
                    from: ctx.accounts.structures_ata.to_account_info(),
                    to: ctx.accounts.user_ata.to_account_info(),
                    authority: ctx.accounts.structures_index.to_account_info()
                };

                let registry_key = ctx.accounts.registry_instance.key();
                let structure_index_signer_seeds:&[&[u8]] = &[
                    SEEDS_PREFIXINDEX,
                    registry_key.as_ref(),
                    &[*ctx.bumps.get("structures_index").unwrap()]
                ];
        
                transfer(CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(), 
                    transfer_accounts,
                    &[structure_index_signer_seeds]
                ), solarite_per_use)?;

                // Grant Score to User
                let player_stats_sc = player.components.get(&kyogen_ref.player_stats).unwrap();
                let mut player_stats = ComponentPlayerStats::try_from_slice(&player_stats_sc.data.as_slice()).unwrap();
                player_stats.score += solarite_per_use;

                // Modify Player Stats 
                let modify_player_ctx = CpiContext::new_with_signer(
                    ctx.accounts.registry_program.to_account_info(),            
                    registry::cpi::accounts::ModifyComponent {
                        entity: ctx.accounts.player.to_account_info(),
                        registry_config: ctx.accounts.registry_config.to_account_info(),
                        action_bundle: ctx.accounts.structures_config.to_account_info(),
                        action_bundle_registration: ctx.accounts.structures_registration.to_account_info(),
                        core_ds: ctx.accounts.coreds.to_account_info(),                
                    }, 
                    signer_seeds
                );
                registry::cpi::req_modify_component(modify_player_ctx, vec![
                    (kyogen_ref.player_stats, player_stats.try_to_vec().unwrap()), // Last Used
                ])?;
        
                // Set high score if needed
                let current_high_score = &ctx.accounts.structures_index.high_score; 
                if player_stats.score > current_high_score.1 {
                    ctx.accounts.structures_index.high_score = (player.entity_id, player_stats.score);
                }

            },
            _ => {
                return err!(StructureError::WrongStructure)
            }
        }        
        
        emit!(MeteorMined{
            tile: ctx.accounts.tile.entity_id,
            meteor: ctx.accounts.meteor.entity_id,
            player: occupant.occupant_id.unwrap()
        });

        Ok(())
    }

     
    // Use Portal
    pub fn use_portal(ctx:Context<UsePortal>) -> Result<()> {
        // Check the game isnt' paused
        if ctx.accounts.kyogen_index.play_phase != kyogen::account::PlayPhase::Play {
            return err!(StructureError::GamePaused)
        }
        
        let kyogen_ref = &ctx.accounts.kyogen_config.components;
        let structures_ref = &ctx.accounts.structures_config.components;

        let from = &ctx.accounts.from;
        let from_portal = &ctx.accounts.from_portal;
        let to = &ctx.accounts.to;
        let to_portal = &ctx.accounts.to_portal;
        let unit = &ctx.accounts.unit;

        // Make sure from.occupant is unit.id
        let from_occupant_sc = from.components.get(&kyogen_ref.occupant).unwrap();
        let mut from_occupant = ComponentOccupant::try_from_slice(&from_occupant_sc.data.as_slice()).unwrap();
        if from_occupant.occupant_id.is_none() || from_occupant.occupant_id.unwrap() != unit.entity_id {
            return err!(StructureError::InvalidUnit)
        }

        // Make sure unit.owner == payer
        let unit_owner_sc = unit.components.get(&kyogen_ref.owner).unwrap();
        let unit_owner = ComponentOwner::try_from_slice(&unit_owner_sc.data.as_slice()).unwrap();
        if unit_owner.owner.unwrap().key() != ctx.accounts.payer.key() {
            return err!(StructureError::InvalidUnit)
        }

        // Make sure to.occupant is none
        let to_occupant_sc = to.components.get(&kyogen_ref.occupant).unwrap();
        let mut to_occupant = ComponentOccupant::try_from_slice(&to_occupant_sc.data.as_slice()).unwrap();
        if to_occupant.occupant_id.is_some() {
            return err!(StructureError::TileOccupied)
        }

        // Make sure from_portal.last_used isn't violated
        let clock = Clock::get().unwrap();
        let from_last_used_c = from_portal.components.get(&kyogen_ref.last_used).unwrap();
        let mut from_last_used = ComponentLastUsed::try_from_slice(&from_last_used_c.data.as_slice()).unwrap();
        if from_last_used.last_used != 0 && (from_last_used.last_used + from_last_used.recovery) >= clock.slot {
            return err!(StructureError::StructureInCooldown)
        }
        // Make sure unit.last_used isn't violated
        let unit_last_used_c = unit.components.get(&kyogen_ref.last_used).unwrap();
        let mut unit_last_used = ComponentLastUsed::try_from_slice(&unit_last_used_c.data.as_slice()).unwrap();
        if unit_last_used.last_used != 0 && (unit_last_used.last_used + unit_last_used.recovery) >= clock.slot {
            return err!(StructureError::UnitCooldown)
        }

        // Check from.location == from_portal.location
        let from_location_sc = from.components.get(&kyogen_ref.location).unwrap();
        let from_location = ComponentLocation::try_from_slice(&from_location_sc.data.as_slice()).unwrap();
        let from_portal_location_sc = from_portal.components.get(&kyogen_ref.location).unwrap();
        let from_portal_location = ComponentLocation::try_from_slice(&from_portal_location_sc.data.as_slice()).unwrap();
        if from_location.x != from_portal_location.x || 
           from_location.y != from_portal_location.y {
            return err!(StructureError::WrongStructure)
        }

        // Check to.location == to_portal.location
        let to_location_sc = to.components.get(&kyogen_ref.location).unwrap();
        let to_location = ComponentLocation::try_from_slice(&to_location_sc.data.as_slice()).unwrap();
        let to_portal_location_sc = to_portal.components.get(&kyogen_ref.location).unwrap();
        let to_portal_location = ComponentLocation::try_from_slice(&to_portal_location_sc.data.as_slice()).unwrap();
        if to_location.x != to_portal_location.x || 
           to_location.y != to_portal_location.y {
            return err!(StructureError::WrongStructure)
        }

        // Check that the From & To channels are the same
        let from_structure_sc = from_portal.components.get(&structures_ref.structure).unwrap();
        let from_structure = ComponentStructure::try_from_slice(&from_structure_sc.data.as_slice()).unwrap();
        
        let to_structure_sc = to_portal.components.get(&structures_ref.structure).unwrap();
        let to_structure = ComponentStructure::try_from_slice(&to_structure_sc.data.as_slice()).unwrap();

        match from_structure.structure {
            StructureType::Portal { channel } => {
                let from_channel = channel;
                match to_structure.structure {
                    StructureType::Portal { channel } => {
                        if from_channel != channel {
                            return err!(StructureError::PortalChannelMismatch)
                        }
                    }   
                    _ => {
                        return err!(StructureError::WrongStructure)
                    }
                }
            }
            _ => {
                return err!(StructureError::WrongStructure)
            }
        }

        // Pay the Fee for the Portal
        let transfer_accounts = Transfer{
            from: ctx.accounts.user_ata.to_account_info(),
            to: ctx.accounts.kyogen_ata.to_account_info(),
            authority: ctx.accounts.payer.to_account_info()
        };

        transfer(CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            transfer_accounts,
        ), from_structure.cost)?;

        //// Updates
        let system_signer_seeds:&[&[u8]] = &[
            SEEDS_STRUCTURESSIGNER,
            &[*ctx.bumps.get("structures_config").unwrap()]
        ];
        let signer_seeds = &[system_signer_seeds];

        // Update from.occupant is None
        from_occupant.occupant_id = None;
        let modify_from_occupant_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.from.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.structures_config.to_account_info(),
                action_bundle_registration: ctx.accounts.structures_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_from_occupant_ctx, vec![
            (kyogen_ref.occupant, from_occupant.try_to_vec().unwrap()), // Last Used
        ])?;
        
        // Update to.occupant = unit.id
        to_occupant.occupant_id = Some(unit.entity_id);
        let modify_to_occupant_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.to.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.structures_config.to_account_info(),
                action_bundle_registration: ctx.accounts.structures_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_to_occupant_ctx, vec![
            (kyogen_ref.occupant, to_occupant.try_to_vec().unwrap()), // Last Used
        ])?;

        // Update from_portal last used
        from_last_used.last_used = clock.slot;
        let modify_from_portal_last_used_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.from_portal.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.structures_config.to_account_info(),
                action_bundle_registration: ctx.accounts.structures_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_from_portal_last_used_ctx, vec![
            (kyogen_ref.last_used, from_last_used.try_to_vec().unwrap()), // Last Used
        ])?;

        // Update unit last used
        unit_last_used.last_used = clock.slot;
        let modify_unit_last_used_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.unit.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.structures_config.to_account_info(),
                action_bundle_registration: ctx.accounts.structures_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_unit_last_used_ctx, vec![
            (kyogen_ref.last_used, unit_last_used.try_to_vec().unwrap()), // Last Used
        ])?;   

        emit!(PortalUsed{
            from: from.entity_id,
            to: to.entity_id,
            unit: unit.entity_id
        });

        Ok(())
    }

    // Use Lootable
    pub fn use_lootable(ctx:Context<UseLootable>) -> Result<()> {
        // Check the game isnt' paused
        if ctx.accounts.kyogen_index.play_phase != kyogen::account::PlayPhase::Play {
            return err!(StructureError::GamePaused)
        }

        // References
        let kyogen_ref = &ctx.accounts.kyogen_config.components;
        let structures_ref = &ctx.accounts.structures_config.components;

        let tile = &ctx.accounts.tile;
        let unit = &ctx.accounts.unit;
        let lootable = &ctx.accounts.lootable;
        let pack = &ctx.accounts.pack;
        let player = &ctx.accounts.player;

        // Check if Tile.location == Structre.location
        let tile_location_sc = tile.components.get(&kyogen_ref.location).unwrap();
        let tile_location = ComponentLocation::try_from_slice(&tile_location_sc.data.as_slice()).unwrap();
        let lootable_location_sc = lootable.components.get(&kyogen_ref.location).unwrap();
        let lootable_location = ComponentLocation::try_from_slice(&lootable_location_sc.data.as_slice()).unwrap();

        if tile_location.x != lootable_location.x || 
           tile_location.y != lootable_location.y {
            return err!(StructureError::WrongStructure)
        }

        // Check if Tile.occupant == unit.id
        let tile_occupant_sc = tile.components.get(&kyogen_ref.location).unwrap();
        let tile_occupant = ComponentOccupant::try_from_slice(&tile_occupant_sc.data.as_slice()).unwrap();
        if tile_occupant.occupant_id.unwrap() != unit.entity_id {
            return err!(StructureError::InvalidUnit)
        }

        // Unit.owner == payer
        let unit_owner_sc = unit.components.get(&kyogen_ref.owner).unwrap();
        let unit_owner = ComponentOwner::try_from_slice(&unit_owner_sc.data.as_slice()).unwrap();
        if unit_owner.owner.unwrap().key() != ctx.accounts.payer.key() {
            return err!(StructureError::InvalidUnit)
        }

        // Player owner is payer
        let player_stats_sc = player.components.get(&kyogen_ref.player_stats).unwrap();
        let mut player_stats = ComponentPlayerStats::try_from_slice(&player_stats_sc.data.as_slice()).unwrap();
        if player_stats.key.key() != ctx.accounts.payer.key() {
            return err!(StructureError::InvalidOwner)
        }

        // Lootable.last_used isn't violated
        let clock = Clock::get().unwrap();
        let lootable_last_used_c = lootable.components.get(&kyogen_ref.last_used).unwrap();
        let mut lootable_last_used = ComponentLastUsed::try_from_slice(&lootable_last_used_c.data.as_slice()).unwrap();
        if lootable_last_used.last_used != 0 && (lootable_last_used.last_used + lootable_last_used.recovery) >= clock.slot {
            return err!(StructureError::StructureInCooldown)
        }

        // Unit.last_used isn't violated
        let unit_last_used_c = unit.components.get(&kyogen_ref.last_used).unwrap();
        let mut unit_last_used = ComponentLastUsed::try_from_slice(&unit_last_used_c.data.as_slice()).unwrap();
        if unit_last_used.last_used != 0 && (unit_last_used.last_used + unit_last_used.recovery) >= clock.slot {
            return err!(StructureError::StructureInCooldown)
        }

        // Match Lootable pack with Player clan, get random number
        let lootable_structure_sc = lootable.components.get(&structures_ref.structure).unwrap();
        let lootable_structure = ComponentStructure::try_from_slice(&lootable_structure_sc.data.as_slice()).unwrap();

        let pack_key;
        match lootable_structure.structure {
            StructureType::Lootable { ancients_pack, wildings_pack, creepers_pack, synths_pack } => {
                match player_stats.clan {
                    Clans::Ancients => {
                        pack_key = ancients_pack;
                    }
                    Clans::Wildings => {
                        pack_key = wildings_pack;
                    }
                    Clans::Creepers => {
                        pack_key = creepers_pack;
                    }
                    Clans::Synths => {
                        pack_key = synths_pack;
                    }
                }
            }
            _=> {
                return err!(StructureError::WrongStructure)
            }
        }
        if pack_key.key() != pack.key() {
            return err!(StructureError::InvalidPack)
        }
        
        let random_card_idx = get_random_u64(pack.blueprints.len() as u64 - 1 );
        // TODO Add error handling for exceeding max cards
        player_stats.cards.push(pack.blueprints.get(random_card_idx as usize).unwrap().clone());

        // Update Structure Last Used
        let system_signer_seeds:&[&[u8]] = &[
            SEEDS_STRUCTURESSIGNER,
            &[*ctx.bumps.get("structures_config").unwrap()]
        ];
        let signer_seeds = &[system_signer_seeds];

        lootable_last_used.last_used = clock.slot;
        let modify_structure_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.lootable.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.structures_config.to_account_info(),
                action_bundle_registration: ctx.accounts.structures_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_structure_ctx, vec![
            (kyogen_ref.last_used, lootable_last_used.try_to_vec().unwrap()), // Last Used
        ])?;

        // Update Unit Last Used
        unit_last_used.last_used = clock.slot;
        let modify_unit_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.unit.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.structures_config.to_account_info(),
                action_bundle_registration: ctx.accounts.structures_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_unit_ctx, vec![
            (kyogen_ref.last_used, lootable_last_used.try_to_vec().unwrap()), // Last Used
        ])?;

        // Update Player Hand
        let modify_player_ctx = CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),            
            registry::cpi::accounts::ModifyComponent {
                entity: ctx.accounts.player.to_account_info(),
                registry_config: ctx.accounts.registry_config.to_account_info(),
                action_bundle: ctx.accounts.structures_config.to_account_info(),
                action_bundle_registration: ctx.accounts.structures_registration.to_account_info(),
                core_ds: ctx.accounts.coreds.to_account_info(),                
            }, 
            signer_seeds
        );
        registry::cpi::req_modify_component(modify_player_ctx, vec![
            (kyogen_ref.player_stats, player_stats.try_to_vec().unwrap()),
        ])?;
        
        Ok(())
    }

    // Use Healer

    // Claim Victory

}

pub fn get_random_u64(max: u64) -> u64 {
    let clock = Clock::get().unwrap();
    let slice = &hash(&clock.slot.to_be_bytes()).to_bytes()[0..8];
    let num: u64 = u64::from_be_bytes(slice.try_into().unwrap());
    let target = num/(u64::MAX/max);
    return target;
}