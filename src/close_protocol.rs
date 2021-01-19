use near_sdk::{
    near_bindgen,
    AccountId,
    env,
    borsh::{
        self,
        BorshDeserialize,
        BorshSerialize,
    }
};

use crate::token::FungibleToken;
use crate::price_feed::PriceFeed;

pub const BASIS_POINTS_DIVISOR: u128 = 10000;
pub const INITIAL_REBASE_DIVISOR: u128 = 10_000_000_000;
pub const MAX_DIVISOR: u64 = u64::MAX;
pub const MAX_PRICE: u128 = u128::MAX;
pub const TOKEN_DENOM: u128 = 1_000_000_000_000_000_000;


/*** operators that take decimals into account ***/
pub fn div(a: u128, b: u128) -> u128 {
    let c0 = a * TOKEN_DENOM;
    let c1 = c0 + (b / 2);

    c1 / b
}

pub fn mul(a: u128, b: u128) -> u128 {
    let c0 = a * b;

    let c1 = c0 + (TOKEN_DENOM / 2);

    c1 / TOKEN_DENOM
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct CloseProtocol {
    bull_token: FungibleToken,
    bear_token: FungibleToken,
    price_feed: PriceFeed,

    // divisors
    previous_bull_divisor: u128,
    previous_bear_divisor: u128,
    cached_bull_divisor: u128,
    cached_bear_divisor: u128,

    last_price: u128,
    last_round: u128,

    multiplier_basis_points: u128,
    max_profit_basis_points: u128,
    fee_reserve: u128
}

impl Default for CloseProtocol {
    fn default() -> Self {
        panic!("not initiated")
    }
}

#[near_bindgen]
impl CloseProtocol {
    #[init]
    pub fn init(multiplier_basis_points: u128, max_profit_basis_points: u128) -> Self {
        assert!(!env::state_exists(), "Already initialized");

        Self {
            bull_token: FungibleToken::new(0, env::current_account_id(), 0),
            bear_token: FungibleToken::new(1, env::current_account_id(), 0),
            price_feed: PriceFeed::new(),

            previous_bull_divisor: 0,
            previous_bear_divisor: 0,
            cached_bull_divisor: 0,
            cached_bear_divisor: 0,

            last_price: TOKEN_DENOM,
            last_round: 0,

            multiplier_basis_points,
            max_profit_basis_points,
            fee_reserve: 0,
        }
    }

    pub fn rebase(&mut self) -> bool {
        let last_price = self.last_price;
        let next_price = self.latest_price();
        let latest_round = self.latest_round();
        let last_round_timestamp = self.last_round;

        let (cached_bull_divisor, cached_bear_divisor) = self.get_divisors(last_price, next_price);

        if latest_round - self.last_round == 1 {
            self.last_price = next_price;
            self.last_round = latest_round;
            self.previous_bull_divisor = self.cached_bull_divisor;
            self.previous_bear_divisor = self.cached_bear_divisor;
            self.cached_bull_divisor = cached_bull_divisor;
            self.cached_bear_divisor = cached_bear_divisor;
            return true;
        }

        let (ok, previous_price): (bool, u128) = self.get_round_price(latest_round);

        true
    }

    fn get_round_price(&self, round: u128) -> (bool, u128) {
        let (_round, price) = self.price_feed.get_round_data(round);

        if price == 0 || price > MAX_PRICE {
            (false, 0)
        } else {
            (true, price)
        }
    }

    
    fn latest_price(& self) -> u128 {
        let price = self.price_feed.latest_price();

        if price == 0 || price > MAX_PRICE {
            self.last_price
        } else {
            price
        }
    }
    
    fn latest_round(& self) -> u128 {
        self.price_feed.latest_round()
    }

    fn get_divisors(&self, last_price: u128, next_price: u128) -> (u128, u128) {
        if last_price == next_price {
            return (self.cached_bear_divisor, self.cached_bull_divisor)
        }

        let bull_ref_supply = self.bull_token.total_supply;
        let bear_ref_supply = self.bear_token.total_supply;

        let mut total_bulls = bull_ref_supply / self.cached_bull_divisor as u128;
        let mut total_bears = bear_ref_supply / self.cached_bear_divisor as u128;

        let ref_supply = if total_bulls < total_bears { total_bulls } else { total_bears };

        let delta = if next_price > last_price { next_price - last_price } else { last_price - next_price };

        let mut profit = div(mul(div(mul(ref_supply, delta), last_price), self.multiplier_basis_points), BASIS_POINTS_DIVISOR);
        
        let max_profit = div(mul(ref_supply, self.max_profit_basis_points), BASIS_POINTS_DIVISOR);

        if profit > max_profit { profit = max_profit }

        total_bulls = if next_price > last_price { total_bulls + profit } else { total_bulls - profit };
        total_bears = if next_price > last_price { total_bears - profit } else { total_bears + profit };
        
        (self.get_next_divisor(bull_ref_supply, total_bulls, self.cached_bull_divisor), self.get_next_divisor(bear_ref_supply, total_bears, self.cached_bear_divisor))
    }

    fn get_next_divisor(&self, ref_supply: u128, next_supply: u128, fall_back_divisor: u128) -> u128 {
        if next_supply == 0 {
            return INITIAL_REBASE_DIVISOR
        }

        let divisor = ref_supply * 10 / next_supply + 9 / 10;

        if divisor == 0 || divisor > MAX_DIVISOR as u128 { return fall_back_divisor }
        
        divisor
    }
}



#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    
}