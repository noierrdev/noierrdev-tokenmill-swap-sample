use anyhow::{Context, Result};
use clap::Parser;

use std::{
    env,
    time::{  SystemTime, UNIX_EPOCH},
};
use tokio::time::{Duration};

use solana_sdk::{
    bs58,
    signature::{Keypair,Signature,Signer},
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
    message::Message,
    transaction::Transaction,
    hash::Hash,
    commitment_config::CommitmentConfig,
    system_instruction,
    compute_budget::{ComputeBudgetInstruction}
};

use solana_client::{
    rpc_client::RpcClient,
    tpu_client::{TpuClient, TpuClientConfig},
    rpc_response::RpcContactInfo
};

use spl_associated_token_account::{get_associated_token_address};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::{
    instruction::close_account,
    instruction::transfer,
    instruction::sync_native,
    id as token_program_id, // Import the token program ID
};

use base64::{engine::general_purpose, Engine as _};

use reqwest::Client;
use serde_json::json;
use serde_json::Value;

use borsh::BorshDeserialize;
use borsh::BorshSerialize;


use std::convert::TryInto;

mod math;
mod swap_math;
mod quote;
mod market;

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SwapParameters {
    BuyExactIn(u64, u64),
    BuyExactOut(u64, u64),
    SellExactIn(u64, u64),
    SellExactOut(u64, u64),
}


#[tokio::main]
async fn main() {
    
    dotenv::dotenv().ok();

    let sol_mint="So11111111111111111111111111111111111111112";

    let jito_tip_accounts = ["ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt","3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT","HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe","DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL","Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY","DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh","ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49","96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5"];

    //Initialize wallet from private key of .env
    let private_key_str = env::var("PRIVATE_KEY").unwrap();
    let private_key_bytes = bs58::decode(private_key_str)
        .into_vec().unwrap();
    let wallet = Keypair::from_bytes(&private_key_bytes).unwrap();
    let public_key= wallet.pubkey();
    println!("Public Key: {}", public_key.to_string());



    //Create web3 connection
    let rpc_api_str = env::var("RPC_API").unwrap();
    let rpc_url = rpc_api_str;
    let commitment = CommitmentConfig::processed();
    let rpc_client = RpcClient::new_with_commitment(rpc_url.to_string(),commitment);

    let wallet_balance = rpc_client.get_balance(&public_key).unwrap();
    println!("Balance : {}", wallet_balance);
    let recent_blockhash: Hash = rpc_client
        .get_latest_blockhash()
        .unwrap();
    println!("Recent Blockhash : {}",recent_blockhash);


    let sample_market="2Gcc963e2BY6syyjB3CGrFMw93e8c8zysygzu89Bxdic";

    let sample_mint="ALZNPVu3KUZ9jRpbuZKkQbPKf9wHFHpGS2mpjdDtofE1";

    let account_raw_data=rpc_client.get_account(&Pubkey::from_str_const(sample_market)).unwrap();

    println!("{:?}", account_raw_data);

    let account_raw_bytes: &[u8] = &account_raw_data.data;
    // println!("{:?}", account_raw_bytes);

    let market_data=market::Market::from_bytes(account_raw_bytes).unwrap();
    println!("{:?}", market_data);

    // zero_for_one : direction of swap
    // false : token 1 -> token 2
    // true : token 0 -> token 1
    let quote_data_1=quote::quote(&market_data, false, 300, 427779926819030658986966333).unwrap();
    println!("{:?}", quote_data_1);

    // let quote_data_2=quote::quote(&market_data, true, 100, market_data.sqrt_price_x96*2).unwrap();
    // println!("{:?}", quote_data_2);

    // let mut sample_swap_base_buy_tx:Transaction=build_tokenmill_swap_base_output(
    //     &wallet, 
    //     &sample_pool, 
    //     &sample_input_mint, 
    //     &sample_output_mint, 
    //     1000000, 
    //     4000, 
    //     true, 
    //     true,
    //     true,
    //     false
    // );

    // let mut sample_swap_base_sell_tx:Transaction=build_tokenmill_swap_base_input(
    //     &wallet, 
    //     &sample_pool, 
    //     &sample_output_mint, 
    //     &sample_input_mint, 
    //     4000, 
    //     0, 
    //     false, 
    //     true,
    //     true,
    //     true
    // );

    // sample_swap_base_buy_tx.sign(&[&wallet], recent_blockhash);
    // sample_swap_base_sell_tx.sign(&[&wallet], recent_blockhash);

    // let serialized_buy_transaction = bincode::serialize(&sample_swap_base_buy_tx).unwrap();
    // let serialized_sell_transaction = bincode::serialize(&sample_swap_base_sell_tx).unwrap();

    // let body_string=format!(
    //     r#" {{
    //         "jsonrpc": "2.0",
    //         "id": 1,
    //         "method": "sendBundle",
    //         "params": [
    //             [{:?}, {:?}],
    //             {{
    //             "encoding": "base64"
    //             }}
    //         ]
    //     }} "#
    //     ,general_purpose::STANDARD.encode(&serialized_buy_transaction)
    //     ,general_purpose::STANDARD.encode(&serialized_sell_transaction)
    // );

    // let http_client=Client::new();

    // let jito_url=format!("https://mainnet.block-engine.jito.wtf/api/v1/bundles?uuid={}",env::var("JITO_UUID").unwrap() );

    // let response = http_client
    //     .post(&jito_url)
    //     .header("Content-Type", "application/json")
    //     .header("Connection","keep-alive")
    //     .body(body_string)
    //     .send()
    //     .await;
    // match response {
    //     Ok(response) => {
    //         if response.status().is_success() {
    //             let response_text=response.text().await.unwrap();
    //             match serde_json::from_str::<Value>(&response_text){
    //                 Ok(parsed_json)=>{
    //                     let bundle_id=parsed_json.get("result");
    //                     println!("==== {} =====",jito_url);
    //                     if let Some(bundle_id) = parsed_json.get("result") {
    //                         if let Some(bundle_id_str)=bundle_id.as_str() {
    //                             println!("Bundle ID: {}", bundle_id_str);
    //                         }
    //                         else {
    //                             println!("==== {} =====",jito_url);
    //                             println!("Error processing response : {}", response_text);
    //                         }
    //                     }else {
    //                         println!("==== {} =====",jito_url);
    //                         println!("Error processing response : {}", response_text);
    //                     }
    //                 }
    //                 Err(err)=>{
    //                     println!("==== {} =====",jito_url);
    //                     println!("Error processing response : {}", err);
    //                 }
    //             }
    //         } else {
    //             let response_text=response.text().await.unwrap();
    //             println!("==== {} =====",jito_url);
    //             println!("Error submitting transaction: {}", response_text);
    //         }
    //     }
    //     Err(err) => {
    //         println!("==== {} =====",jito_url);
    //         println!("Error sending to : {}", err);
    //     }
    // }
}

// pub fn build_tokenmill_buy_base_input (
//     wallet : &Keypair, 
//     pool : &str, 
//     input_mint : &str, 
//     input_amount: u64,
//     output_mint : &str, 
//     output_threshold : u64, 
//     create_wsol_account : bool, 
//     create_token_account :bool, 
//     close_wsol_account:bool, 
//     close_token_account:bool 
// )-> Transaction{

//     let mut instructions=vec![];


//     let tokenmill_program="JoeGXemoPqPeGPEXA3Z3UbjoPoGqqfbg8PD58M7Rqj2";
//     let tokenmill_config="LFJxVxETTXwoxuuFCpqj3KihrYxmJc7maQFg4UjHZ3r";
//     let tokenmill_fee_reserve="3k2CNzkGCrdZ1VSKAxcCZcutcQxBvC8HSjg31YyfFn4h";
//     let tokenmill_protocol_fee="H5ykxF3zEN6biiXPLM8u4bbNMXxiAbBMQx5MXth1sC6";
//     let tokenmill_creator_fee="3k2CNzkGCrdZ1VSKAxcCZcutcQxBvC8HSjg31YyfFn4h";

//     let rent_program="SysvarRent111111111111111111111111111111111";
//     let token_program="TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
//     let associated_token_program="ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
//     let system_program="11111111111111111111111111111111";
//     let sol_mint="So11111111111111111111111111111111111111112";

//     let (market_pda, market_pda_bump)=Pubkey::find_program_address(&[b"market", Pubkey::from_str_const(target_mint).as_ref()], &Pubkey::from_str_const(tokenmill_program));

//     let input_token_account=get_associated_token_address(&wallet.pubkey(), &Pubkey::from_str_const(input_mint));
//     let target_token_account=get_associated_token_address(&wallet.pubkey(), &Pubkey::from_str_const(output_mint));

//     if create_input_account {
//         let create_input_account_instruction = create_associated_token_account(
//             &wallet.pubkey(), // Payer (funding the account creation)
//             &wallet.pubkey(),          // Wallet address owning the token account
//             &Pubkey::from_str_const(sol_mint),           // Token mint
//             &Pubkey::from_str_const(token_program)
//         );
//         instructions.push(create_input_account_instruction);
        
//     }

//     if input_mint ==sol_mint {
//         let wsol_transfer_instruction = system_instruction::transfer(&wallet.pubkey(), &input_token_account, input_amount);
//         instructions.push(wsol_transfer_instruction);
//         let sync_native_instruction =  sync_native(&Pubkey::from_str_const(token_program), &input_token_account).unwrap();
//         instructions.push(sync_native_instruction);
//     }

//     if create_output_account {
//         let create_output_account_instruction = create_associated_token_account(
//             &wallet.pubkey(), // Payer (funding the account creation)
//             &wallet.pubkey(),          // Wallet address owning the token account
//             &Pubkey::from_str_const(output_mint),           // Token mint
//             &Pubkey::from_str_const(token_program)
//         );
//         instructions.push(create_output_account_instruction);
//     }

//     let instruction_accounts=vec![
//         AccountMeta::new_readonly(Pubkey::from_str_const(tokenmill_config),false),//#1 config
//         AccountMeta::new(market_pda,false),//#2 market
//         AccountMeta::new_readonly(Pubkey::from_str_const(raydium_authority),false),//#2 raydium authority
//         AccountMeta::new_readonly(Pubkey::from_str_const(raydium_amm_config),false),//#3 raydium amm config
//         AccountMeta::new(Pubkey::from_str_const(pool),false),//#4 metadata
//         AccountMeta::new(input_token_account,false),//#5 input token account
//         AccountMeta::new(output_token_account,false),//#6 output token account
//         AccountMeta::new(input_vault,false),//#7 input vault
//         AccountMeta::new(output_vault,false),//#8 output vault
//         AccountMeta::new_readonly(Pubkey::from_str_const(token_program),false),//#9 input token program
//         AccountMeta::new_readonly(Pubkey::from_str_const(token_program),false),//#10 output token program
//         AccountMeta::new_readonly(Pubkey::from_str_const(input_mint),false),//#11 input mint
//         AccountMeta::new_readonly(Pubkey::from_str_const(output_mint),false),//#12 output mint
//         AccountMeta::new(observation_pda,false),//#13 observation
        
//     ];

//     let mut data: Vec<u8> = vec![0x8f, 0xbe, 0x5a, 0xda, 0xc4, 0x1e, 0x33, 0xde];

//     let mut input_amount_buffer = [0u8; 8];
//     input_amount_buffer.copy_from_slice(&input_amount.to_le_bytes());

//     let mut output_threshold_buffer = [0u8; 8];
//     output_threshold_buffer.copy_from_slice(&output_threshold.to_le_bytes());

//     data.extend_from_slice(&input_amount_buffer);
//     data.extend_from_slice(&output_threshold_buffer);

//     let mut instruction_raw_data=data;
    

//     let swap_instruction = Instruction {
//         program_id:Pubkey::from_str_const(raydium_cpmm_program),
//         accounts:instruction_accounts,
//         data: instruction_raw_data,
//     };
//     instructions.push(swap_instruction.clone());


//     if close_input_account {
//         let close_input_account_instruction=close_account( &Pubkey::from_str_const(token_program), &input_token_account, &wallet.pubkey(), &wallet.pubkey(), &[&wallet.pubkey()]).unwrap();
//         instructions.push(close_input_account_instruction);
//     }

//     if close_output_account {
        
//         let close_output_account_instruction=close_account( &Pubkey::from_str_const(token_program), &output_token_account, &wallet.pubkey(), &wallet.pubkey(), &[&wallet.pubkey()]).unwrap();
//         instructions.push(close_output_account_instruction);
        
//     }

//     //need to add jito tip instruction 
//     let jito_tip_instruction = system_instruction::transfer(&wallet.pubkey(), &Pubkey::from_str_const("ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt"), 10000);
//     instructions.push(jito_tip_instruction);

//     let message = Message::new(&instructions, Some(&wallet.pubkey()));
//     let mut transaction = Transaction::new_unsigned(message);
    
//     transaction
// }


// pub fn build_tokenmill_swap_base_output (
//     wallet : &Keypair, 
//     pool : &str, 
//     input_mint : &str, 
//     output_mint : &str, 
//     input_threshold: u64, 
//     output_amount : u64, 
//     create_input_account : bool, 
//     create_output_account :bool, 
//     close_input_account:bool, 
//     close_output_account:bool 
// )-> Transaction{

//     let mut instructions=vec![];


//     let raydium_cpmm_program="CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";

//     let raydium_authority="GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL";//PDA but fixed now
//     let raydium_amm_config="D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2";//PDA but fixed now

//     let rent_program="SysvarRent111111111111111111111111111111111";
//     let token_program="TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
//     let associated_token_program="ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
//     let system_program="11111111111111111111111111111111";

//     let sol_mint="So11111111111111111111111111111111111111112";
    

//     let (input_vault, input_vault_bump)=Pubkey::find_program_address(&[b"pool_vault", Pubkey::from_str_const(pool).as_ref(), Pubkey::from_str_const(input_mint).as_ref()], &Pubkey::from_str_const(raydium_cpmm_program));
//     let (output_vault, output_vault_bump)=Pubkey::find_program_address(&[b"pool_vault", Pubkey::from_str_const(pool).as_ref(), Pubkey::from_str_const(output_mint).as_ref()], &Pubkey::from_str_const(raydium_cpmm_program));

//     let (observation_pda, observation_bump)=Pubkey::find_program_address(&[b"observation", Pubkey::from_str_const(pool).as_ref()], &Pubkey::from_str_const(raydium_cpmm_program));

//     let input_token_account=get_associated_token_address(&wallet.pubkey(), &Pubkey::from_str_const(input_mint));
//     let output_token_account=get_associated_token_address(&wallet.pubkey(), &Pubkey::from_str_const(output_mint));

//     if create_input_account {
//         let create_input_account_instruction = create_associated_token_account(
//             &wallet.pubkey(), // Payer (funding the account creation)
//             &wallet.pubkey(),          // Wallet address owning the token account
//             &Pubkey::from_str_const(input_mint),           // Token mint
//             &Pubkey::from_str_const(token_program)
//         );
//         instructions.push(create_input_account_instruction);
//     }

//     if input_mint == "So11111111111111111111111111111111111111112" {
//         let wsol_transfer_instruction = system_instruction::transfer(&wallet.pubkey(), &input_token_account, input_threshold);
//         instructions.push(wsol_transfer_instruction);
//         let sync_native_instruction =  sync_native(&Pubkey::from_str_const(token_program), &input_token_account).unwrap();
//         instructions.push(sync_native_instruction);
//     }

//     if create_output_account {
//         let create_output_account_instruction = create_associated_token_account(
//             &wallet.pubkey(), // Payer (funding the account creation)
//             &wallet.pubkey(),          // Wallet address owning the token account
//             &Pubkey::from_str_const(output_mint),           // Token mint
//             &Pubkey::from_str_const(token_program)
//         );
//         instructions.push(create_output_account_instruction);
//     }

//     let instruction_accounts=vec![
//         AccountMeta::new(wallet.pubkey(),true),//#1
//         AccountMeta::new_readonly(Pubkey::from_str_const(raydium_authority),false),//#2 raydium authority
//         AccountMeta::new_readonly(Pubkey::from_str_const(raydium_amm_config),false),//#3 raydium amm config
//         AccountMeta::new(Pubkey::from_str_const(pool),false),//#4 metadata
//         AccountMeta::new(input_token_account,false),//#5 input token account
//         AccountMeta::new(output_token_account,false),//#6 output token account
//         AccountMeta::new(input_vault,false),//#7 input vault
//         AccountMeta::new(output_vault,false),//#8 output vault
//         AccountMeta::new_readonly(Pubkey::from_str_const(token_program),false),//#9 input token program
//         AccountMeta::new_readonly(Pubkey::from_str_const(token_program),false),//#10 output token program
//         AccountMeta::new_readonly(Pubkey::from_str_const(input_mint),false),//#11 input mint
//         AccountMeta::new_readonly(Pubkey::from_str_const(output_mint),false),//#12 output mint
//         AccountMeta::new(observation_pda,false),//#13 observation
        
//     ];

//     let mut data: Vec<u8> = vec![0x37, 0xd9, 0x62, 0x56, 0xa3, 0x4a, 0xb4, 0xad];

//     let mut input_threshold_buffer = [0u8; 8];
//     input_threshold_buffer.copy_from_slice(&input_threshold.to_le_bytes());

//     let mut output_amount_buffer = [0u8; 8];
//     output_amount_buffer.copy_from_slice(&output_amount.to_le_bytes());

//     data.extend_from_slice(&input_threshold_buffer);
//     data.extend_from_slice(&output_amount_buffer);

//     let mut instruction_raw_data=data;
    

//     let create_instruction = Instruction {
//         program_id:Pubkey::from_str_const(raydium_cpmm_program),
//         accounts:instruction_accounts,
//         data: instruction_raw_data,
//     };
//     instructions.push(create_instruction);

//     if close_input_account {
//         let close_input_account_instruction=close_account( &Pubkey::from_str_const(token_program), &input_token_account, &wallet.pubkey(), &wallet.pubkey(), &[&wallet.pubkey()]).unwrap();
//         instructions.push(close_input_account_instruction);
//     }

//     if close_output_account {
//         let close_output_account_instruction=close_account( &Pubkey::from_str_const(token_program), &output_token_account, &wallet.pubkey(), &wallet.pubkey(), &[&wallet.pubkey()]).unwrap();
//         instructions.push(close_output_account_instruction);
        
//     }

//     //need to add jito tip instruction 
//     let jito_tip_instruction = system_instruction::transfer(&wallet.pubkey(), &Pubkey::from_str_const("ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt"), 10000);
//     instructions.push(jito_tip_instruction);

//     let message = Message::new(&instructions, Some(&wallet.pubkey()));
//     let mut transaction = Transaction::new_unsigned(message);
    
//     transaction
// }