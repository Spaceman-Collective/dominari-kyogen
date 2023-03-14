use anchor_lang::prelude::*;

use kyogen::component::*;
use core_ds::state::SerializedComponent;
use std::collections::BTreeMap;
use core_ds::account::MaxSize;
use anchor_spl::token::{Transfer, transfer};

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
    pub fn init_index(ctx:Context<InitStructureIndex>, _instance:u64) -> Result<()> {
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
            ctx.accounts.structure_index.portal.push(entity_id);
        } else if matches!(structure_type.structure, StructureType::Lootable { .. }) {
            ctx.accounts.structure_index.portal.push(entity_id);
        } else if matches!(structure_type.structure, StructureType::Meteor { .. }) {
            ctx.accounts.structure_index.portal.push(entity_id);
        }

        Ok(())
    }

    // Use Meteor    
        // Transfer Solarite from SI to User
    pub fn use_meteor(ctx:Context<UseMeteor>) -> Result<()> {
        let kyogen_ref = &ctx.accounts.kyogen_config.components;
        let structures_ref = &ctx.accounts.structures_config.components;
        
        let tile = &ctx.accounts.tile;
        let unit = &ctx.accounts.unit;
        let meteor = &ctx.accounts.meteor;

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
                    from: ctx.accounts.user_ata.to_account_info(),
                    to: ctx.accounts.structures_ata.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info()
                };
        
                transfer(CpiContext::new(
                    ctx.accounts.token_program.to_account_info(), 
                    transfer_accounts,
                ), solarite_per_use)?;
            },
            _ => {
                return err!(StructureError::WrongStructure)
            }
        }        
        
        emit!(MeteorMined{
            tile: ctx.accounts.tile.entity_id,
            player: occupant.occupant_id.unwrap()
        });

        Ok(())
    }

    /* 
    // Use Portal
    pub fn use_portal(ctx:Context<UsePortal>) -> Result<()> {
        let kyogen_ref = &ctx.accounts.kyogen_config.components;
        let structures_ref = &ctx.accounts.structures_config.components;

        // Make sure from.occupant is not unit.id
        // Make sure unit.owner == payer
        // Make sure to.occupant is none
        // Make sure from.last_used isn't violated
        // Make sure unit.last_used isn't violated

        // Update from.occupant is None
        // Update to.occupant = unit.id
        // Update unit lastused
        Ok(())
    }
    */


    // Use Healer
    // Use Lootable

    // Claim Victory
        // Check that SI ATA == 0 (should be held by users or in KI ATA)
}