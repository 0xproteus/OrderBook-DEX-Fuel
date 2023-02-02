use fuels::prelude::*;
use fuels::signers::fuel_crypto::coins_bip32::ecdsa::digest::typenum::Log2;
use fuels::test_helpers::WalletsConfig;
use fuels::tx::{AssetId, Input, Output, TxPointer};
use fuels::types::resource::Resource;
use std::ops::Add;
use std::str::FromStr;
use fuels::core::abi_encoder::ABIEncoder;
use fuels::types::Token;

// Load abi from json

 
const BASE_ASSET: AssetId = AssetId::new([0u8; 32]);
const A_TOKEN_ID: AssetId = AssetId::new([1u8; 32]);
const B_TOKEN_ID: AssetId = AssetId::new([2u8; 32]);

const AB_ASK_AMOUNT: u64 = 500;
const AB_OFFER_AMOUNT: u64 = 250;

const BA_ASK_AMOUNT: u64 = 250;
const BA_OFFER_AMOUNT: u64 = 500;

async fn get_balance(provider: &Provider, address: &Bech32Address, asset: AssetId) -> u64 {
    provider.get_asset_balance(address, asset).await.unwrap()
}

pub fn configure_wallets() -> WalletsConfig {
    let assets = [BASE_ASSET, A_TOKEN_ID, B_TOKEN_ID];

    WalletsConfig::new_multiple_assets(
        3,
        assets
            .map(|asset| AssetConfig {
                id: asset,
                num_coins: 1,
                coin_amount: 1000,
            })
            .to_vec(),
    )
}

#[tokio::test]
async fn can_make_swap() {
    
    abigen!(Script(name="MatchScript", abi="/mnt/c/Users/Proteus/Documents/OrderBook-DEX-Fuel/project/scripts/AB-BA-match/out/debug/AB-BA-match-abi.json"),
    Predicate(name="PredicateAB", abi="/mnt/c/Users/Proteus/Documents/OrderBook-DEX-Fuel/project/predicates/AB-pair/out/debug/AB-pair-abi.json"),
    Predicate(name="PredicateBA", abi="/mnt/c/Users/Proteus/Documents/OrderBook-DEX-Fuel/project/predicates/BA-pair/out/debug/BA-pair-abi.json"));

    //launch local chain and generate wallets
    let wallets = launch_custom_provider_and_get_wallets(configure_wallets(), None, None).await;

    let ab_predicate_owner_wallet  = &wallets[0];
    let ba_predicate_owner_wallet  = &wallets[1];
    
    let provider = ab_predicate_owner_wallet.get_provider().unwrap();

    let script_bin_path =
    "/mnt/c/Users/Proteus/Documents/OrderBook-DEX-Fuel/project/scripts/AB-BA-match/out/debug/AB-BA-match.bin";

    let predicate_ab_bin_path =  "/mnt/c/Users/Proteus/Documents/OrderBook-DEX-Fuel/project/predicates/AB-pair/out/debug/AB-pair.bin";
    let precicate_ba_bin_path = "/mnt/c/Users/Proteus/Documents/OrderBook-DEX-Fuel/project/predicates/BA-pair/out/debug/BA-pair.bin";

    //get the predicates and script
    let predicate_ab = PredicateAB::load_from( predicate_ab_bin_path,).unwrap();
    let predicate_ba = PredicateBA::load_from( precicate_ba_bin_path,).unwrap();
    let script = MatchScript::new(wallets[2].clone(), script_bin_path);

    //deposit the tokens wished to trade on the predicates
    predicate_ab.receive(ab_predicate_owner_wallet, AB_OFFER_AMOUNT, B_TOKEN_ID, None).await.unwrap();

    predicate_ba.receive(ba_predicate_owner_wallet, BA_OFFER_AMOUNT, A_TOKEN_ID, None).await.unwrap();

    let inital_predicate0_balance = get_balance(provider, predicate_ab.address(), B_TOKEN_ID).await;
    let inital_predicate1_balance = get_balance(provider, predicate_ba.address(), A_TOKEN_ID).await;
    println!("{:?}",inital_predicate0_balance);
    println!("{:?}",inital_predicate1_balance);


    //Prepare to make the match, define the transaction Inputs and Outputs
    let predicate_ab_coin = &provider
        .get_spendable_resources(predicate_ab.address(), B_TOKEN_ID, 1).await.unwrap()[0];
        
    let predicate_ab_coin_utxo_id = match predicate_ab_coin {
        Resource::Coin(coin) => coin.utxo_id,
        _ => panic!(),
    };
    
     let input_predicate_ab = Input::CoinPredicate {
        utxo_id: predicate_ab_coin_utxo_id,
        tx_pointer: TxPointer::default(),
        owner: predicate_ab.address().into(),
        amount: AB_OFFER_AMOUNT,
        asset_id: B_TOKEN_ID,
        maturity: 0,
        predicate: predicate_ab.code(),
        predicate_data: vec![],
    };
   
    let predicate_ba_coin = &provider
    .get_spendable_resources(predicate_ba.address(), A_TOKEN_ID, 1).await.unwrap()[0];

    let predicate_ba_coin_utxo_id = match predicate_ba_coin {
        Resource::Coin(coin) => coin.utxo_id,
        _ => panic!(),
    };

    let input_predicate_ba = Input::CoinPredicate {
        utxo_id: predicate_ba_coin_utxo_id,
        tx_pointer: TxPointer::default(),
        owner: predicate_ba.address().into(),
        amount: BA_OFFER_AMOUNT,
        asset_id: A_TOKEN_ID,
        maturity: 0,
        predicate: predicate_ba.code(),
        predicate_data: vec![],
    };

    let output_to_ab_receiver = Output::Coin {
        to: Address::from(ab_predicate_owner_wallet.address()),
        amount: AB_ASK_AMOUNT,
        asset_id: A_TOKEN_ID,
    };

    // Output for the offered coin transferred from the predicate to the order taker
    let output_to_ba_receiver = Output::Coin {
        to: Address::from(ba_predicate_owner_wallet.address()),
        amount: BA_ASK_AMOUNT,
        asset_id: B_TOKEN_ID,
    };

    let output_to_predicate_ab = Output::Coin {
        to: Address::from(predicate_ab.address()),
        amount: 0,
        asset_id: B_TOKEN_ID,
    };

    let output_to_predicate_ba = Output::Coin {
        to: Address::from(predicate_ba.address()),
        amount: 0,
        asset_id: A_TOKEN_ID,
    };


    let inputs = vec![input_predicate_ab, input_predicate_ba];
    let outputs = vec![output_to_ab_receiver, output_to_ba_receiver, output_to_predicate_ab , output_to_predicate_ba];

    //Perform the transaction
    let result = script.main().with_inputs(inputs).with_outputs(outputs)
    .tx_params( TxParameters::new(None, Some(10_000_000), None)).call().await.unwrap();

}

#[tokio::test]
async fn can_cancel() {
    abigen!(Script(name="MatchScript", abi="/mnt/c/Users/Proteus/Documents/OrderBook-DEX-Fuel/project/scripts/AB-BA-match/out/debug/AB-BA-match-abi.json"),
    Predicate(name="PredicateAB", abi="/mnt/c/Users/Proteus/Documents/OrderBook-DEX-Fuel/project/predicates/AB-pair/out/debug/AB-pair-abi.json"),
    Predicate(name="PredicateBA", abi="/mnt/c/Users/Proteus/Documents/OrderBook-DEX-Fuel/project/predicates/BA-pair/out/debug/BA-pair-abi.json"));

    let wallets = launch_custom_provider_and_get_wallets(configure_wallets(), None, None).await;

    let ab_predicate_owner_wallet  = &wallets[0];
    let ba_predicate_owner_wallet  = &wallets[1];
    
    let provider = ab_predicate_owner_wallet.get_provider().unwrap();

    let predicate_ab_bin_path =  "/mnt/c/Users/Proteus/Documents/OrderBook-DEX-Fuel/project/predicates/AB-pair/out/debug/AB-pair.bin";

    let predicate_ab = PredicateAB::load_from( predicate_ab_bin_path,).unwrap();
    
    let initial_wallet_balance =  get_balance(provider, ab_predicate_owner_wallet.address(), B_TOKEN_ID).await;

    predicate_ab.receive(ab_predicate_owner_wallet, AB_OFFER_AMOUNT, B_TOKEN_ID, None).await.unwrap();

    let predicate_ab_coin = &provider
        .get_spendable_resources(predicate_ab.address(), B_TOKEN_ID, 1).await.unwrap()[0];
        
    let predicate_ab_coin_utxo_id = match predicate_ab_coin {
        Resource::Coin(coin) => coin.utxo_id,
        _ => panic!(),
    };
     // Offered asset coin belonging to the predicate root
     let input_predicate_ab = Input::CoinPredicate {
        utxo_id: predicate_ab_coin_utxo_id,
        tx_pointer: TxPointer::default(),
        owner: predicate_ab.address().into(),
        amount: AB_OFFER_AMOUNT,
        asset_id: B_TOKEN_ID,
        maturity: 0,
        predicate: predicate_ab.code(),
        predicate_data: vec![],
    };

    let base_coin = &provider
    .get_spendable_resources(ab_predicate_owner_wallet.address(), BASE_ASSET, 1)
    .await
    .unwrap()[0];
    let (base_coin_utxo_id, swap_coin_amount) = match base_coin {
        Resource::Coin(coin) => (coin.utxo_id, coin.amount),
        _ => panic!(),
    };

    let input_coin = Input::CoinSigned {
        utxo_id: base_coin_utxo_id,
        tx_pointer: TxPointer::default(),
        owner: Address::from(ab_predicate_owner_wallet.address()),
        amount: swap_coin_amount,
        asset_id: BASE_ASSET,
        witness_index: 0,
        maturity: 0,
    };

    let output_to_taker = Output::Coin {
        to: Address::from(ab_predicate_owner_wallet.address()),
        amount: AB_OFFER_AMOUNT,
        asset_id: B_TOKEN_ID,
    };


    let mut tx = Wallet::build_transfer_tx(
        &[input_predicate_ab, input_coin],
        &[output_to_taker],
        TxParameters::new(None, Some(10_000_000), None),
    );

    ab_predicate_owner_wallet.sign_transaction(&mut tx).await.unwrap();
    let _receipts = provider.send_transaction(&tx).await.unwrap();

    let predicate_balance = get_balance(provider, predicate_ab.address(), B_TOKEN_ID).await;
    assert_eq!(predicate_balance, 0);

    // Wallet balance is the same as before it sent the coins to the predicate
    let wallet_balance = get_balance(provider, ab_predicate_owner_wallet.address(), B_TOKEN_ID).await;
    assert_eq!(wallet_balance, initial_wallet_balance);
}