use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, MintTo};
// use std::mem::size_of;

declare_id!("9eNoMwnzGs7FCEMD4RscEGgfCGJmbwDme9bAKNPXjyNy");

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
        let presale = &mut ctx.accounts.presale;

        // Adding a debug statement
        msg!("Creating presale with owner: {}", _owner);
        msg!("Token: {}", _token);
        msg!("Hard Cap: {}", _hard_cap);

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

        msg!("Presale account initialized: {}", presale.key());

        let mint_account_info = ctx.accounts.token_mint.to_account_info();
        msg!("Token Mint Initialized: {}", mint_account_info.data_len() > 0);

        // // Check if the mint account is already initialized
        // if ctx.accounts.token_mint.supply == 0 {
        //     // Mint not initialized, so initialize it
        //     let cpi_accounts = InitializeMint {
        //         mint: ctx.accounts.token_mint.to_account_info(),
        //         rent: ctx.accounts.rent.to_account_info(),
        //     };
        //     let cpi_program = ctx.accounts.token_program.to_account_info();
        //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        //     token::initialize_mint(cpi_ctx, 9, &presale.owner, Some(&presale.owner))?;
        // }

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

    pub fn buy_tokens(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
        let presale = &mut ctx.accounts.presale;
        let clock = Clock::get()?;

        // Check if presale is active
        if clock.unix_timestamp < presale.start_sale {
            return Err(ErrorCode::PresaleNotStarted.into());
        }
        if clock.unix_timestamp > presale.end_sale {
            return Err(ErrorCode::PresaleEnded.into());
        }

        // Check if sale is finalized
        if presale.is_finalized {
            return Err(ErrorCode::PresaleAlreadyFinalized.into());
        }

        // Whitelist check
        if presale.is_whitelist && !presale.participants.contains(&ctx.accounts.buyer.key()) {
            return Err(ErrorCode::Unauthorized.into());
        }

        // Check funding cap
        if presale.funds_raised.checked_add(amount).ok_or(ErrorCode::FundingCapExceeded)? > presale.hard_cap {
            return Err(ErrorCode::FundingCapExceeded.into());
        }

        // Enforce min and max buy
        if amount < presale.min_buy {
            return Err(ErrorCode::AmountTooLow.into());
        }
        if amount > presale.max_buy {
            return Err(ErrorCode::AmountTooHigh.into());
        }

        // Calculate tokens to buy
        let tokens_to_buy = amount.checked_div(presale.presale_rate).ok_or(ErrorCode::InsufficientTokens)?;

        if presale.tokens_sold + tokens_to_buy > presale.hard_cap {
            return Err(ErrorCode::InsufficientTokens.into());
        }

        // Handle payment transfer
        if presale.is_native {
            // Handling native currency (e.g., SOL)
            let cpi_accounts = anchor_lang::system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.presale_vault.to_account_info(),
            };
            let cpi_program = ctx.accounts.system_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            anchor_lang::system_program::transfer(cpi_ctx, amount)?;
        } else {
            // Handling SPL tokens
            let cpi_accounts_transfer = Transfer {
                from: ctx.accounts.buyer_payment_account.to_account_info(),
                to: ctx.accounts.presale_payment_vault.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            };
            let cpi_program_transfer = ctx.accounts.token_program.to_account_info();
            let cpi_ctx_transfer = CpiContext::new(cpi_program_transfer, cpi_accounts_transfer);
            token::transfer(cpi_ctx_transfer, amount)?;
        }

        // Transfer tokens from token vault to buyer
        let cpi_accounts_transfer = Transfer {
            from: ctx.accounts.token_vault.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program_transfer = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_transfer = CpiContext::new(cpi_program_transfer, cpi_accounts_transfer);
        token::transfer(cpi_ctx_transfer, tokens_to_buy)?;

        // Update presale state
        presale.tokens_sold += tokens_to_buy;
        presale.funds_raised += amount;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeFactory<'info> {
    #[account(init, payer = owner, space = 8 + Factory::MAX_SIZE)]
    pub factory: Account<'info, Factory>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePresale<'info> {
    #[account(mut, has_one = owner)]
    pub factory: Account<'info, Factory>,
    #[account(init, payer = owner, space = 8 + Presale::MAX_SIZE)] // Adjust space as needed
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

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub presale: Account<'info, Presale>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,  // Add the owner as a signer
    /// CHECK: Presale vault (SOL or SPL tokens)
    #[account(mut)]
    pub presale_vault: AccountInfo<'info>,
    /// CHECK: Presale payment vault (for SPL tokens)
    #[account(mut)]
    pub presale_payment_vault: AccountInfo<'info>,
    /// CHECK: Buyer payment token account (if not native)
    #[account(mut)]
    pub buyer_payment_account: AccountInfo<'info>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>, 
}

#[account]
pub struct Factory {
    pub owner: Pubkey,          // Owner of the Factory
    pub presale_count: u64,     // Number of presales created
    pub platform_fee: u64,      // Platform fee in basis points (e.g., 500 = 5%)
}

impl Factory {
  pub const MAX_SIZE: usize = 32 + 8 + 8;
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
    pub buyers: Vec<Buyer>,
}

impl Presale {
  pub const MAX_SIZE: usize = (7 * 32) + (4 + 100 * 40) +  (4 + 100 * 32) + (16 * 8) + (1 * 6);
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct Buyer {
    pub buyer: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Soft cap cannot be greater than hard cap")]
    InvalidCap,
    #[msg("Start time must be before end time")]
    InvalidTime,
    #[msg("Min buy must be less than or equal to max buy")]
    InvalidMinMax,
     #[msg("Presale has not started yet.")]
    PresaleNotStarted,
    #[msg("Presale has already ended.")]
    PresaleEnded,
    #[msg("Funding cap exceeded.")]
    FundingCapExceeded,
    #[msg("Insufficient tokens available.")]
    InsufficientTokens,
    #[msg("Presale is already finalized.")]
    PresaleAlreadyFinalized,
    #[msg("Unauthorized access.")]
    Unauthorized,
    #[msg("Invalid fee percentage.")]
    InvalidFee,
    #[msg("Whitelist not enabled.")]
    WhitelistNotEnabled,
    #[msg("Amount is below the minimum purchase limit.")]
    AmountTooLow,
    #[msg("Amount exceeds the maximum purchase limit.")]
    AmountTooHigh,
    #[msg("Insufficient funds.")]
    InsufficientFunds,
    #[msg("Presale has not ended yet.")]
    PresaleNotEnded,
}