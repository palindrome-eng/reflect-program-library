### solana-stake-markets (ssm) 
SSM is a simple and intuitive way to allow for order book style bids on natively staked solana. specifically by specifying the premium rate you are willing to pay and funding this bid with X amount of liquid SOL you create an environment whereby stake_account holders can instantly sell active stake. 

⚠️ SSM is in active-development and is non-audited onchain software. it operates under a public source license available for public education, viewing and inspiration but not to be used for commercial purposes in any format. 

### development pipeline

- [x] implement the order_book statistics PDA for use in bid derivation.
- [x] create a bid at X rate with Y amount of SOL funding.
- [x] close a bid providing it does not own stake_accounts and receive remaining lamports.
- [x] validate minimum rates and other bid safety assumptions.
- [x] implement the ability to sell stake_account into a bid.
- [x] allow client to supply remaining accounts (bids) in the event that stake_accounts need to be split to fulfill sell order.
- [ ] validate the transfer of stake authorisation - new auth owner, and the increase in SOL to seller.
- [ ] validate that arbitrary sale amounts can be sold into multiple bids via the split stake implementation.
- [ ] check if selling stake will empty a bid and close the bid account for bidder.
- [ ] validate changes to the order_book statistics PDA on sale of stake_account. 
- [ ] implement more extensive checks around the stake_account handling.
- [ ] finalise end-to-end testing file which runs through all-scenarios start to finish.
- [ ] submit for audit.