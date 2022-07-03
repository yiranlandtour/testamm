// use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, ext_contract, require, AccountId, Balance, Gas, PromiseOrValue};
use near_sdk::{log, near_bindgen, PanicOnDefault};

const NO_DEPOSIT: Balance = 0;
const BASE_GAS: u64 = 5_000_000_000_000;
const PROMISE_CALL: u64 = 5_000_000_000_000;
const GAS_FOR_FT_ON_TRANSFER: u64 = BASE_GAS + PROMISE_CALL;
const GAS_FOR_FT_TRANSFER: u64 = 20_000_000_000_000;

#[ext_contract(ext_self)]
pub trait ReturnCallback {
    fn owner_transfer_in_callback(amount_return: U128, is_positive_direction: bool);
    fn exchange_transfer_in_callback(
        origin_user: AccountId,
        exchange_origin_amount: U128,
        is_positive_direction: bool,
    ) -> PromiseOrValue<()>;
    fn exchange_transfer_out_callback(
        exchange_origin_amount: U128,
        change_amount: U128,
        is_positive_direction: bool,
    );
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AMM {
    owner_address: AccountId,

}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Pool {
    owner_address: AccountId,
    token_a_address: AccountId,
    token_a_decimal: u8,
    token_a_symbol: String,
    token_a_pool_amount: Balance,
    token_b_address: AccountId,
    token_b_decimal: u8,
    token_b_symbol: String,
    token_b_pool_amount: Balance,
    // lp_token: AccountId,
    // mine_token: AccountId,
}

#[near_bindgen]
impl Pool {
    #[private]
    fn new(
        owner_address: AccountId,
        token_a_address: AccountId,
        token_a_decimal: u8,
        token_a_symbol: String,
        token_b_address: AccountId,
        token_b_decimal: u8,
        token_b_symbol: String,
    ) -> Pool {
        Pool {
            owner_address,
            token_a_address,
            token_a_decimal,
            token_a_symbol,
            token_a_pool_amount: 0,
            token_b_address,
            token_b_symbol,
            token_b_decimal,
            token_b_pool_amount: 0,
            
        }
    }
    #[init]
    pub fn init_pool(
        owner_address: AccountId,
        token_a_address: AccountId,
        token_a_decimal: u8,
        token_a_symbol: String,
        token_b_address: AccountId,
        token_b_decimal: u8,
        token_b_symbol: String,
    ) -> Self {
        require!(!env::state_exists(), "Already initialized");
        // log!("init amm owner address:{:?},token a address:{:?},token a decimal:{},token a symbol:{},token b address:{:?},token b symbol:{},token b decimal:{} success",
        //     token_a_address.as_str(), token_a_decimal, token_a_symbol, token_b_address.as_str(),
        //     token_b_symbol, token_b_decimal
        // );
        Self::new(
            owner_address,
            token_a_address,
            token_a_decimal,
            token_a_symbol,
            token_b_address,
            token_b_decimal,
            token_b_symbol,
        )
    }

    pub fn get_name(&self) -> String {
        format!("{}-{}", self.token_a_symbol, self.token_b_symbol)
    }

    pub fn get_decimals(&self) -> (u8, u8) {
        (self.token_a_decimal, self.token_b_decimal)
    }

    pub fn get_pool_token_amount(&self, is_token_a: bool) -> U128 {
        if is_token_a {
            return self.token_a_pool_amount.into();
        }
        self.token_b_pool_amount.into()
    }

    
    
    pub fn get_ratio_atob(&self, pay_token_amount: U128, is_positive: bool) -> U128 {
        if self.token_a_pool_amount == 0 || self.token_b_pool_amount == 0 {
            return 0.into();
        }
        let k = self.token_a_pool_amount * self.token_b_pool_amount;
        if is_positive {
            return (self.token_b_pool_amount
                - (k / (self.token_a_pool_amount + Balance::from(pay_token_amount))))
            .into();
        }
        (self.token_a_pool_amount
            - (k / (self.token_b_pool_amount + Balance::from(pay_token_amount))))
        .into()
    }


}