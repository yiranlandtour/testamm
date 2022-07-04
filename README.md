near-blank-project Smart Contract
==================

A [smart contract] written in [Rust] for an app initialized with [create-near-app]


Quick Start
===========

use [npm run build:contract] to build contract

```rust
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
```

