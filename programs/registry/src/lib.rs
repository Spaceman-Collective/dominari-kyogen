use anchor_lang::prelude::*;
use std::collections::{BTreeSet, BTreeMap};
use core_ds::state::SerializedComponent;

declare_id!("7Vpu3mY18uA2iWBhAyKc72F9xs1SaMByV5KaPpuLhFQz");

pub mod account;
pub mod context;
pub mod constant;
pub mod error;
pub mod event;
pub mod state;

//use account::*;
use context::*;
use constant::*;
//use error::*;
//use event::*;
//use state::*;

#[program]
pub mod registry {
    use super::*;

    /**
     * Sets the Registry Config account with Payer as Registry Admin
    */
    pub fn initialize(ctx:Context<Initialize>, core_ds: Pubkey) -> Result<()> {
        ctx.accounts.registry_config.core_ds = core_ds;
        ctx.accounts.registry_config.components = 0;
        ctx.accounts.registry_config.authority = ctx.accounts.payer.key();
        Ok(())
    }

    /**
     * Registered Action Bundles can instance this registry and are given control over the instance 
     */
    pub fn instance_registry(ctx:Context<InstanceRegistry>, instance:u64) -> Result<()> {
        let core_ds = ctx.accounts.core_ds.to_account_info();
        let accounts = core_ds::cpi::accounts::InitRegistryInstance {
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            registry_instance: ctx.accounts.registry_instance.to_account_info(),
            registry_signer: ctx.accounts.registry_config.to_account_info()
        };
        let registry_signer_seeds:&[&[u8]] = &[
            SEEDS_REGISTRYSIGNER,
            &[*ctx.bumps.get("registry_config").unwrap()]
        ];
        let signer_seeds = &[registry_signer_seeds];

        let register_registry_ctx = CpiContext::new_with_signer(
            core_ds,
            accounts,
            signer_seeds
        );

        core_ds::cpi::init_registry(register_registry_ctx, ctx.program_id.key(), instance)?;        
        // Allow this Action Bundle authority over it's own Instance
        ctx.accounts.action_bundle_registration.instances.insert(instance);
        Ok(())
    }
    
    /**
     * Anyone can register a component with the registry as long as it's a unique URI
     */
    pub fn register_component(ctx:Context<RegisterComponent>, schema:String) -> Result<()> {
        ctx.accounts.component.url = schema.clone();
        ctx.accounts.registry_config.components += 1;
        Ok(())
    }

    /**
     * Only the ACTION BUNDLE can register itself, as one of the requirements is it's Signer PDA
     * which is set as it's authority
     */
    pub fn register_action_bundle(ctx: Context<RegisterAB>) -> Result<()> {
        ctx.accounts.action_bundle_registration.action_bundle = ctx.accounts.action_bundle.key();
        ctx.accounts.action_bundle_registration.instances = BTreeSet::new();
        ctx.accounts.action_bundle_registration.can_mint = true;
        Ok(())
    }

    /**
     * Only the Registry Admin can add components to the Action Bundle
     * Prevents AB from adding components they shouldn't have access to.
     */
    pub fn add_components_to_action_bundle_registration(ctx:Context<AddComponentsToActionBundleRegistration>, components:Vec<Pubkey>) -> Result<()> {
        for comp in components {
            ctx.accounts.action_bundle_registration.components.insert(comp);
        }
        Ok(())
    }

    pub fn init_entity(ctx:Context<InitEntity>, entity_id: u64, components: BTreeMap<Pubkey, SerializedComponent>) -> Result<()> {
        let accounts = core_ds::cpi::accounts::InitEntity {
            entity: ctx.accounts.entity.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            registry_instance: ctx.accounts.registry_instance.to_account_info(),
            registry_signer: ctx.accounts.registry_config.to_account_info(),
        };  
        let registry_signer_seeds:&[&[u8]] = &[
            SEEDS_REGISTRYSIGNER,
            &[*ctx.bumps.get("registry_config").unwrap()]
        ];
        let signer_seeds = &[registry_signer_seeds];
        
        core_ds::cpi::init_entity(CpiContext::new_with_signer(
            ctx.accounts.core_ds.to_account_info(),
            accounts,
            signer_seeds
        ), entity_id, components)?;
        
        Ok(())
    }

    pub fn mint_arcnft(ctx:Context<MintARCNFT>) -> Result<()> {
        let accounts = core_ds::cpi::accounts::MintARCNFT {
            entity: ctx.accounts.entity.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            registry_instance: ctx.accounts.registry_instance.to_account_info(),
            registry_signer: ctx.accounts.registry_config.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            arcnft: ctx.accounts.arcnft.to_account_info(),
        };  
        let registry_signer_seeds:&[&[u8]] = &[
            SEEDS_REGISTRYSIGNER,
            &[*ctx.bumps.get("registry_config").unwrap()]
        ];
        let signer_seeds = &[registry_signer_seeds];
        
        core_ds::cpi::mint_arcnft(CpiContext::new_with_signer(
            ctx.accounts.core_ds.to_account_info(),
            accounts,
            signer_seeds
        ))?;
        
        Ok(())
    }

    pub fn req_add_component(ctx:Context<AddComponents>, components: Vec<(Pubkey,SerializedComponent)>) -> Result<()> {
        let accounts = core_ds::cpi::accounts::AddComponent {
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            entity: ctx.accounts.entity.to_account_info(),
            registry_signer: ctx.accounts.registry_config.to_account_info()
        };
        let registry_signer_seeds:&[&[u8]] = &[
            SEEDS_REGISTRYSIGNER,
            &[*ctx.bumps.get("registry_config").unwrap()]
        ];
        let signer_seeds = &[registry_signer_seeds];
        
        core_ds::cpi::add_components(CpiContext::new_with_signer(
            ctx.accounts.core_ds.to_account_info(),
            accounts,
            signer_seeds
        ), components)?;
        Ok(())
    }

    pub fn req_remove_component(ctx:Context<RemoveComponent>, components: Vec<Pubkey>) -> Result<()> {
        let accounts = core_ds::cpi::accounts::RemoveComponent {
            benefactor: ctx.accounts.benefactor.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            entity: ctx.accounts.entity.to_account_info(),
            registry_signer: ctx.accounts.registry_config.to_account_info()
        };
        let registry_signer_seeds:&[&[u8]] = &[
            SEEDS_REGISTRYSIGNER,
            &[*ctx.bumps.get("registry_config").unwrap()]
        ];
        let signer_seeds = &[registry_signer_seeds];
        
        core_ds::cpi::remove_component(CpiContext::new_with_signer(
            ctx.accounts.core_ds.to_account_info(),
            accounts,
            signer_seeds
        ), components)?;
        Ok(())
    }

    pub fn req_modify_component(ctx:Context<ModifyComponent>, components: Vec<(Pubkey, Vec<u8>)>) -> Result<()> {
        let accounts = core_ds::cpi::accounts::ModifyComponent {
            entity: ctx.accounts.entity.to_account_info(),
            registry_signer: ctx.accounts.registry_config.to_account_info()
        };
        let registry_signer_seeds:&[&[u8]] = &[
            SEEDS_REGISTRYSIGNER,
            &[*ctx.bumps.get("registry_config").unwrap()]
        ];
        let signer_seeds = &[registry_signer_seeds];
        
        core_ds::cpi::modify_components(CpiContext::new_with_signer(
            ctx.accounts.core_ds.to_account_info(),
            accounts,
            signer_seeds
        ), components)?;

        Ok(())
    }

    pub fn req_remove_entity(ctx:Context<RemoveEntity>) -> Result<()> {
        let accounts = core_ds::cpi::accounts::RemoveEntity {
            benefactor: ctx.accounts.benefactor.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            entity: ctx.accounts.entity.to_account_info(),
            registry_signer: ctx.accounts.registry_config.to_account_info()
        };
        let registry_signer_seeds:&[&[u8]] = &[
            SEEDS_REGISTRYSIGNER,
            &[*ctx.bumps.get("registry_config").unwrap()]
        ];
        let signer_seeds = &[registry_signer_seeds];
        
        core_ds::cpi::remove_entity(CpiContext::new_with_signer(
            ctx.accounts.core_ds.to_account_info(),
            accounts,
            signer_seeds
        ))?;

        Ok(())
    }

}