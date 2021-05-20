use crate::utils::*;
const AMM_DEPOSIT: u128 = 50000000000000000000000;
pub fn init_balance() -> u128 {
    to_yocto("1000")
}

pub struct TestAccount {
    pub account: UserAccount
}

impl TestAccount {
    pub fn new(
        master_account: Option<&UserAccount>, 
        account_id: Option<&str>
    ) -> Self {
        match master_account {
            Some(master_account) => {
                let account = master_account.create_user(account_id.expect("expected account id").to_string(), init_balance());
                storage_deposit(AMM_CONTRACT_ID, &master_account, AMM_DEPOSIT, Some(account.account_id()));
                storage_deposit(TOKEN_CONTRACT_ID, &master_account, SAFE_STORAGE_AMOUNT, Some(account.account_id()));
                storage_deposit(ORACLE_CONTRACT_ID, &master_account, SAFE_STORAGE_AMOUNT, Some(account.account_id()));
                near_deposit(&account, init_balance() / 2);
                Self {
                    account
                }
            },
            None => Self { account: init_simulator(None) }
        }
    }
    /*** Getters ***/
    pub fn get_token_balance(&self, account_id: Option<String>) -> u128 {
        let account_id = match account_id {
            Some(account_id) => account_id,
            None => self.account.account_id()
        };

        let res: U128 = self.account.view(
            PendingContractTx::new(
                TOKEN_CONTRACT_ID, 
                "ft_balance_of", 
                json!({
                    "account_id": account_id
                }), 
                true
            )
        ).unwrap_json();

        res.into()
    }

    pub fn get_pool_balances(&self, market_id: u64) -> Vec<u128> {
        let wrapped_balance: Vec<U128> = self.account.view(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "get_pool_balances", 
                json!({
                    "market_id": U64(market_id)
                }), 
                true
            )
        ).unwrap_json();

        wrapped_balance.into_iter().map(|wrapped_balance| { wrapped_balance.into() }).collect()

    }

    /*** Setters ***/
    pub fn create_market(&self, outcomes: u16, fee_opt: Option<U128>) -> ExecutionResult {
        let msg = json!({
            "CreateMarketArgs": {
                "description": empty_string(),
                "extra_info": empty_string(),
                "outcomes": outcomes,
                "outcome_tags": empty_string_vec(outcomes),
                "categories": empty_string_vec(outcomes),
                "end_time": env_time(),
                "resolution_time": env_time(),
                "collateral_token_id": TOKEN_CONTRACT_ID,
                "swap_fee": fee_opt,
                "is_scalar": false
            }
        }).to_string();
        self.ft_transfer_call(AMM_CONTRACT_ID.to_string(), to_yocto("100"), msg)
    }

    pub fn add_liquidity(&self, market_id: u64, amount: u128, weights: Option<Vec<U128>>) -> ExecutionResult {
        let msg  = json!({
            "AddLiquidityArgs": {
                "market_id": market_id.to_string(),
                "weight_indication": weights,
            }
        }).to_string();
        self.ft_transfer_call(AMM_CONTRACT_ID.to_string(), amount, msg)
    }

    pub fn ft_transfer_call(
        &self,
        receiver: String,
        amount: u128,
        msg: String
    ) -> ExecutionResult {        
        let res = self.account.call(
            PendingContractTx::new(
                TOKEN_CONTRACT_ID, 
                "ft_transfer_call", 
                json!({
                    "receiver_id": receiver,
                    "amount": U128(amount),
                    "msg": msg,
                    "memo": "".to_string()
                }), 
                true
            ),
            1,
            DEFAULT_GAS
        );
        println!("{:?}", res);
        assert!(res.is_ok(), "ft_transfer_call failed with res: {:?}", res);
        res
    }

}