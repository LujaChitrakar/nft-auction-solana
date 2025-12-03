use anchor_lang::prelude::*;
pub mod error;
pub mod event;
pub mod instructions;
pub mod states;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::{transfer as token_transfer, Transfer as TransferToken};
use error::ErrorCode;
use event::*;
use instructions::*;

declare_id!("E6nE6seBRzfDrk1m96fKXKWxYe7JYWSpkFVMj5CLGeP6");

#[program]
pub mod auction {
    use anchor_lang::system_program;

    use crate::states::auction;

    use super::*;

    pub fn create_auction(
        ctx: Context<CreateAuction>,
        starting_price: u64,
        end_time: i64,
        item_mint: Pubkey,
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        let current_time = Clock::get()?.unix_timestamp;
        let owner = ctx.accounts.owner.key();
        auction.seller = owner;
        auction.item_mint = item_mint;
        auction.starting_price = starting_price;
        auction.highest_bid = starting_price;
        auction.highest_bidder = Pubkey::default();
        auction.start_time = current_time;
        auction.end_time = end_time;
        auction.is_open = true;
        auction.bump = ctx.bumps.auction;
        auction.escrow_bump = ctx.bumps.auction_escrow;

        let cpi_program = ctx.accounts.token_program.to_account_info();

        let cpi_ctx_nft = CpiContext::new(
            cpi_program,
            TransferToken {
                to: ctx.accounts.escrow_nft_token_account.to_account_info(),
                from: ctx.accounts.owner_nft_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token_transfer(cpi_ctx_nft, 1)?;

        emit!(AuctionStarted {
            seller: ctx.accounts.owner.key(),
            item_mint: item_mint,
            starting_price: starting_price,
            starting_time: current_time,
            end_time: end_time
        });
        Ok(())
    }






    
    pub fn create_bid(ctx: Context<CreateBid>, bid_amount: u64) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        let highest_bid = auction.highest_bid;
        let auction_key = auction.key();
        let auction_end_time = auction.end_time;
        let current_time = Clock::get()?.unix_timestamp;

        require!(highest_bid < bid_amount, ErrorCode::BidNotHighestBid);
        require!(
            auction_end_time > current_time,
            ErrorCode::AuctionTImeHasPassed
        );
        require!(auction.is_open == true, ErrorCode::AuctionClosed);

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.bidder.to_account_info(),
                to: ctx.accounts.auction_escrow.to_account_info(),
            },
        );
        transfer(cpi_ctx, bid_amount)?;

        if auction.highest_bidder != Pubkey::default() {
            require_keys_eq!(
                ctx.accounts.previous_bidder.key(),
                auction.highest_bidder,
                ErrorCode::PreviousBidderMismatch
            );

            let refund_amount = auction.highest_bid;

            ctx.accounts.auction_escrow.sub_lamports(refund_amount)?;
            ctx.accounts.previous_bidder.add_lamports(refund_amount)?;
            msg!("Refunded to previous bidder");
        }

        auction.highest_bid = bid_amount;
        auction.highest_bidder = ctx.accounts.bidder.key();

        emit!(BidPlaced {
            bidder: ctx.accounts.auction.highest_bidder,
            bid_amount: bid_amount,
            bid_time: current_time
        });
        Ok(())
    }

    pub fn end_auction(ctx: Context<EndAuction>) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        let clock = Clock::get()?.unix_timestamp;
        let auction_key = auction.key();

        require!(
            auction.end_time <= clock,
            ErrorCode::AuctionEndTimeNotReached
        );
        require_keys_eq!(
            ctx.accounts.previous_bidder.key(),
            auction.highest_bidder,
            ErrorCode::PreviousBidderMismatch
        );
        require_keys_eq!(
            ctx.accounts.owner.key(),
            auction.seller,
            ErrorCode::NotOwner
        );

        let seeds_nft: &[&[&[u8]]] =
            &[&[b"auction", ctx.accounts.owner.key.as_ref(), &[auction.bump]]];
        let cpi_program_nft = ctx.accounts.token_program.to_account_info();

        let cpi_ctx_nft = CpiContext::new(
            cpi_program_nft,
            TransferToken {
                from: ctx.accounts.escrow_nft_token_account.to_account_info(),
                to: ctx.accounts.highest_bidder_nft_account.to_account_info(),
                authority: auction.to_account_info(),
            },
        )
        .with_signer(seeds_nft);

        token_transfer(cpi_ctx_nft, 1)?;

        ctx.accounts
            .auction_escrow
            .sub_lamports(auction.highest_bid)?;
        ctx.accounts.owner.add_lamports(auction.highest_bid)?;

        auction.is_open = false;

        emit!(AuctionEnded {
            highest_bidder: auction.highest_bidder,
            highest_bid: auction.highest_bid
        });
        Ok(())
    }
}
