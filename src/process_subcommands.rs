use anyhow::Result;
use metaboss_lib::decode::{
    decode_collection_authority_record, decode_metadata_delegate, decode_token_record,
    decode_use_authority_record,
};
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};

use crate::burn::{
    burn_all, burn_one, burn_print_all, burn_print_one, BurnAllArgs, BurnPrintAllArgs,
};
use crate::collections::{
    approve_delegate, check_collection_items, get_collection_items, migrate_collection,
    revoke_delegate, set_and_verify_nft_collection, set_size, unverify_nft_collection,
    verify_nft_collection, MigrateArgs,
};
use crate::create::{
    create_fungible, create_master_edition, create_metadata, CreateFungibleArgs,
    CreateMasterEditionArgs, CreateMetadataArgs,
};
use crate::decode::{
    decode_edition_marker, decode_master_edition, decode_metadata, decode_metadata_from_mint,
    decode_mint_account, decode_print_edition, decode_token_account,
    process_decode_bpf_loader_upgradable_state,
};
use crate::derive::{
    get_cmv2_pda, get_edition_marker_pda, get_edition_pda, get_generic_pda, get_metadata_pda,
};
use crate::find::find_missing_editions_process;
use crate::mint::{mint_editions, mint_list, mint_missing_editions, mint_one, process_mint_asset};
use crate::opt::*;
use crate::parse::{parse_errors_code, parse_errors_file};
use crate::sign::{sign_all, sign_one};
use crate::snapshot::{
    snapshot_cm_accounts, snapshot_holders, snapshot_indexed_holders, snapshot_indexed_mints,
    snapshot_mints, snapshot_mints_by_collection, snapshot_mints_by_creator, GetMintsArgs, Method,
    NftsByCreatorArgs, SnapshotHoldersArgs, SnapshotMintsArgs,
};
use crate::transfer::process_transfer_asset;
use crate::update::*;
use crate::uses::{approve_use_delegate, revoke_use_delegate, utilize_nft};

pub fn process_uses(client: &RpcClient, commands: UsesSubcommands) -> Result<()> {
    match commands {
        UsesSubcommands::ApproveAuthority {
            keypair,
            delegate_use_authority,
            mint_nft,
            token_account_nft,
            burner_program_id,
            number_of_uses,
        } => approve_use_delegate(
            client,
            keypair,
            mint_nft,
            delegate_use_authority,
            token_account_nft,
            burner_program_id,
            number_of_uses,
        ),

        UsesSubcommands::RevokeAuthority {
            delegate_use_authority,
            keypair,
            mint_nft,
            token_account_nft,
        } => revoke_use_delegate(
            client,
            keypair,
            mint_nft,
            delegate_use_authority,
            token_account_nft,
        ),

        UsesSubcommands::Utilize {
            burner_program_id,
            is_delegate,
            keypair,
            mint_nft,
            holder_nft,
            token_account_nft,
        } => utilize_nft(
            client,
            keypair,
            mint_nft,
            holder_nft,
            token_account_nft,
            burner_program_id,
            is_delegate,
        ),
    }
}

pub async fn process_collections(
    client: RpcClient,
    async_client: AsyncRpcClient,
    commands: CollectionsSubcommands,
) -> Result<()> {
    match commands {
        CollectionsSubcommands::VerifyCollection {
            keypair,
            collection_mint,
            nft_mint,
            is_delegate,
        } => verify_nft_collection(client, keypair, nft_mint, collection_mint, is_delegate),

        CollectionsSubcommands::UnverifyCollection {
            keypair,
            collection_mint,
            is_delegate,
            nft_mint,
        } => unverify_nft_collection(client, keypair, nft_mint, collection_mint, is_delegate),

        CollectionsSubcommands::SetAndVerifyCollection {
            keypair,
            nft_mint,
            update_authority_nft,
            collection_mint,
            is_delegate,
        } => set_and_verify_nft_collection(
            client,
            keypair,
            nft_mint,
            collection_mint,
            update_authority_nft,
            is_delegate,
        ),

        CollectionsSubcommands::ApproveAuthority {
            keypair,
            collection_mint,
            delegate_authority,
        } => approve_delegate(client, keypair, collection_mint, delegate_authority),

        CollectionsSubcommands::RevokeAuthority {
            keypair,
            collection_mint,
            delegate_authority,
        } => revoke_delegate(client, keypair, collection_mint, delegate_authority),

        CollectionsSubcommands::SetSize {
            keypair,
            collection_mint,
            size,
        } => set_size(client, keypair, collection_mint, size),

        CollectionsSubcommands::Migrate {
            keypair,
            mint_address,
            candy_machine_id,
            mint_list,
            cache_file,
            retries,
            batch_size,
            output_file,
        } => {
            migrate_collection(MigrateArgs {
                client,
                async_client,
                keypair,
                mint_address,
                candy_machine_id,
                mint_list,
                cache_file,
                retries,
                batch_size,
                output_file,
            })
            .await
        }
        CollectionsSubcommands::GetItems {
            collection_mint,
            method,
            api_key,
        } => get_collection_items(collection_mint, method, api_key).await,

        CollectionsSubcommands::CheckItems {
            collection_mint,
            item_list,
            debug,
        } => check_collection_items(async_client, collection_mint, item_list, debug).await,
    }
}

pub async fn process_burn(client: RpcClient, commands: BurnSubcommands) -> Result<()> {
    match commands {
        BurnSubcommands::One { keypair, account } => burn_one(client, keypair, account).await,
        BurnSubcommands::All {
            keypair,
            mint_list,
            cache_file,
            batch_size,
            retries,
        } => {
            burn_all(BurnAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
    }
}

pub async fn process_burn_print(client: RpcClient, commands: BurnPrintSubcommands) -> Result<()> {
    match commands {
        BurnPrintSubcommands::One {
            keypair,
            account,
            master_edition,
        } => burn_print_one(client, keypair, account, master_edition).await,
        BurnPrintSubcommands::All {
            keypair,
            mint_list,
            master_mint,
            cache_file,
            batch_size,
            retries,
        } => {
            burn_print_all(BurnPrintAllArgs {
                client,
                keypair,
                mint_list,
                master_mint,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
    }
}

pub fn process_create(client: RpcClient, commands: CreateSubcommands) -> Result<()> {
    match commands {
        CreateSubcommands::Metadata {
            keypair,
            mint,
            metadata,
            immutable,
        } => create_metadata(CreateMetadataArgs {
            client,
            keypair,
            mint,
            metadata,
            immutable,
        }),
        CreateSubcommands::Fungible {
            keypair,
            metadata,
            decimals,
            initial_supply,
            immutable,
        } => create_fungible(CreateFungibleArgs {
            client,
            keypair,
            metadata,
            decimals,
            initial_supply,
            immutable,
        }),
        CreateSubcommands::MasterEdition {
            keypair,
            mint_authority,
            mint,
            max_supply,
        } => create_master_edition(CreateMasterEditionArgs {
            client,
            keypair,
            mint_authority,
            mint,
            max_supply,
        }),
    }
}

pub fn process_decode(client: &RpcClient, commands: DecodeSubcommands) -> Result<()> {
    match commands {
        DecodeSubcommands::BpfUpgradeableState {
            bpf_upgradeable_state_address,
        } => process_decode_bpf_loader_upgradable_state(client, &bpf_upgradeable_state_address)?,
        DecodeSubcommands::Metadata {
            metadata_address,
            output,
        } => {
            decode_metadata(client, metadata_address, &output)?;
        }
        DecodeSubcommands::MintAccount { mint_address } => {
            decode_mint_account(client, &mint_address)?
        }
        DecodeSubcommands::TokenAccount { token_address } => {
            decode_token_account(client, &token_address)?
        }
        DecodeSubcommands::CollectionDelegate { authority_record } => {
            let record = decode_collection_authority_record(client, authority_record)?;

            println!("{record:?}");
        }
        DecodeSubcommands::UseDelegate { use_record } => {
            let record = decode_use_authority_record(client, use_record)?;

            println!("{record:?}");
        }
        DecodeSubcommands::MetadataDelegate {
            metadata_delegate_record,
        } => {
            let record = decode_metadata_delegate(client, metadata_delegate_record)?;

            println!("{record:?}");
        }
        DecodeSubcommands::TokenRecord { token_record } => {
            let record = decode_token_record(client, token_record)?;

            println!("{record:?}");
        }
        DecodeSubcommands::Mint {
            account,
            full,
            list_file,
            raw,
            ref output,
        } => decode_metadata_from_mint(
            client,
            account.as_ref(),
            full,
            list_file.as_ref(),
            raw,
            output,
        )?,
        DecodeSubcommands::Master { account } => decode_master_edition(client, &account)?,
        DecodeSubcommands::Edition { account } => decode_print_edition(client, &account)?,
        DecodeSubcommands::EditionMarker {
            account,
            edition_num,
            marker_num,
        } => decode_edition_marker(client, &account, edition_num, marker_num)?,
    }
    Ok(())
}

pub fn process_derive(commands: DeriveSubcommands) {
    match commands {
        DeriveSubcommands::Pda { seeds, program_id } => get_generic_pda(seeds, program_id),
        DeriveSubcommands::Metadata { mint_account } => get_metadata_pda(mint_account),
        DeriveSubcommands::Edition { mint_account } => get_edition_pda(mint_account),
        DeriveSubcommands::EditionMarker {
            mint_account,
            edition_num,
        } => get_edition_marker_pda(mint_account, edition_num),
        DeriveSubcommands::CMV2Creator { candy_machine_id } => get_cmv2_pda(candy_machine_id),
    }
}

pub fn process_find(client: &RpcClient, commands: FindSubcommands) -> Result<()> {
    match commands {
        FindSubcommands::MissingEditions { account } => {
            find_missing_editions_process(client, &account)
        }
        FindSubcommands::Error { error_code } => parse_errors_code(&error_code),
    }
}

pub fn process_mint(client: &RpcClient, commands: MintSubcommands) -> Result<()> {
    match commands {
        MintSubcommands::Asset {
            keypair,
            receiver,
            asset_data,
            amount,
            decimals,
            max_print_edition_supply,
        } => process_mint_asset(
            client,
            keypair,
            receiver,
            asset_data,
            decimals,
            amount,
            max_print_edition_supply,
        ),
        MintSubcommands::One {
            keypair,
            receiver,
            nft_data_file,
            external_metadata_uri,
            immutable,
            primary_sale_happened,
            max_editions,
            sign,
            sized,
        } => mint_one(
            client,
            keypair,
            &receiver,
            nft_data_file,
            external_metadata_uri.as_ref(),
            immutable,
            primary_sale_happened,
            max_editions,
            sign,
            sized,
        )
        .map(|_| ()),
        MintSubcommands::Editions {
            keypair,
            account,
            receiver,
            next_editions,
            specific_editions,
        } => mint_editions(
            client,
            keypair,
            account,
            &receiver,
            next_editions,
            specific_editions,
        ),
        MintSubcommands::MissingEditions { keypair, account } => {
            mint_missing_editions(client, &keypair, &account)
        }
        MintSubcommands::List {
            keypair,
            receiver,
            nft_data_dir,
            external_metadata_uris,
            immutable,
            primary_sale_happened,
            sign,
            track,
        } => mint_list(
            client,
            keypair,
            receiver,
            nft_data_dir,
            external_metadata_uris,
            immutable,
            primary_sale_happened,
            sign,
            track,
        ),
    }
}

pub async fn process_set(client: RpcClient, commands: SetSubcommands) -> Result<()> {
    match commands {
        SetSubcommands::PrimarySaleHappened { keypair, account } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);

            let args = SetPrimarySaleHappenedArgs {
                client: Arc::new(client),
                keypair: Arc::new(keypair),
                mint_account: account,
            };

            let sig = set_primary_sale_happened(args)
                .await
                .map_err(Into::<ActionError>::into)?;
            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        SetSubcommands::PrimarySaleHappenedAll {
            keypair,
            mint_list,
            cache_file,
            batch_size,
            retries,
        } => {
            set_primary_sale_happened_all(SetPrimarySaleHappenedAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
        SetSubcommands::UpdateAuthority {
            keypair,
            account,
            new_update_authority,
            keypair_payer,
        } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);
            let solana_opts = parse_solana_config();
            let payer = keypair_payer.map(|path| parse_keypair(Some(path), solana_opts));

            let args = SetUpdateAuthorityArgs {
                client: Arc::new(client),
                keypair: Arc::new(keypair),
                payer: Arc::new(payer),
                mint_account: account,
                new_authority: new_update_authority,
            };

            let sig = set_update_authority(args)
                .await
                .map_err(Into::<ActionError>::into)?;
            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        SetSubcommands::UpdateAuthorityAll {
            keypair,
            payer,
            mint_list,
            new_authority,
            cache_file,
            batch_size,
            retries,
        } => {
            set_update_authority_all(SetUpdateAuthorityAllArgs {
                client,
                keypair,
                payer,
                mint_list,
                new_authority,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
        SetSubcommands::Immutable { keypair, account } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);

            let args = SetImmutableArgs {
                client: Arc::new(client),
                keypair: Arc::new(keypair),
                mint_account: account,
            };

            let sig = set_immutable(args)
                .await
                .map_err(Into::<ActionError>::into)?;
            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        SetSubcommands::ImmutableAll {
            keypair,
            mint_list,
            cache_file,
            batch_size,
            retries,
        } => {
            set_immutable_all(SetImmutableAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
        SetSubcommands::TokenStandard { keypair, account } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);

            let args = SetTokenStandardArgs {
                client: Arc::new(client),
                keypair: Arc::new(keypair),
                mint_account: account,
            };

            let sig = set_token_standard_one(args)
                .await
                .map_err(Into::<ActionError>::into)?;
            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        SetSubcommands::TokenStandardAll {
            keypair,
            mint_list,
            cache_file,
            batch_size,
            retries,
        } => {
            set_token_standard_all(SetTokenStandardAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
    }
}

pub fn process_sign(client: &RpcClient, commands: SignSubcommands) -> Result<()> {
    match commands {
        SignSubcommands::One { keypair, account } => sign_one(client, keypair, account),
        SignSubcommands::All {
            keypair,
            creator,
            position,
            v2,
            v3,
            mint_accounts_file,
        } => sign_all(
            client,
            keypair,
            &creator,
            position,
            v2,
            v3,
            mint_accounts_file,
        ),
    }
}

pub async fn process_snapshot(client: &RpcClient, commands: SnapshotSubcommands) -> Result<()> {
    match commands {
        SnapshotSubcommands::Holders {
            update_authority,
            creator,
            position,
            mint_accounts_file,
            v2,
            v3,
            allow_unverified,
            output,
        } => snapshot_holders(
            client,
            SnapshotHoldersArgs {
                update_authority,
                creator,
                position,
                mint_accounts_file,
                v2,
                v3,
                allow_unverified,
                output,
            },
        ),
        SnapshotSubcommands::IndexedHolders {
            indexer,
            api_key,
            creator,
            output,
        } => {
            snapshot_indexed_holders(NftsByCreatorArgs {
                creator,
                api_key,
                indexer,
                output,
            })
            .await
        }
        SnapshotSubcommands::CMAccounts {
            update_authority,
            output,
        } => snapshot_cm_accounts(client, &update_authority, &output),
        SnapshotSubcommands::Mints {
            creator,
            position,
            update_authority,
            v2,
            v3,
            allow_unverified,
            output,
        } => snapshot_mints(
            client,
            SnapshotMintsArgs {
                creator,
                position,
                update_authority,
                v2,
                v3,
                allow_unverified,
                output,
            },
        ),
        SnapshotSubcommands::IndexedMints {
            indexer,
            api_key,
            creator,
            output,
        } => {
            snapshot_indexed_mints(NftsByCreatorArgs {
                creator,
                api_key,
                indexer,
                output,
            })
            .await
        }
        SnapshotSubcommands::MintsByCreator {
            indexer,
            api_key,
            address,
            output,
        } => {
            snapshot_mints_by_creator(GetMintsArgs {
                indexer,
                api_key,
                method: Method::Creator,
                address,
                output,
            })
            .await
        }
        SnapshotSubcommands::MintsByCollection {
            indexer,
            api_key,
            address,
            output,
        } => {
            snapshot_mints_by_collection(GetMintsArgs {
                indexer,
                api_key,
                method: Method::Collection,
                address,
                output,
            })
            .await
        }
    }
}

pub fn process_transfer(client: RpcClient, commands: TransferSubcommands) -> Result<()> {
    match commands {
        TransferSubcommands::Asset {
            keypair,
            receiver,
            mint,
            amount,
        } => process_transfer_asset(&client, keypair, receiver, mint, amount),
    }
}

pub async fn process_update(client: RpcClient, commands: UpdateSubcommands) -> Result<()> {
    match commands {
        UpdateSubcommands::RuleSet {
            keypair,
            mint,
            new_rule_set,
        } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);

            let args = UpdateRuleSetArgs {
                client: Arc::new(client),
                keypair: Arc::new(keypair),
                mint_account: mint,
                new_rule_set,
            };

            let sig = update_rule_set(args)
                .await
                .map_err(Into::<ActionError>::into)?;
            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        UpdateSubcommands::RuleSetAll {
            keypair,
            mint_list,
            cache_file,
            new_rule_set,
            batch_size,
            retries,
        } => {
            update_rule_set_all(UpdateRuleSetAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                new_rule_set,
                batch_size,
                retries,
            })
            .await
        }
        UpdateSubcommands::ClearRuleSet { keypair, mint } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);

            let args = ClearRuleSetArgs {
                client: Arc::new(client),
                keypair: Arc::new(keypair),
                mint_account: mint,
            };

            let sig = clear_rule_set(args)
                .await
                .map_err(Into::<ActionError>::into)?;

            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        UpdateSubcommands::ClearRuleSetAll {
            keypair,
            mint_list,
            cache_file,
            batch_size,
            retries,
        } => {
            clear_rule_set_all(ClearRuleSetAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
        UpdateSubcommands::SellerFeeBasisPoints {
            keypair,
            account,
            new_sfbp,
        } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);

            let args = UpdateSellerFeeBasisPointsArgs {
                client: Arc::new(client),
                keypair: Arc::new(keypair),
                mint_account: account,
                new_sfbp,
            };

            let sig = update_sfbp(args).await.map_err(Into::<ActionError>::into)?;
            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        UpdateSubcommands::SellerFeeBasisPointsAll {
            keypair,
            mint_list,
            cache_file,
            new_sfbp,
            batch_size,
            retries,
        } => {
            update_sfbp_all(UpdateSellerFeeBasisPointsAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                new_sfbp,
                batch_size,
                retries,
            })
            .await
        }
        UpdateSubcommands::Name {
            keypair,
            account,
            new_name,
        } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);

            let args = UpdateNameArgs {
                client: Arc::new(client),
                keypair: Arc::new(keypair),
                mint_account: account,
                new_name,
            };

            let sig = update_name(args).await.map_err(Into::<ActionError>::into)?;
            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        UpdateSubcommands::Symbol {
            keypair,
            account,
            new_symbol,
        } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);

            let args = UpdateSymbolArgs {
                client: Arc::new(client),
                keypair: Arc::new(keypair),
                mint_account: account,
                new_symbol,
            };

            let sig = update_symbol(args)
                .await
                .map_err(Into::<ActionError>::into)?;
            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        UpdateSubcommands::SymbolAll {
            keypair,
            mint_list,
            cache_file,
            new_symbol,
            batch_size,
            retries,
        } => {
            update_symbol_all(UpdateSymbolAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                new_symbol,
                batch_size,
                retries,
            })
            .await
        }
        UpdateSubcommands::Creators {
            keypair,
            account,
            new_creators,
            append,
        } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);

            let args = UpdateCreatorArgs {
                client: Arc::new(client),
                keypair: Arc::new(keypair),
                mint_account: account,
                new_creators,
                should_append: append,
            };

            let sig = update_creator(args)
                .await
                .map_err(Into::<ActionError>::into)?;
            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        UpdateSubcommands::CreatorsAll {
            keypair,
            mint_list,
            cache_file,
            new_creators,
            append,
            batch_size,
            retries,
        } => {
            update_creator_all(UpdateCreatorAllArgs {
                client,
                keypair_path: keypair,
                mint_list,
                cache_file,
                new_creators,
                should_append: append,
                batch_size,
                retries,
            })
            .await
        }
        UpdateSubcommands::Data {
            keypair,
            account,
            new_data_file,
        } => update_data_one(&client, keypair, &account, &new_data_file),
        UpdateSubcommands::DataAll { keypair, data_dir } => {
            update_data_all(&client, keypair, &data_dir)
        }
        UpdateSubcommands::Uri {
            keypair,
            account,
            new_uri,
        } => {
            let solana_opts = parse_solana_config();
            let keypair = parse_keypair(keypair, solana_opts);

            let args = UpdateUriArgs {
                client: &client,
                keypair: &keypair,
                payer: None,
                mint_account: account,
                new_uri,
            };

            let sig = update_uri(args).map_err(Into::<ActionError>::into)?;
            info!("Tx sig: {:?}", sig);
            println!("Tx sig: {sig:?}");

            Ok(())
        }
        UpdateSubcommands::UriAll { keypair, json_file } => {
            update_uri_all(&client, keypair, &json_file)
        }
        UpdateSubcommands::Uses {
            keypair,
            account,
            method,
            remaining,
            total,
            overwrite,
        } => update_uses_one(UsesArgs {
            client,
            keypair,
            account,
            method,
            remaining,
            total,
            overwrite,
        }),
    }
}

pub fn process_parse_errors_file(commands: ParseErrorsSubCommands) -> Result<()> {
    match commands {
        ParseErrorsSubCommands::File => parse_errors_file(),
    }
}
