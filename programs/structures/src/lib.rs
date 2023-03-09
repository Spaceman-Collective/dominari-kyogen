use anchor_lang::prelude::*;

use kyogen::component::*;
use core_ds::state::SerializedComponent;
use std::collections::BTreeMap;

pub mod context;
pub mod account;
pub mod state;
pub mod constant;
pub mod component;

use context::*;
//use account::*;
use state::*;
use constant::*;
use component::*;

declare_id!("4Bo4cgr4RhGpXJsQUV4KENCf3HJwPveFsPELJGGN9GkR");

#[program]
pub mod structures {
    use core_ds::account::MaxSize;

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
                entity: ctx.accounts.structure_entity.to_account_info(),
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
    
    // Use Structure (each structure has a unique function)

}