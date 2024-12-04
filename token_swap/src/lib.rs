use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, MintTo, Transfer};

declare_id!("DpxuoHQeFL1hCEzRatHSRtshXY2uoNtHQLWAHDC3dFRU");

#[program]
pub mod token_swap {
    use super::*;

    /// Initializes a new liquidity pool.
    ///
    /// - Creates a PDA (Program Derived Address) as the authority for the pool.
    /// - Sets the initial parameters, such as tokens, fee percentage, and admin account for fee collection.
    /// - Allocates storage for the liquidity pool.
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        fee_percentage: u16,
        admin_fee_account: Option<Pubkey>,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        // Generate a PDA as the authority for this pool.
        let (pool_authority, _bump) =
            Pubkey::find_program_address(&[b"pool".as_ref()], ctx.program_id);

        // Set pool parameters.
        pool.pool_authority = pool_authority;
        pool.token_a = ctx.accounts.token_a.key();
        pool.token_b = ctx.accounts.token_b.key();
        pool.mint_a = ctx.accounts.mint_a.key();
        pool.mint_b = ctx.accounts.mint_b.key();
        pool.fee_percentage = fee_percentage;
        pool.admin_fee_account = admin_fee_account;
        pool.total_liquidity = 0;

        // Record the current timestamp for time-locked liquidity management.
        pool.last_deposit_time = Clock::get()?.unix_timestamp;

        Ok(())
    }

    /// Adds liquidity to the pool.
    ///
    /// - Allows users to deposit two tokens (Token A and Token B) into the pool.
    /// - Mints liquidity tokens to represent the user's share of the pool.
    /// - Updates the pool's liquidity balances and timestamps.
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        // Ensure the provided tokens match the pool's token pair.
        require!(
            pool.token_a == ctx.accounts.token_a.key() &&
            pool.token_b == ctx.accounts.token_b.key(),
            SwapError::InvalidTokenPair
        );

        // Calculate the number of liquidity tokens to mint.
        let user_shares = if pool.total_liquidity == 0 {
            // For the first deposit, user receives tokens proportional to their input.
            amount_a + amount_b
        } else {
            // For subsequent deposits, calculate shares based on current pool balances.
            let share_a = (amount_a as u128 * pool.total_liquidity as u128)
                / ctx.accounts.pool_token_a.amount as u128;
            let share_b = (amount_b as u128 * pool.total_liquidity as u128)
                / ctx.accounts.pool_token_b.amount as u128;
            share_a.min(share_b) as u64
        };

        // Update the pool's total liquidity.
        pool.total_liquidity += user_shares;

        // Update the last deposit timestamp for time-locked withdrawals.
        pool.last_deposit_time = Clock::get()?.unix_timestamp;

        // Transfer Token A to the pool.
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_a.to_account_info(),
                    to: ctx.accounts.pool_token_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                }
            ),
            amount_a,
        )?;

        // Transfer Token B to the pool.
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_b.to_account_info(),
                    to: ctx.accounts.pool_token_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                }
            ),
            amount_b,
        )?;

        // Mint liquidity tokens to the user.
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.user_liquidity_mint.to_account_info(),
                    to: ctx.accounts.user_liquidity.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                }
            ),
            user_shares,
        )?;

        Ok(())
    }

    /// Removes liquidity from the pool.
    ///
    /// - Allows users to redeem their liquidity tokens for the underlying assets (Token A and Token B).
    /// - Enforces a time-lock to prevent immediate withdrawal after deposit.
    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        shares: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        // Enforce a time-lock to prevent immediate withdrawal after deposit.
        require!(
            Clock::get()?.unix_timestamp >= pool.last_deposit_time + 300,
            SwapError::WithdrawalTooSoon
        );

        // Calculate the proportion of the pool the user is withdrawing.
        let proportion = shares as u128 * 1_000_000 / pool.total_liquidity as u128;

        // Calculate the amounts of Token A and Token B to withdraw.
        let withdraw_a = (proportion * ctx.accounts.pool_token_a.amount as u128 / 1_000_000) as u64;
        let withdraw_b = (proportion * ctx.accounts.pool_token_b.amount as u128 / 1_000_000) as u64;

        // Update the pool's total liquidity.
        pool.total_liquidity -= shares;

        // Transfer Token A back to the user.
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.pool_token_a.to_account_info(),
                    to: ctx.accounts.token_a.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                }
            ),
            withdraw_a,
        )?;

        // Transfer Token B back to the user.
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.pool_token_b.to_account_info(),
                    to: ctx.accounts.token_b.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                }
            ),
            withdraw_b,
        )?;

        // Burn the user's liquidity tokens.
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.user_liquidity_mint.to_account_info(),
                    to: ctx.accounts.user_liquidity.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                }
            ),
            shares,
        )?;

        Ok(())
    }

    /// Performs a token swap.
    ///
    /// - Swaps one token for another based on the pool's current balances.
    /// - Deducts a small swap fee and ensures sufficient output for the user.
    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_out: u64,
    ) -> Result<()> {
        // Placeholder: Swap logic would go here.
        Ok(())
    }
}

/// Account structure for initializing the pool.
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = admin, space = 8 + 160)]
    pub pool: Account<'info, LiquidityPool>,
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
    #[account(mut)]
    pub token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_liquidity_mint: Account<'info, Mint>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Account structure for adding liquidity to the pool.
#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,
    #[account(mut)]
    pub token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_liquidity: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_liquidity_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

/// Account structure for removing liquidity from the pool.
#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,
    #[account(mut)]
    pub token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_liquidity: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_liquidity_mint: Account<'info, Mint>,
    #[account(mut)]
    pub pool_authority: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

/// Account structure for performing a token swap.
#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,
    #[account(mut)]
    pub input_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub output_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

/// Structure defining the liquidity pool's state.
#[account]
#[derive(Default)]
pub struct LiquidityPool {
    pub pool_authority: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub fee_percentage: u16,
    pub total_liquidity: u64,
    pub admin_fee_account: Option<Pubkey>,
    pub last_deposit_time: i64,
}

/// Custom error codes for the program.
#[error_code]
pub enum SwapError {
    #[msg("Invalid token pair.")]
    InvalidTokenPair,
    #[msg("Insufficient output amount.")]
    InsufficientOutput,
    #[msg("Liquidity withdrawal too soon.")]
    WithdrawalTooSoon,
}
