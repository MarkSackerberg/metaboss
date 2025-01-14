use super::*;

pub struct SetPrimarySaleHappenedAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub batch_size: usize,
    pub retries: u8,
}

pub struct SetPrimarySaleHappenedArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
}

pub async fn set_primary_sale_happened(
    args: SetPrimarySaleHappenedArgs,
) -> Result<Signature, ActionError> {
    let (_, token, current_rule_set) = update_asset_preface(&args.client, &args.mint_account)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    let UpdateArgs::V1 {
        ref mut primary_sale_happened,
        ..
    } = update_args;
    *primary_sale_happened = Some(true);

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &args.keypair,
        mint: args.mint_account.clone(),
        token,
        delegate_record: None::<String>, // Not supported yet in update.
        current_rule_set,
        update_args,
    };

    update_asset(&args.client, update_args)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}

pub struct SetPrimarySaleHappenedAll {}

#[async_trait]
impl Action for SetPrimarySaleHappenedAll {
    fn name() -> &'static str {
        "set-secondary-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        set_primary_sale_happened(SetPrimarySaleHappenedArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
        })
        .await
        .map(|_| ())
    }
}

pub async fn set_primary_sale_happened_all(args: SetPrimarySaleHappenedAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    // We don't support an optional payer for this action currently.
    let payer = None;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list: args.mint_list,
        cache_file: args.cache_file,
        new_value: "".to_string(),
        batch_size: args.batch_size,
        retries: args.retries,
    };
    SetPrimarySaleHappenedAll::run(args).await
}
