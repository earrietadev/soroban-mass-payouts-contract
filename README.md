## Payouts to multiple accounts contract

Making payments to multiple accounts can be a boring task if we need to do everything manually and specially if that task repeats every month. With Soroban we can design a contract that lets us set the recipients and how much do we need to send them so the next month we will just need to deposit the total in the contract and tell the contract to distribute everything to each one of them... All done in a transparent and easy way.

> IMPORTANT: This is an example for educational purposes, it hasn't been audited, so it should not be used in production applications

## The contract

```rust
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
```

You can read the complete contract in the `src/lib.rs` file.

## Makefile

This repo includes a `Makefile` in order to help with the interaction with the contract, it has useful commands like `build`, `test` and `deploy_contract`. Check it out in order to see all of them.

### Steps to test it out locally

In order to test it locally you can do a series of commands which are already set up in order to make everything easier. We are going to be using the next Stellar Account for this:
```text
GBISQU3NHN2FMYTUTWMUYDILSQM6GNE7HMOSOHMFF6F46V7DEO543AVC
SCFDNKIBISHPK7ZDJDRWW3D3FAJU5FLI2BCUBEEVI4VY3CNPXP5C7XHH
```
In you terminal (with "make" package installed) do the following:
- `make build`
- `make initialize admin=5128536d3b745662749d994c0d0b9419e3349f3b1d271d852f8bcf57e323bbcd currency=ADD_THE_CURRENCY_CONTRACT_ID_HERE`
- `make get_state`

At this point you will get the current state saved in the contract! Now let's add a new recipient:
```text
GAA3UXD4DV7DBZ5KPVITW3SLWSV2PE25FYQC6ITRAHJR2VD7FR3XLN4J
SCZEKBSODMMKQVZWI6MNLQL7RT5WACMQJYCY645I67RJFRREE5ROZSHK
```
- `make set_acc admin=GBISQU3NHN2FMYTUTWMUYDILSQM6GNE7HMOSOHMFF6F46V7DEO543AVC account=01ba5c7c1d7e30e7aa7d513b6e4bb4aba7935d2e202f227101d31d547f2c7775 amount=2500`
- `make get_acc account=01ba5c7c1d7e30e7aa7d513b6e4bb4aba7935d2e202f227101d31d547f2c7775`

At this point you should receive the value `2500` and if you run `make get_state` you will see that now values `total_acct` and `total_amnt` are updated.

## Things to improve

If you would like to learn by improving this contract, here are a few things you could improve:

- Handle more currencies
- Prevent admin setting itself as a recipient (in case that's a requirement)
- Allow removing recipients
- Extend the Makefile file in order to make setting accounts easier
- Allow contracts to be recipients
- Be creative and add more stuff to it!

Also keep in mind that Soroban is still in development, this version of the contract targets the SDK 0.2.1 but things are already changing in 0.3.1 which is in development, so it's possible you will need to do a few changes if you want to use this contract with newer versions of the SDK.