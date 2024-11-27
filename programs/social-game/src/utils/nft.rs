use {
    anchor_lang::{ prelude::*, system_program },
    anchor_spl::{
        associated_token::{ self, AssociatedToken },
        token_2022,
        token_interface::{ Token2022 },
    },
};
use solana_program::program::{ invoke, invoke_signed };
use spl_token_2022::{ extension::ExtensionType, state::Mint };

use crate::model::NftAuthority;
use crate::model::ProgramErrorCode;

pub fn mint<'info>(
    meta_data_space: usize,
    name: String,
    symbol: String,
    uri: String,
    signer: &[&[&[u8]]],
    token_program: &Program<'info, Token2022>,
    payer: &Signer<'info>,
    mint_account: &Signer<'info>,
    associated_token_program: &Program<'info, AssociatedToken>,
    associated_token_account: &AccountInfo<'info>,
    system_program: &Program<'info, System>,
    nft_authority: &Account<'info, NftAuthority>,
) -> Result<()> {
    let space = match
        ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::MetadataPointer])
    {
        Ok(space) => space,
        Err(_) => {
            return err!(ProgramErrorCode::InvalidMintAccountSpace);
        }
    };

    let lamports_required = Rent::get()?.minimum_balance(space + meta_data_space);

    system_program::create_account(
        CpiContext::new(
            token_program.to_account_info(),
            system_program::CreateAccount {
                from: payer.to_account_info(),
                to: mint_account.to_account_info(),
            }
        ),
        lamports_required,
        space as u64,
        &token_program.key()
    )?;

    // Assign the mint to the token program
    system_program::assign(
        CpiContext::new(token_program.to_account_info(), system_program::Assign {
            account_to_assign: mint_account.to_account_info(),
        }),
        &token_2022::ID
    )?;

    // Initialize the metadata pointer (Need to do this before initializing the mint)
    let init_meta_data_pointer_ix = match
        spl_token_2022::extension::metadata_pointer::instruction::initialize(
            &Token2022::id(),
            &mint_account.key(),
            Some(nft_authority.key()),
            Some(mint_account.key())
        )
    {
        Ok(ix) => ix,
        Err(_) => {
            return err!(ProgramErrorCode::CantInitializeMetadataPointer);
        }
    };

    invoke(
        &init_meta_data_pointer_ix,
        &[mint_account.to_account_info(), nft_authority.to_account_info()]
    )?;

    // Initialize the mint cpi
    let mint_cpi_ix = CpiContext::new(
        token_program.to_account_info(),
        token_2022::InitializeMint2 {
            mint: mint_account.to_account_info(),
        }
    );

    token_2022::initialize_mint2(mint_cpi_ix, 0, &nft_authority.key(), None).unwrap();

    // Init the metadata account
    let init_token_meta_data_ix = &spl_token_metadata_interface::instruction::initialize(
        &spl_token_2022::id(),
        mint_account.key,
        nft_authority.to_account_info().key,
        mint_account.key,
        nft_authority.to_account_info().key,
        name,
        symbol,
        uri
    );

    invoke_signed(
        init_token_meta_data_ix,
        &[
            mint_account.to_account_info().clone(),
            nft_authority.to_account_info().clone(),
        ],
        signer
    )?;

    // Create the associated token account
    associated_token::create(
        CpiContext::new(
            associated_token_program.to_account_info(),
            associated_token::Create {
                payer: payer.to_account_info(),
                associated_token: associated_token_account.to_account_info(),
                authority: payer.to_account_info(),
                mint: mint_account.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            }
        )
    )?;

    // Mint one token to the associated token account of the player
    token_2022::mint_to(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            token_2022::MintTo {
                mint: mint_account.to_account_info(),
                to: associated_token_account.to_account_info(),
                authority: nft_authority.to_account_info(),
            },
            signer
        ),
        1
    )?;

    Ok(())
}

pub fn update_field<'info>(
    key: String,
    value: String,
    mint_account: &Signer<'info>,
    nft_authority: &Account<'info, NftAuthority>,
    signer: &[&[&[u8]]]
) -> Result<()> {

    invoke_signed(
        &spl_token_metadata_interface::instruction::update_field(
            &spl_token_2022::id(),
            mint_account.key,
            nft_authority.to_account_info().key,
            spl_token_metadata_interface::state::Field::Key(key),
            value
        ),
        &[
            mint_account.to_account_info().clone(),
            nft_authority.to_account_info().clone(),
        ],
        signer
    )?;

    Ok(())
}