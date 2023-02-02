script;

use std::tx::{tx_script_bytecode_hash};
use std::inputs::{input_predicate_data,input_type, Input,input_count, input_owner};
use std::outputs::{output_count, output_type, Output};
use std::logging::{log};
use std::{context::*, token::*};

pub const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
pub const GTF_INPUT_COIN_AMOUNT = 0x105;

const GTF_OUTPUT_COIN_TO = 0x202;
const GTF_OUTPUT_COIN_ASSET_ID = 0x204;

pub fn input_amount(index: u64) -> Option<u64> {
    match input_type(index) {
        Input::Coin => Option::Some(__gtf::<u64>(index, GTF_INPUT_COIN_AMOUNT)),
        Input::Message => Option::Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_AMOUNT)),
        Input::Contract => Option::None,
    }
}

fn main(){
    match output_type(2) {
        Output::Coin => (),
        _ => revert(0),
    };
    match output_type(3) {
        Output::Coin => (),
        _ => revert(0),
    };
    let to0 = Address::from(__gtf::<b256>(2, GTF_OUTPUT_COIN_TO));
    let to1 = Address::from(__gtf::<b256>(3, GTF_OUTPUT_COIN_TO));

    assert(input_owner(0).unwrap() == to0);
    assert(input_owner(1).unwrap() == to1);
    
    let amount = input_amount(0);
    log(amount.unwrap());
    let input_count1: u8 = input_count();
    let output_count1: u64 = output_count();
    let pr = input_owner(1).unwrap();

    log(pr);
    log(input_owner(0).unwrap());
    log(input_count1);
    log(output_count1);
   
    }
