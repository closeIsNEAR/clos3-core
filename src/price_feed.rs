use near_sdk::{
    collections::TreeMap,
    borsh::{
        self,
        BorshDeserialize,
        BorshSerialize,
    }
};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct PriceFeed {
    history: TreeMap<u128, u128>,
    answer: u128,
    round: u128
}

impl PriceFeed {
    pub fn new() -> Self {
        Self {
            history: TreeMap::new(b"price_feed".to_vec()),
            answer: 0,
            round: 0
        }
    }


    pub fn latest_price(&self) -> u128 {
        self.answer
    }

    pub fn latest_round(&self) -> u128 {
        self.round
    }

    pub fn set_latest_answer(&mut self, answer: u128) {
        self.round += 1;
        self.answer = answer;
        self.history.insert(&self.round, &answer);
    }

    pub fn get_round_data(&self, round: u128) -> (u128, u128) {
        let price = self.history.get(&round).expect("ERR_INVALID_ROUND");
        (round, price)
    }
}