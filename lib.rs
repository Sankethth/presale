use anchor_lang::prelude::*;
use anchor_lang::{prelude::Pubkey, solana_program::pubkey};
use anchor_spl::token::{self, Token, TokenAccount, Transfer as SplTransfer};

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("3Zur1pLnbCVMJFimT2XdsYf1HbanEuVhvMHqYAQPr8Fz");
pub const PRESALE_SIGNER_SEED: &'static [u8] = b"presale";

#[program]
mod hello_anchor {
    use super::*;
    pub fn initialize_global_account(ctx: Context<Initialize>) -> Result<()> {
        let presale_state = &mut ctx.accounts.presale_state;
        presale_state.bump = ctx.bumps.presale_state;

        msg!("bump {} , ", presale_state.bump);
        Ok(())
    }

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        let user_state_account = &mut ctx.accounts.user_state;
        user_state_account.sol_deposited = 0;
        user_state_account.token_allocated = 0;
        user_state_account.token_withdrawn = 0;

        msg!("Greetings from: {:?}", ctx.program_id);

        Ok(())
    }

    pub fn deposit_sol(ctx: Context<DepositSol>, sol_deposited: u64) -> Result<()> {
        let user_state = &mut ctx.accounts.user_state;

        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.user.to_account_info(),
                    to: ctx.accounts.presale_state.to_account_info(),
                },
            ),
            sol_deposited,
        );

        user_state.sol_deposited += sol_deposited;
        user_state.token_allocated = sol_deposited * 1000;
        user_state.token_withdrawn = 0;
        Ok(())
    }

    pub fn transfer_tokens(ctx: Context<TransferToken>) -> Result<()> {
        let from_account = &mut ctx.accounts.from;
        let to_account = &mut ctx.accounts.to;
        let token_program = &mut ctx.accounts.token_program;

        let user_state = &mut ctx.accounts.user_state;
        let presale_state = &mut ctx.accounts.presale_state;

        let seeds = &[PRESALE_SIGNER_SEED, &[presale_state.bump]];

        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = SplTransfer {
            from: from_account.to_account_info(),
            to: to_account.to_account_info(),
            authority: presale_state.to_account_info(),
        };
        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        let tokens_to_transfer = user_state.token_allocated - user_state.token_withdrawn;

        if tokens_to_transfer <= 0 {
            panic!("Cannot Withdraw");
        }

        token::transfer(cpi_ctx, tokens_to_transfer)?;
        user_state.token_withdrawn += tokens_to_transfer;

        msg!("Tokens Transfered {}", tokens_to_transfer);

        Ok(())
    }

    pub fn transfer_sol(ctx: Context<TransferSol>, amount: u64) -> Result<()> {
        let presale_state = &mut ctx.accounts.presale_state;
        let user = &mut ctx.accounts.user;
        let to = &mut ctx.accounts.to;
        let seeds = &[PRESALE_SIGNER_SEED, &[presale_state.bump]];

        // let ix = anchor_lang::solana_program::system_instruction::transfer(
        //     &presale_state.key(),
        //     &to.key(),
        //     amount,
        // );
        // anchor_lang::solana_program::program::invoke_signed(
        //     &ix,
        //     &[presale_state.to_account_info(), to.to_account_info(),user.to_account_info()],
        //     &[&seeds[..]],
        // )?;
       **ctx.accounts.presale_state.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += amount;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init,seeds = [b"presale", user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 32
    )]
    pub user_state: Account<'info, UserState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut,seeds = [b"presale", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    #[account(mut, seeds = [b"presale"],
        bump
    )]
    pub presale_state: Account<'info, PresaleState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = pubkey!("AceJtsd3F1KKuAao4C2EUoQf9ghPC9x39PttyZKGzP6T"),//address of the mint(token)
        associated_token::authority = presale_state
     )]
    pub from: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"presale"], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,

    #[account(
        mut,
        associated_token::mint = pubkey!("AceJtsd3F1KKuAao4C2EUoQf9ghPC9x39PttyZKGzP6T"),
        associated_token::authority = user,
    )]
    pub to: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    #[account(mut,seeds = [b"presale", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init,seeds = [b"presale"],
        bump,
        payer = user,
        space = 8 + 16
    )]
    pub presale_state: Account<'info, PresaleState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"presale"], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,


    #[account(mut)]
    pub to: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}
#[account]
pub struct PresaleState {
    pub bump: u8,
}

#[account]
pub struct UserState {
    pub sol_deposited: u64,
    pub token_allocated: u64,
    pub token_withdrawn: u64,
}
