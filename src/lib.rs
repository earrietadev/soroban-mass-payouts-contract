#![no_std]

use core::ops::{Add, Sub};
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{BigInt, AccountId, Address, BytesN, contracterror, contractimpl, contracttype, Env, panic_with_error, Symbol, symbol, Vec, vec};

mod token {
    soroban_sdk::contractimport!(file = "soroban_token_spec.wasm");
}

#[contracttype]
pub struct State {
    admin: Address,
    currency: BytesN<32>,
    total_acct: u32,
    total_amnt: u32,
}
const STATE: Symbol = symbol!("STATE");
const PUB_KEYS: Symbol = symbol!("PUB_KEYS");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInit = 0,
    VaultUnderfunded = 1,
    AmountAtLeast1 = 2,
    StateNotStarted = 3,
    OnlyAdmin = 4,
}

pub trait MassPayoutContractTrait {

    // This method is what it sets the contract configuration
    // Most methods in the contract won't work if this method hasn't been called before
    // Method can only be called once
    // "admin" can be other than the deployer of the contract
    // "currency" is the token contract id which you will pay recipients with
    fn initialize(
        env: Env,
        admin: Address,
        currency: BytesN<32>,
    );

    // This method is used to check if the method "initialize" was already called
    // If not initialized, the method will throw an error
    fn init_done(env: Env);

    // A method to return the current state of the contract
    // This method is meant to be used internally by the contract
    fn get_state(env: Env) -> State;

    // This method returns the amount an account is set to receive during a payout
    fn get_acc(env: Env, account: AccountId) -> u32;

    // The amount a recipient will receive during a payout is done with this method
    // It can be called multiple times for the same account, the value will be updated
    // This method requires the contract had been initialized already
    // This method can only be called by the admin of the contract
    fn set_acc(env: Env, account: AccountId, amount: u32);

    // With this method the invoker will deposit funds into the contract
    // Depositor needs to approve with the currency contract before calling this method
    // This method requires the contract had been initialized already
    fn deposit(env: Env, amount: u32);

    // This method starts the payout process to all accounts set in the contract
    // If there is not enough funds in the contract then it will throw an error
    // This method requires the contract had been initialized already
    // This method can only be called by the admin of the contract
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
            panic_with_error!(&env, Error::AlreadyInit);
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

    fn init_done(env: Env) {
        if env.data().has(STATE) == false {
            panic_with_error!(&env, Error::StateNotStarted);
        }
    }

    fn get_state(env: Env) -> State {
        env.data().get(STATE).unwrap().unwrap()
    }

    fn get_acc(env: Env, account: AccountId) -> u32 {
        env.data().get(account).unwrap().unwrap()
    }

    fn set_acc(env: Env, account: AccountId, amount: u32) {
        Self::init_done(env.clone());
        is_admin(&env);

        if BigInt::from_u32(&env, amount).le(&0) {
            panic_with_error!(&env, Error::AmountAtLeast1);
        }

        let mut current_state: State = env.data().get(STATE).unwrap().unwrap();
        if env.data().has(&account) {
            let current_amount: u32 = env.data().get(&account).unwrap().unwrap();
            current_state.total_amnt = current_state.total_amnt
              .sub(current_amount)
              .add(&amount);
        } else {
            add_recipient(&env, account.clone());
            current_state.total_acct = current_state.total_acct.add(1);
            current_state.total_amnt = current_state.total_amnt.add(&amount);
        }

        env.data().set(account, &amount);
        env.data().set(STATE, current_state);
    }

    fn deposit(env: Env, amount: u32) {
        Self::init_done(env.clone());
        let state = Self::get_state(env.clone());
        let currency_client = token::Client::new(&env, state.currency);
        let contract_id = Identifier::Contract(env.current_contract());

        currency_client.xfer_from(
            &Signature::Invoker,
            &BigInt::zero(&env),
            &Identifier::from(&env.invoker()),
            &contract_id,
            &BigInt::from_u32(&env, amount),
        );
    }

    fn payout(env: Env) {
        Self::init_done(env.clone());
        is_admin(&env);

        let state = Self::get_state(env.clone());
        let currency_client = token::Client::new(&env, state.currency);
        let amount_to_send = state.total_amnt;
        let contract_id = Identifier::Contract(env.current_contract());
        let amount_in_vault = currency_client.balance(&contract_id);

        if amount_in_vault < amount_to_send {
            panic_with_error!(&env, Error::VaultUnderfunded);
        }

        let recipients = get_recipients(&env);

        for recipient in recipients {
            let account_id = recipient.unwrap();
            let amount = Self::get_acc(env.clone(), account_id.clone());
            currency_client.xfer(
                &Signature::Invoker,
                &currency_client.nonce(&Signature::Invoker.identifier(&env)),
                &Identifier::Account(account_id.clone()),
                &BigInt::from_u32(&env, amount),
            );
        }
    }
}

fn is_admin(env: &Env) {
    let state: State = env.data().get(STATE).unwrap().unwrap();
    let invoker: Address = env.invoker();

    if invoker != state.admin {
        panic_with_error!(&env, Error::OnlyAdmin);
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