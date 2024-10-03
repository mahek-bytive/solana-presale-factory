use anchor_lang::prelude::*;
// use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, InitializeMint, MintTo, Burn};
use anchor_spl::token::{self, Mint, Token, TokenAccount, InitializeMint, MintTo};
// use anchor_spl::token::assert_initialized;
// use std::collections::HashSet;

declare_id!("AjDwp2hFQaKF6ntfiG9xmfAwsRH8vTCmbC8ydvqfYZ4q");

#[program]
pub mod solana_presale_factory {
    use super::*;

    /// Initializes the Factory with the owner and platform fee
    pub fn initialize_factory(ctx: Context<InitializeFactory>, platform_fee: u64) -> Result<()> {
        let factory = &mut ctx.accounts.factory;
        factory.owner = *ctx.accounts.owner.key;
        factory.presale_count = 0;
        factory.platform_fee = platform_fee; // e.g., 500 for 5% (basis points)
        Ok(())
    }

     /// Creates a new Presale with specified parameters
    pub fn create_presale(
        ctx: Context<CreatePresale>,
        _owner: Pubkey,               // _owner
        _token: Pubkey,               // _token
        _payment_token: Pubkey,       // _paymentToken
        _dex_router: Pubkey,          // _dexRouter
        _presale_rate: u64,           // _presaleRate
        _soft_cap: u64,               // _softCap
        _hard_cap: u64,               // _hardCap
        _min_buy: u64,                // _minBuy
        _max_buy: u64,                // _maxBuy
        _start_sale: i64,             // _startSale
        _end_sale: i64,               // _endSale
        _liquidity_percent: u64,      // _liquidityPercent
        _is_fund: bool,               // _isFund
        _is_native: bool,             // _isNative
        _is_whitelist: bool,          // _isWhitelist
        _is_auto_listing: bool,       // _isAutoListing
        _is_vesting: bool,            // _isVesting
        _first_release_percent: u64,  // _firstReleasePercent
        _vesting_period: u64,         // _vestingPeriod
        _tokens_release_percent: u64, // _tokensReleasePercent
        _listing_rate: u64,           // _listingRate
        _demy_address: Pubkey,        // _demyAddress
        _liquidity_time: u64,         // _liquidityTime
        _qerralock: Pubkey,           // _qerralock
        _uniswap_factory: Pubkey,     // _uniswapFactory
    ) -> Result<()> {
        let presale_key = ctx.accounts.presale.key();
        let owner_key = ctx.accounts.owner.key();

        let presale = &mut ctx.accounts.presale;

        // Validate parameters
        if _soft_cap > _hard_cap {
            return Err(ErrorCode::InvalidCap.into());
        }
        if _start_sale >= _end_sale {
            return Err(ErrorCode::InvalidTime.into());
        }
        if _min_buy > _max_buy {
            return Err(ErrorCode::InvalidMinMax.into());
        }

        // Initialize Presale fields
        presale.owner = _owner;
        presale.token = _token;
        presale.payment_token = _payment_token;
        presale.dex_router = _dex_router;
        presale.presale_rate = _presale_rate;
        presale.soft_cap = _soft_cap;
        presale.hard_cap = _hard_cap;
        presale.min_buy = _min_buy;
        presale.max_buy = _max_buy;
        presale.start_sale = _start_sale;
        presale.end_sale = _end_sale;
        presale.liquidity_percent = _liquidity_percent;
        presale.is_fund = _is_fund;
        presale.is_native = _is_native;
        presale.is_whitelist = _is_whitelist;
        presale.is_auto_listing = _is_auto_listing;
        presale.is_vesting = _is_vesting;
        presale.first_release_percent = _first_release_percent;
        presale.vesting_period = _vesting_period;
        presale.tokens_release_percent = _tokens_release_percent;
        presale.listing_rate = _listing_rate;
        presale.demy_address = _demy_address;
        presale.liquidity_time = _liquidity_time;
        presale.qerralock = _qerralock;
        presale.uniswap_factory = _uniswap_factory;
        presale.tokens_sold = 0;
        presale.funds_raised = 0;
        presale.is_finalized = false;

        // Increment presale count in Factory
        let factory = &mut ctx.accounts.factory;
        factory.presale_count += 1;

        // Calculate platform fee
        let fee = (_hard_cap * factory.platform_fee) / 10_000; // Assuming platform_fee is in basis points
        presale.platform_fee = fee;

        // Emit event
        emit!(PresaleCreated {
            presale: presale_key,
            owner: owner_key,
            start_sale: _start_sale,
            end_sale: _end_sale,
        });

        // Check if the mint account is already initialized
        if ctx.accounts.token_mint.supply == 0 {
            // Mint not initialized, so initialize it
            let cpi_accounts = InitializeMint {
                mint: ctx.accounts.token_mint.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::initialize_mint(cpi_ctx, 9, &presale.owner, Some(&presale.owner))?;
        }

        // Mint tokens to the token vault
        let cpi_accounts_mint = MintTo {
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.token_vault.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program_mint = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_mint = CpiContext::new(cpi_program_mint, cpi_accounts_mint);
        token::mint_to(cpi_ctx_mint, presale.hard_cap)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeFactory<'info> {
    #[account(init, payer = owner, space = 8 + 56)]
    pub factory: Account<'info, Factory>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePresale<'info> {
    #[account(mut, has_one = owner)]
    pub factory: Account<'info, Factory>,
    #[account(init, payer = owner, space = 8 + 3636)] // Adjust space as needed
    pub presale: Account<'info, Presale>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: Presale vault (SOL or SPL tokens)
    #[account(mut)]
    pub presale_vault: AccountInfo<'info>,
    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>, // Treat token_mint as a `Mint` type
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Factory {
    pub owner: Pubkey,          // Owner of the Factory
    pub presale_count: u64,     // Number of presales created
    pub platform_fee: u64,      // Platform fee in basis points (e.g., 500 = 5%)
}

#[account]
pub struct Presale {
    pub owner: Pubkey,
    pub token: Pubkey,
    pub payment_token: Pubkey,
    pub dex_router: Pubkey,
    pub presale_rate: u64,
    pub soft_cap: u64,
    pub hard_cap: u64,
    pub min_buy: u64,
    pub max_buy: u64,
    pub start_sale: i64,
    pub end_sale: i64,
    pub liquidity_percent: u64,
    pub is_fund: bool,
    pub is_native: bool,
    pub is_whitelist: bool,
    pub is_auto_listing: bool,
    pub is_vesting: bool,
    pub first_release_percent: u64,
    pub vesting_period: u64,
    pub tokens_release_percent: u64,
    pub listing_rate: u64,
    pub demy_address: Pubkey,
    pub liquidity_time: u64,
    pub qerralock: Pubkey,
    pub uniswap_factory: Pubkey,
    pub tokens_sold: u64,
    pub funds_raised: u64,
    pub is_finalized: bool,
    pub platform_fee: u64,
    pub participants: Vec<Pubkey>,     // Whitelisted participants
}

#[event]
pub struct PresaleCreated {
    pub presale: Pubkey,
    pub owner: Pubkey,
    pub start_sale: i64,
    pub end_sale: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Soft cap cannot be greater than hard cap")]
    InvalidCap,
    #[msg("Start time must be before end time")]
    InvalidTime,
    #[msg("Min buy must be less than or equal to max buy")]
    InvalidMinMax,
}