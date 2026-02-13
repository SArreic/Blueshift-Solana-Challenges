use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{errors::EscrowError, state::Escrow};

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    
    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
        has_one = maker @ EscrowError::InvalidMaker,
        has_one = mint_a @ EscrowError::InvalidMintA,
        has_one = mint_b @ EscrowError::InvalidMintB,
    )]
    pub escrow: Account<'info, Escrow>,
    
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    fn transfer_to_maker(&self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = anchor_spl::token_interface::TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        anchor_spl::token_interface::transfer_checked(
            cpi_ctx,
            self.escrow.receive,
            self.mint_b.decimals,
        )?;
        Ok(())
    }
    
    fn withdraw_and_close_vault(&self) -> Result<()> {
        let maker_key = self.maker.key();
        let seed_bytes = self.escrow.seed.to_le_bytes();
        
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            maker_key.as_ref(),
            &seed_bytes,
            &[self.escrow.bump],
        ]];
        
        // Transfer from vault to taker
        let transfer_cpi_program = self.token_program.to_account_info();
        let transfer_cpi_accounts = anchor_spl::token_interface::TransferChecked {
            from: self.vault.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        let transfer_cpi_ctx = CpiContext::new_with_signer(
            transfer_cpi_program,
            transfer_cpi_accounts,
            signer_seeds,
        );
        
        anchor_spl::token_interface::transfer_checked(
            transfer_cpi_ctx,
            self.vault.amount,
            self.mint_a.decimals,
        )?;
        
        // Close vault
        let close_cpi_program = self.token_program.to_account_info();
        let close_cpi_accounts = anchor_spl::token_interface::CloseAccount {
            account: self.vault.to_account_info(),
            authority: self.escrow.to_account_info(),
            destination: self.maker.to_account_info(),
        };
        let close_cpi_ctx = CpiContext::new_with_signer(
            close_cpi_program,
            close_cpi_accounts,
            signer_seeds,
        );
        
        anchor_spl::token_interface::close_account(close_cpi_ctx)?;
        
        Ok(())
    }
}

pub fn handler(ctx: Context<Take>) -> Result<()> {
    ctx.accounts.transfer_to_maker()?;
    ctx.accounts.withdraw_and_close_vault()?;
    Ok(())
}