use crate::states::Auction;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct CreateAuction<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer=owner,
        space=139,
        seeds=[b"auction",owner.key().as_ref()],
        bump,
    )]
    pub auction: Account<'info, Auction>,


    
    ///CHECK: ONLY HOLDS SOL too
    #[account
    (
        init,
        payer=owner,
        space=0,
        seeds=[b"auction_escrow",auction.key().as_ref()],bump
    )]
    pub auction_escrow: AccountInfo<'info>,

    #[account(
        mut,
        constraint=owner_nft_account.owner==owner.key()
    )]
    pub owner_nft_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer=owner,
        token::mint=nft_mint,
        token::authority=auction,
        seeds=[b"escrow_nft",auction.key().as_ref()],
        bump,
    )]
    pub escrow_nft_token_account: Account<'info, TokenAccount>,

    pub nft_mint: Account<'info, Mint>,

    pub rent: Sysvar<'info, Rent>,
    
    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}
