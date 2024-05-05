use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
        commitment_config::CommitmentConfig,
    },
    Client, Cluster,
};
use std::str::FromStr;
use solana_stake_market::instructions::{place_bid, PlaceBid};  // Adjust according to actual paths
use solana_stake_market::states::{Bid};


mod utils {
    use super::*;
    use solana_stake_market::states::Bid; // Ensure Bid is accessible from states module

    pub fn setup() -> (Client, Keypair) {
        let payer = Keypair::new();
        let program_id = Pubkey::from_str("sSmYaKe6tj5VKjPzHhpakpamw1PYoJFLQNyMJD3PU37").unwrap();
        let client = Client::new_with_options(
            Cluster::Localnet,
            Box::new(payer),
            CommitmentConfig::processed(),
        );

        (client, program_id)
    }
}

#[tokio::test]
async fn test_place_bids_at_different_rates() {
    let (client, program_id) = utils::setup();
    let program = client.program(program_id);

    let rates = [9700, 9800, 9900]; // Rates as per 0.97:1, 0.98:1, 0.99:1
    let mut accounts = vec![];

    for rate in rates.iter() {
        let bid_account = Keypair::new();

        // Airdrop SOL to the bid account to cover transaction fees and operational costs
        let airdrop_request = client.request_airdrop(&bid_account.pubkey(), 1_000_000_000).await.unwrap();
        client.confirm_transaction(&airdrop_request).await.unwrap();

        program.request()
            .accounts({
                let accounts = anchor_lang::prelude::Accounts {
                    bid: bid_account.pubkey(),
                    user: client.payer(),
                    system_program: system_program::ID,
                };
                accounts
            })
            .args(anchor_lang::prelude::InstructionData::PlaceBid {
                amount: 1_000_000_000, // 1 SOL in lamports
                rate: *rate,
            })
            .signer(&bid_account)
            .send()
            .await
            .unwrap();

        accounts.push(bid_account.pubkey());
    }

    // Verification
    for account in accounts {
        let bid = program.account::<Bid>(account).await.unwrap();
        assert_eq!(bid.fulfilled, false);
        println!("Bid at rate {} is not fulfilled.", bid.bid_rate);
    }
}

#[tokio::test]
async fn test_invalid_bid_rate() {
    let (client, program_id) = utils::setup();
    let program = client.program(program_id);

    let bid_account = Keypair::new();
    let result = program.request()
        .accounts(anchor_lang::prelude::Accounts {
            bid: bid_account.pubkey(),
            user: client.payer(),
            system_program: system_program::ID,
        })
        .args(anchor_lang::prelude::InstructionData::PlaceBid {
            amount: 1_000_000_000, // 1 SOL in lamports
            rate: 15000, // Invalid rate
        })
        .signer(&bid_account)
        .send()
        .await;

    assert!(result.is_err());
    println!("Attempting to place a bid with an invalid rate correctly failed.");
}
