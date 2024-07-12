use anchor_lang::prelude::*;

#[account]
pub struct Bid {
    pub index: u64, // 8
    pub amount: u64, // Amount of SOL offered in lamports, 8
    pub rate: u64, // Rate in lamports, 8
    pub bidder: Pubkey, // Public key of the bidder, 32
    pub fulfilled: bool, // Indicates if the bid is fulfilled, 1
}

impl Bid {
    // Maybe unsafe? Should panic if amount > self.amount.
    pub fn partial_fill(
        &mut self, 
        amount: u64
    ) -> () {
        self.amount -= amount;

        if self.amount == 0 {
            self.fulfilled = true;
        }
    }
}
