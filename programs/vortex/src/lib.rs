use anchor_lang::prelude::*;

declare_id!("71kECueXZuecQ7ngyxbThU22XyTM1jfk4SpGk7PSVbGY");

#[program]
pub mod vortex {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
