#![no_std]

use soroban_auth::{Identifier, Signature};
use soroban_sdk::{AccountId, Address, BytesN, contractimpl, contracttype, Env, Symbol, symbol, Vec, vec};

mod token {
    soroban_sdk::contractimport!(file = "soroban_token_spec.wasm");
}

#[contracttype]
pub struct State {
    admin: Address,
    currency: BytesN<32>,
    total_acct: i128,
    total_amnt: i128,
}
const STATE: Symbol = symbol!("STATE");
const PUB_KEYS: Symbol = symbol!("PUB_KEYS");

pub trait MassPayoutContractTrait {
    fn initialize(
        env: Env,
        admin: Address,
        currency: BytesN<32>,
    );

    fn get_state(env: Env) -> State;

    fn set_acc(env: Env, account: AccountId, amount: i128);

    fn get_acc(env: Env, account: AccountId) -> i128;

    fn deposit(env: Env, amount: i128);

    fn payout(env: Env);
}

pub struct MassPayoutContract;

#[contractimpl]
impl MassPayoutContractTrait for MassPayoutContract {
    fn initialize(
        env: Env,
        admin: Address,
        currency: BytesN<32>,
    ) {
        if env.data().has(STATE) {
            panic!("ALREADY_INIT");
        }

        let state = State {
            admin,
            currency,
            total_acct: 0,
            total_amnt: 0
        };

        env.data().set(STATE, state);

        env.data().set(PUB_KEYS, vec![&env] as Vec<Address>);
    }

    fn get_state(env: Env) -> State {
        is_initialized(&env);
        env.data().get(STATE).unwrap().unwrap()
    }

    fn set_acc(env: Env, account: AccountId, amount: i128) {
        is_initialized(&env);
        is_admin(&env);

        if amount < 1 {
            panic!("AMOUNT_AT_LEAST_1");
        }

        let mut current_state: State = env.data().get(STATE).unwrap().unwrap();
        if env.data().has(&account) {
            let current_amount: i128 = env.data().get(&account).unwrap().unwrap();
            current_state.total_amnt = current_state.total_amnt - current_amount + amount;
        } else {
            add_recipient(&env, account.clone());
            current_state.total_acct = current_state.total_acct + 1;
            current_state.total_amnt = current_state.total_amnt + amount;
        }

        env.data().set(account.clone(), amount);
        env.data().set(STATE, current_state);
    }

    fn get_acc(env: Env, account: AccountId) -> i128 {
        is_initialized(&env);
        env.data().get(account).unwrap().unwrap()
    }

    fn deposit(env: Env, amount: i128) {
        let state = Self::get_state(env.clone());
        let currency_client = token::Client::new(&env, state.currency);
        let contract_id = Identifier::Contract(env.current_contract());

        currency_client.xfer_from(
            &Signature::Invoker,
            &0,
            &Identifier::from(&env.invoker()),
            &contract_id,
            &amount,
        );
    }

    fn payout(env: Env) {
        let state = Self::get_state(env.clone());
        let currency_client = token::Client::new(&env, state.currency);
        let amount_to_send = state.total_amnt;
        let contract_id = Identifier::Contract(env.current_contract());
        let amount_in_vault = currency_client.balance(&contract_id);

        if amount_in_vault < amount_to_send {
            panic!("VAULT_UNDERFUNDED");
        }

        let recipients = get_recipients(&env);

        for recipient in recipients {
            let account_id = recipient.unwrap();
            let amount = Self::get_acc(env.clone(), account_id.clone());
            currency_client.xfer(
                &Signature::Invoker,
                &currency_client.nonce(&Signature::Invoker.identifier(&env)),
                &Identifier::Account(account_id.clone()),
                &amount,
            );
        }
    }
}

fn is_initialized(env: &Env) {
    if env.data().has(STATE) == false {
        panic!("STATE_NOT_STARTED");
    }
}

fn is_admin(env: &Env) {
    let state: State = env.data().get(STATE).unwrap().unwrap();
    let invoker: Address = env.invoker();

    if invoker != state.admin {
        panic!("ONLY_ADMIN");
    }
}

fn get_recipients(env: &Env) -> Vec<AccountId> {
    env.data()
      .get(PUB_KEYS)
      .unwrap()
      .unwrap()
}

fn add_recipient(env: &Env, account: AccountId) {
    let mut addresses = get_recipients(&env);
    addresses.push_back(account);

    env.data().set(PUB_KEYS, addresses);
}

mod test;