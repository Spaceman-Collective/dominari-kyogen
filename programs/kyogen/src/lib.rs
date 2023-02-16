use anchor_lang::prelude::*;

declare_id!("CTQCiB97LrAjAtHy1eqGwqGiy2mjefBXR762nrDhWYTL");

#[program]
pub mod kyogen {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
