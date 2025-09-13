use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod citychests_vault {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>, vault: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.vault = vault;
        config.bump = *ctx.bumps.get("config").unwrap();
        Ok(())
    }

    pub fn create_mint_record(
        ctx: Context<CreateMintRecord>,
        recipient: Pubkey,
        rarity: u8,
        client_nonce: u64,
    ) -> Result<()> {
        let config = &ctx.accounts.config;
        require_keys_eq!(ctx.accounts.admin.key(), config.admin, CityChestsError::Unauthorized);

        let record = &mut ctx.accounts.record;
        record.recipient = recipient;
        record.rarity = rarity;
        record.mint = Pubkey::default();
        record.minted = false;
        record.client_nonce = client_nonce;
        record.created_at = Clock::get()?.unix_timestamp;
        emit!(MintRecordCreated {
            recipient,
            rarity,
            client_nonce,
        });
        Ok(())
    }

    pub fn confirm_mint(
        ctx: Context<ConfirmMint>,
        recipient: Pubkey,
        client_nonce: u64,
        mint: Pubkey,
    ) -> Result<()> {
        let config = &ctx.accounts.config;
        require_keys_eq!(ctx.accounts.admin.key(), config.admin, CityChestsError::Unauthorized);

        let vault_ata = &ctx.accounts.vault_ata;
        require_keys_eq!(vault_ata.owner, config.vault, CityChestsError::WrongVaultOwner);
        require_keys_eq!(vault_ata.mint, mint, CityChestsError::WrongMint);
        require!(vault_ata.amount == 1, CityChestsError::WrongAmount);

        let record = &mut ctx.accounts.record;
        record.mint = mint;
        record.minted = true;
        emit!(MintConfirmed {
            recipient,
            mint,
            client_nonce,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Config::LEN,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(recipient: Pubkey, client_nonce: u64)]
pub struct CreateMintRecord<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = admin,
        space = 8 + MintRecord::LEN,
        seeds = [b"record", recipient.as_ref(), &client_nonce.to_le_bytes()],
        bump
    )]
    pub record: Account<'info, MintRecord>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(recipient: Pubkey, client_nonce: u64, mint: Pubkey)]
pub struct ConfirmMint<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"record", recipient.as_ref(), &client_nonce.to_le_bytes()],
        bump
    )]
    pub record: Account<'info, MintRecord>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        constraint = vault_ata.owner == config.vault @ CityChestsError::WrongVaultOwner,
        constraint = vault_ata.mint == mint @ CityChestsError::WrongMint,
        constraint = vault_ata.amount == 1 @ CityChestsError::WrongAmount
    )]
    pub vault_ata: Account<'info, TokenAccount>,
}

#[account]
pub struct Config {
    pub admin: Pubkey,
    pub vault: Pubkey,
    pub bump: u8,
}

impl Config {
    pub const LEN: usize = 32 + 32 + 1;
}

#[account]
pub struct MintRecord {
    pub recipient: Pubkey,
    pub rarity: u8,
    pub mint: Pubkey,
    pub minted: bool,
    pub client_nonce: u64,
    pub created_at: i64,
}

impl MintRecord {
    pub const LEN: usize = 32 + 1 + 32 + 1 + 8 + 8;
}

#[event]
pub struct MintRecordCreated {
    pub recipient: Pubkey,
    pub rarity: u8,
    pub client_nonce: u64,
}

#[event]
pub struct MintConfirmed {
    pub recipient: Pubkey,
    pub mint: Pubkey,
    pub client_nonce: u64,
}

#[error_code]
pub enum CityChestsError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Wrong vault owner")]
    WrongVaultOwner,
    #[msg("Wrong mint")]
    WrongMint,
    #[msg("Wrong amount")]
    WrongAmount,
}
