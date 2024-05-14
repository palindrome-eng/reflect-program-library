#![allow(clippy::arithmetic_side_effects)]
#![cfg(feature = "test-sbf")]
mod helpers;

use {
    helpers::*,
    solana_program_test::*,
    solana_sdk::{signature::Signer, transaction::Transaction},
    reflect_single_pool::{error::ReflectPoolError, id, instruction},
    test_case::test_case,
};

const UPDATED_NAME: &str = "reflectSOL";
const UPDATED_SYMBOL: &str = "rSOL";
const UPDATED_URI: &str = "https://arweave.net/ZVZLNjT7jKomzeI5u8e4pHv40JCqsHicaG3iTPBxk5s";

#[tokio::test]
async fn success_update_pool_token_metadata() {
    let mut context = program_test(false).start_with_context().await;
    let accounts = ReflectPoolAccounts::default();
    accounts.initialize(&mut context).await;

    let instruction = instruction::update_token_metadata(
        &id(),
        &accounts.vote_account.pubkey(),
        &accounts.withdrawer.pubkey(),
        UPDATED_NAME.to_string(),
        UPDATED_SYMBOL.to_string(),
        UPDATED_URI.to_string(),
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &accounts.withdrawer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let metadata = get_metadata_account(&mut context.banks_client, &accounts.mint).await;

    assert!(metadata.name.starts_with(UPDATED_NAME));
    assert!(metadata.symbol.starts_with(UPDATED_SYMBOL));
    assert!(metadata.uri.starts_with(UPDATED_URI));
}

#[tokio::test]
async fn fail_no_signature() {
    let mut context = program_test(false).start_with_context().await;
    let accounts = ReflectPoolAccounts::default();
    accounts.initialize(&mut context).await;

    let mut instruction = instruction::update_token_metadata(
        &id(),
        &accounts.vote_account.pubkey(),
        &accounts.withdrawer.pubkey(),
        UPDATED_NAME.to_string(),
        UPDATED_SYMBOL.to_string(),
        UPDATED_URI.to_string(),
    );
    assert_eq!(instruction.accounts[3].pubkey, accounts.withdrawer.pubkey());
    instruction.accounts[3].is_signer = false;

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let e = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err();
    check_error(e, ReflectPoolError::SignatureMissing);
}

enum BadWithdrawer {
    Validator,
    Voter,
    VoteAccount,
}

#[test_case(BadWithdrawer::Validator; "validator")]
#[test_case(BadWithdrawer::Voter; "voter")]
#[test_case(BadWithdrawer::VoteAccount; "vote_account")]
#[tokio::test]
async fn fail_bad_withdrawer(withdrawer_type: BadWithdrawer) {
    let mut context = program_test(false).start_with_context().await;
    let accounts = ReflectPoolAccounts::default();
    accounts.initialize(&mut context).await;

    let withdrawer = match withdrawer_type {
        BadWithdrawer::Validator => &accounts.validator,
        BadWithdrawer::Voter => &accounts.voter,
        BadWithdrawer::VoteAccount => &accounts.vote_account,
    };

    let instruction = instruction::update_token_metadata(
        &id(),
        &accounts.vote_account.pubkey(),
        &withdrawer.pubkey(),
        UPDATED_NAME.to_string(),
        UPDATED_SYMBOL.to_string(),
        UPDATED_URI.to_string(),
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, withdrawer],
        context.last_blockhash,
    );

    let e = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err();
    check_error(e, ReflectPoolError::InvalidMetadataSigner);
}
