mod response;
mod utils;

use std::str::FromStr;

use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use near_sdk::env::promise_result;
use near_sdk::json_types::U128;
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance,PanicOnDefault, PromiseOrValue};
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

use response::MetadataTokens;
use utils::parse_promise_result;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Amm {
    owner_address: AccountId,

    account_asset_a: AccountId,
    token_a_pool_amount: Balance,

    account_asset_b: AccountId,
    token_b_pool_amount: Balance,

    metadata_token_a: Option<FungibleTokenMetadata>,
    metadata_token_b: Option<FungibleTokenMetadata>,
}

#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_balance_of(&mut self, account_id: AccountId) -> U128;

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;

    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;

    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

#[ext_contract(ext_self_metadata)]
pub trait MetadataReceiver {
    fn cb_initialization_metadata(&mut self) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_self_tokens)]
pub trait TokenRelayer {
    fn cb_transfer_token(
        &self,
        counterparty: AccountId,
        token_received: AccountId,
        amount_received: U128,
    ) -> PromiseOrValue<U128>;
}

#[near_bindgen]
impl Amm {
    #[init]
    pub fn new(
        owner_address: String, 
        account_asset_a: String,

        account_asset_b: String,

    ) -> Self {
        ext_ft::ext(AccountId::from_str(&account_asset_a).unwrap())
            .ft_metadata()
            .and(ext_ft::ext(AccountId::from_str(&account_asset_a).unwrap()).ft_metadata())
            .then(ext_self_metadata::ext(env::current_account_id()).cb_initialization_metadata());

        Self {
            owner_address: AccountId::from_str(&owner_address).unwrap(),
            account_asset_a: AccountId::from_str(&account_asset_a).unwrap(),
            token_a_pool_amount:0,
            token_b_pool_amount:0,
            account_asset_b: AccountId::from_str(&account_asset_b).unwrap(),

            metadata_token_a: None,
            metadata_token_b: None,
        }
    }
}

#[near_bindgen]
impl Amm {
    
    pub fn get_ticker(&self) -> String {
        let meta_a = &self.metadata_token_a.as_ref();
        let meta_b = &self.metadata_token_b.as_ref();
        format!("{}-{}", meta_a.unwrap().symbol, meta_b.unwrap().symbol)
    }

    pub fn get_decimals(&self) -> (u8, u8) {
        let meta_a = &self.metadata_token_a.as_ref();
        let meta_b = &self.metadata_token_b.as_ref();
        (meta_a.unwrap().decimals, meta_b.unwrap().decimals)
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

    #[private]
    pub fn initialization_metadata(&mut self) {
        assert_eq!(env::promise_results_count(), 2, "INVALID_PROMISE_RESULTS");

        let metadata = parse_promise_result::<FungibleTokenMetadata>(&promise_result(0));
        if metadata.is_some() {
            self.metadata_token_a = metadata;
        } else {
            env::panic_str("Error when querying token A metadata.");
        }

        let metadata = parse_promise_result::<FungibleTokenMetadata>(&promise_result(1));
        if metadata.is_some() {
            self.metadata_token_b = metadata;
        } else {
            env::panic_str("Error when querying token B metadata.");
        }
    }

    pub fn ft_on_transfer(
        self,
        sender_id: AccountId,
        amount: U128,
        _msg: String,
    ) -> PromiseOrValue<U128> {
        if env::predecessor_account_id() != self.account_asset_a
            && env::predecessor_account_id() != self.account_asset_b
        {
            near_sdk::env::panic_str("Method can only be called by registered assets");
        }

        if sender_id == self.owner_address {
            return PromiseOrValue::Value(U128(0));
        }

        let this_id = env::current_account_id();

        return ext_ft::ext(self.account_asset_a)
            .ft_balance_of(this_id.clone())
            .and(ext_ft::ext(self.account_asset_b).ft_balance_of(this_id))
            .then(
                ext_self_tokens::ext(env::current_account_id()).cb_transfer_token(
                    sender_id,
                    env::predecessor_account_id(),
                    amount,
                ),
            )
            .into();

    }

    #[private]
    pub fn cb_transfer_token(
        self,
        counterparty: AccountId,
        token_received: AccountId,
        amount_received: U128,
    ) {
        let balance_token_a: u128 = parse_promise_result::<U128>(&promise_result(0))
            .unwrap()
            .into();
        let balance_token_b: u128 = parse_promise_result::<U128>(&promise_result(1))
            .unwrap()
            .into();

        let amount_received: u128 = amount_received.into();

        let acc_a = self.account_asset_a.clone();
        let acc_b = self.account_asset_b.clone();

        let previous_ratio = match token_received.clone() {
            _acc_a => (balance_token_a - amount_received) * balance_token_b,
            _acc_b => (balance_token_b - amount_received) * balance_token_a,
            _ => env::panic_str("Unsupported asset"),
        };

        match token_received {
            _acc_a => {
                let to_send = balance_token_b - previous_ratio / balance_token_a;
                ext_ft_core::ext(acc_b).with_attached_deposit(1).ft_transfer(counterparty, U128(to_send), None);

            }
            _acc_b => {
                let to_send = balance_token_a - previous_ratio / balance_token_b;
                ext_ft_core::ext(acc_a).with_attached_deposit(1).ft_transfer(counterparty, U128(to_send), None);
            }
            _ => env::panic_str("Unsupported asset"),
        }
    }

    #[result_serializer(borsh)]
    pub fn metadata_tokens(self) -> MetadataTokens {
        return MetadataTokens {
            metadata_token_a: self.metadata_token_a.unwrap(),
            metadata_token_b: self.metadata_token_b.unwrap(),
        };
    }
}


#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    use near_sdk::serde_json::{self, json};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, Balance};
    use near_sdk_sim::{init_simulator, ExecutionResult, UserAccount, DEFAULT_GAS};

    const AMM_ACCOUNT: &str = "near_amm_simple";
    const AMM_OWNER: &str = "amm_owner";
    const TOKEN_A_ACCOUNT: &str = "wnear";
    const TOKEN_A_SYMBOL: &str = "WNEAR";
    const TOKEN_A_OWNER: &str = "alice";
    const TOKEN_A_DECIMALS: u8 = 8;
    const TOKEN_A_REAL_WANTED: Balance = 21_000_000;
    const TOKEN_B_ACCOUNT: &str = "wusdt";
    const TOKEN_B_SYMBOL: &str = "WUSDT";
    const TOKEN_B_OWNER: &str = "bob";
    const TOKEN_B_DECIMALS: u8 = 8;
    const TOKEN_B_REAL_WANTED: Balance = 1000_000_000;
    const NEAR2YOCTO: u128 = 1000_000_000_000_000_000_000_000; // 10 ** 24

    near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
        // update `contract.wasm` for your contract's name
        AMM_WASM_BYTES => "../res/amm.wasm",
        // if you run `cargo build` without `--release` flag:
        FT_TOEKN_WASM_BYTES => "../res/fungible_token.wasm",
    }

    fn new_amm() -> Amm {
        Amm::new(
            AccountId::new_unchecked(AMM_OWNER.to_string()),
            AccountId::new_unchecked(TOKEN_A_ACCOUNT.to_string()),
            AccountId::new_unchecked(TOKEN_B_ACCOUNT.to_string()),
        )
    }

#[test]
fn test_unit_get_ratio() {
    let mut contract = new_amm();
    let per_token_a_amount: Balance =
        (10.0_f64.powi(contract.token_a_decimal as i32) as i64) as Balance;
    let per_token_b_amount: Balance =
        (10.0_f64.powi(contract.token_b_decimal as i32) as i64) as Balance;
    contract.token_a_pool_amount = 10 * per_token_a_amount;
    contract.token_b_pool_amount = 40_000 * per_token_b_amount;
    let simul_token_b_amount =
        contract.get_ratio_atob(U128::from(1 * per_token_a_amount), true);
    let simul_token_b_without_decimal =
        u128::from(simul_token_b_amount) / u128::from(per_token_b_amount);
    assert_eq!(simul_token_b_without_decimal, 3636);

    let simul_token_a_amount =
        contract.get_ratio_atob(U128::from(4000 * per_token_a_amount), false);
    assert_eq!(u128::from(simul_token_a_amount), 90909091);
}
}