predicate;

use std::tx::{tx_script_bytecode_hash};
use std::{
    inputs::{
        input_count,
        input_owner,
        input_type,
        Input,
    },
    outputs::{
        Output,
        output_amount,
        output_pointer,
        output_type,
    },
};
use std::inputs::{input_predicate_data};

const GTF_OUTPUT_COIN_TO = 0x202;
const GTF_OUTPUT_COIN_ASSET_ID = 0x204;
pub const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
pub const GTF_INPUT_COIN_AMOUNT = 0x105;

pub fn input_amount(index: u64) -> Option<u64> {
    match input_type(index) {
        Input::Coin => Option::Some(__gtf::<u64>(index, GTF_INPUT_COIN_AMOUNT)),
        Input::Message => Option::Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_AMOUNT)),
        Input::Contract => Option::None,
    }
}

const RECEIVER_OUTPUT_INDEX = 1;
const SELF_OUTPUT_INDEX = 0;
const RETURN_OUTPUT_INDEX = 3;

const SELF_INPUT_INDEX = 1;

fn main() -> bool {
      
    
    const ASK_TOKEN = ContractId {
        value: B_TOKEN_CONFIG,
    };
    const OFFER_TOKEN = ContractId {
        value: A_TOKEN_CONFIG,
    };
    const RECEIVER = Address::from(RECEIVER_CONFIG);

    //Let Reiceiver cancel order by sending a coin input
    if input_count() == 2u8 { 
        if input_owner(0).unwrap() == RECEIVER
            || input_owner(1).unwrap() == RECEIVER
        {
            return true;
        };
    };

    //assert (0xfa73d2eaa346a25b4dca67b471d49cf339e3e79498625e5b9d78b5c1b8775c5c  == tx_script_bytecode_hash() );
    
     // Revert if output is not an Output::Coin
    match output_type(RECEIVER_OUTPUT_INDEX) {
        Output::Coin => (),
        _ => revert(0),
    };

    match output_type(RETURN_OUTPUT_INDEX) {
        Output::Coin => (),
        _ => revert(0),
    };

    // Since output is known to be a Coin, the following are always valid
    let to = Address::from(__gtf::<b256>(RECEIVER_OUTPUT_INDEX, GTF_OUTPUT_COIN_TO));
    let asset_id_to_receiver = ContractId::from(__gtf::<b256>(RECEIVER_OUTPUT_INDEX, GTF_OUTPUT_COIN_ASSET_ID));

    let asset_id_returned = ContractId::from(__gtf::<b256>(RETURN_OUTPUT_INDEX, GTF_OUTPUT_COIN_ASSET_ID));

    let self_input_amount = input_amount(SELF_INPUT_INDEX).unwrap();

    let amount_to_receiver = output_amount(RECEIVER_OUTPUT_INDEX);

    let amount_to_taker = output_amount(SELF_OUTPUT_INDEX);

    let amount_returned = output_amount(RETURN_OUTPUT_INDEX);

    let ratio_asked = ASK_AMOUNT * 1000 / OFFER_AMOUNT;

    let ratio_received = amount_to_receiver * 1000 / amount_to_taker;

    (to == RECEIVER) && (ratio_received >= ratio_asked) && (asset_id_to_receiver == ASK_TOKEN) && (asset_id_returned == OFFER_TOKEN) && (self_input_amount == amount_returned + amount_to_taker)
}