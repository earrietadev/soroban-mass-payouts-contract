#![cfg(test)]

use super::*;
use soroban_sdk::{IntoVal, testutils::Accounts};
use token::{Client as TokenClient, TokenMetadata};
use soroban_auth::Identifier;

extern crate std;

struct TestData {
  env: Env,
  token_admin: AccountId,
  usdc_id: BytesN<32>,
  usdc_token_client: TokenClient,
  contract_id: BytesN<32>,
  contract_client: MassPayoutContractClient,
  admin: AccountId,
  admin_id: Address,
}

fn create_test_basic_data() -> TestData {
  let env = Env::default();

  // First set up the token contract
  let token_admin = env.accounts().generate();
  let (usdc_id, usdc_token_client) = create_token_contract(&env, &token_admin);

  // Initialize the contract
  let contract_id = env.register_contract(None, MassPayoutContract);
  let contract_client = MassPayoutContractClient::new(&env, &contract_id);
  let admin = env.accounts().generate();
  let admin_id = Address::Account(admin.clone());

  TestData {
    env,
    token_admin,
    usdc_id,
    usdc_token_client,
    contract_id,
    contract_client,
    admin,
    admin_id,
  }
}

fn create_token_contract(e: &Env, admin: &AccountId) -> (BytesN<32>, TokenClient) {
  let id = e.register_contract_token(None);
  let token = TokenClient::new(e, &id);

  token.init(
    &Identifier::Account(admin.clone()),
    &TokenMetadata {
      name: "USD Stablecoin".into_val(e),
      symbol: "USDC".into_val(e),
      decimals: 7,
    },
  );
  (id, token)
}

#[test]
fn test_initialize_of_contract() {
  let test_data = create_test_basic_data();

  test_data.contract_client.initialize(&test_data.admin_id, &test_data.usdc_id);

  let contract_state: State = test_data.contract_client.get_state();

  assert_eq!(contract_state.total_amnt, 0);
  assert_eq!(contract_state.total_acct, 0);
  assert_eq!(contract_state.admin, test_data.admin_id);
  assert_eq!(contract_state.currency, test_data.usdc_id);
}

#[test]
#[should_panic]
fn test_initialize_once_only() {
  let test_data = create_test_basic_data();

  test_data.contract_client.initialize(&test_data.admin_id, &test_data.usdc_id);
  test_data.contract_client.initialize(&test_data.admin_id, &test_data.usdc_id);
}

#[test]
fn test_set_account() {
  let test_data = create_test_basic_data();

  test_data.contract_client.initialize(&test_data.admin_id, &test_data.usdc_id);

  let new_recipient = test_data.env.accounts().generate();
  let amount_to_receive = 2500;
  let mut saved_stated: State = test_data.contract_client.get_state();

  assert_eq!(saved_stated.total_acct, 0);
  assert_eq!(saved_stated.total_amnt, 0);

  test_data.contract_client
    .with_source_account(&test_data.admin)
    .set_acc(&new_recipient, &amount_to_receive);

  let mut amount_saved: i128 = test_data.contract_client.get_acc(&new_recipient);
  saved_stated = test_data.contract_client.get_state();
  assert_eq!(amount_to_receive.clone(), amount_saved);
  assert_eq!(saved_stated.total_acct, 1);
  assert_eq!(saved_stated.total_amnt, amount_saved);

  // We call it a second time with same user in order to test that we are correctly updating the amounts
  test_data.contract_client
    .with_source_account(&test_data.admin)
    .set_acc(&new_recipient, &(amount_to_receive * 2));

  amount_saved = test_data.contract_client.get_acc(&new_recipient);
  saved_stated = test_data.contract_client.get_state();
  assert_eq!(amount_saved, (amount_to_receive * 2));
  assert_eq!(saved_stated.total_acct, 1);
  assert_eq!(saved_stated.total_amnt, (amount_to_receive * 2));

  // We use an extra user to verify we are adding extra accounts and extra amoutns correctly
  let new_recipient_2 = test_data.env.accounts().generate();
  test_data.contract_client
    .with_source_account(&test_data.admin)
    .set_acc(&new_recipient_2, &amount_to_receive);

  amount_saved = test_data.contract_client.get_acc(&new_recipient_2);
  saved_stated = test_data.contract_client.get_state();
  assert_eq!(amount_saved, amount_to_receive);
  assert_eq!(saved_stated.total_acct, 2);
  assert_eq!(saved_stated.total_amnt, (amount_to_receive * 3));
}

#[test]
#[should_panic]
fn test_auth_admin() {
  let test_data = create_test_basic_data();

  test_data.contract_client.initialize(&test_data.admin_id, &test_data.usdc_id);

  // This should work
  let acc_a = test_data.env.accounts().generate();
  let amount_a = 100;
  test_data.contract_client
    .with_source_account(&test_data.admin)
    .set_acc(&acc_a, &amount_a);

  // This should fail
  let random = test_data.env.accounts().generate();
  let acc_b = test_data.env.accounts().generate();
  let amount_b = 100;
  test_data.contract_client
    .with_source_account(&random)
    .set_acc(&acc_b, &amount_b);
}

#[test]
#[should_panic]
fn test_amount_must_be_1_at_least() {
  let test_data = create_test_basic_data();

  test_data.contract_client.initialize(&test_data.admin_id, &test_data.usdc_id);

  let new_recipient = test_data.env.accounts().generate();
  let amount_to_receive = -1;

  test_data.contract_client
    .with_source_account(&test_data.admin)
    .set_acc(&new_recipient, &amount_to_receive);
}

#[test]
fn test_deposit() {
  let test_data = create_test_basic_data();

  test_data.contract_client.initialize(&test_data.admin_id, &test_data.usdc_id);

  let amount_to_deposit: i128 = 100000;

  // Mint USDC tokens to the depositor (in this case the admin but the contract doesn't care who deposits)
  test_data.usdc_token_client
    .with_source_account(&test_data.token_admin)
    .mint(
      &Signature::Invoker,
      &0,
      &Identifier::Account(test_data.admin.clone()),
      &(amount_to_deposit * 2),
    );

  assert_eq!(
    test_data.usdc_token_client.balance(&Identifier::Account(test_data.admin.clone())),
    (amount_to_deposit * 2)
  );

  test_data.usdc_token_client
    .with_source_account(&test_data.admin)
    .approve(
      &Signature::Invoker,
      &0,
      &Identifier::Contract(test_data.contract_id.clone()),
      &amount_to_deposit,
    );

  test_data.contract_client
    .with_source_account(&test_data.admin)
    .deposit(&amount_to_deposit);

  assert_eq!(
    test_data.usdc_token_client.balance(&Identifier::Account(test_data.admin.clone())),
    amount_to_deposit
  );

  assert_eq!(
    test_data.usdc_token_client.balance(&Identifier::Contract(test_data.contract_id.clone())),
    amount_to_deposit
  );
}

#[test]
#[should_panic]
fn test_payout_not_enough_funds() {
  let test_data = create_test_basic_data();
  test_data.contract_client.initialize(&test_data.admin_id, &test_data.usdc_id);
  let new_recipient = test_data.env.accounts().generate();
  test_data.contract_client.set_acc(&new_recipient, &100);
  test_data.contract_client.payout();
}

#[test]
fn test_payout() {
  let test_data = create_test_basic_data();
  test_data.contract_client.initialize(&test_data.admin_id, &test_data.usdc_id);

  let recipient_1 = test_data.env.accounts().generate();
  let recipient_amount_1 = 5000;
  let recipient_2 = test_data.env.accounts().generate();
  let recipient_amount_2 = 2450;
  let recipient_3 = test_data.env.accounts().generate();
  let recipient_amount_3 = 1800;

  test_data.contract_client
    .with_source_account(&test_data.admin)
    .set_acc(&recipient_1, &recipient_amount_1);

  test_data.contract_client
    .with_source_account(&test_data.admin)
    .set_acc(&recipient_2, &recipient_amount_2);

  test_data.contract_client
    .with_source_account(&test_data.admin)
    .set_acc(&recipient_3, &recipient_amount_3);

  let admin_funds: i128 = 10000;
  test_data.usdc_token_client
    .with_source_account(&test_data.token_admin)
    .mint(
      &Signature::Invoker,
      &0,
      &Identifier::Account(test_data.admin.clone()),
      &admin_funds,
    );

  assert_eq!(
    test_data.usdc_token_client.balance(&Identifier::Account(test_data.admin.clone())),
    admin_funds
  );

  test_data.usdc_token_client
    .with_source_account(&test_data.admin)
    .approve(
      &Signature::Invoker,
      &0,
      &Identifier::Contract(test_data.contract_id.clone()),
      &(admin_funds + 1),
    );

  test_data.contract_client
    .with_source_account(&test_data.admin)
    .deposit(&admin_funds);

  assert_eq!(
    test_data.usdc_token_client.balance(&Identifier::Account(test_data.admin.clone())),
    0 as i128
  );

  assert_eq!(
    test_data.usdc_token_client.balance(&Identifier::Contract(test_data.contract_id.clone())),
    admin_funds
  );

  test_data.contract_client
    .with_source_account(&test_data.admin)
    .payout();

  assert_eq!(
    test_data.usdc_token_client.balance(&Identifier::Contract(test_data.contract_id.clone())),
    (admin_funds - recipient_amount_1 - recipient_amount_2 - recipient_amount_3)
  );

  assert_eq!(
    test_data.usdc_token_client.balance(&Identifier::Account(recipient_1.clone())),
    recipient_amount_1
  );

  assert_eq!(
    test_data.usdc_token_client.balance(&Identifier::Account(recipient_2.clone())),
    recipient_amount_2
  );

  assert_eq!(
    test_data.usdc_token_client.balance(&Identifier::Account(recipient_3.clone())),
    recipient_amount_3
  );
}
