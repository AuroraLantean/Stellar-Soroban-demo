#![cfg(test)]

use super::*;
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;
//soroban-token-sdk
//use sep_41_token::Token;
use soroban_sdk::{
  testutils::{Address as TestAddr, Events},
  vec, Address, Env, IntoVal,
}; //Logs
extern crate std;
use std::println as ll;

fn new_token_ctrt<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
  let sac = e.register_stellar_asset_contract_v2(admin.clone());
  (
    token::Client::new(e, &sac.address()),
    token::StellarAssetClient::new(e, &sac.address()),
  )
}
fn new_ctrt(e: &Env) -> (Address, PredictionClient) {
  let contract_id = e.register(Prediction, ());
  let client = PredictionClient::new(e, &contract_id);
  (contract_id, client)
}

#[test]
fn test_token() {
  let env = Env::default();
  env.mock_all_auths();

  let admin = Address::generate(&env);
  let user1 = Address::generate(&env);
  let user2 = Address::generate(&env);
  let (token, asset) = new_token_ctrt(&env, &admin);
  let (contract_id, client) = new_ctrt(&env);

  asset.mint(&user1, &1000);
  asset.mint(&user2, &2000);

  assert_eq!(token.balance(&user1), 1000);
  assert_eq!(token.balance(&user2), 2000);
  assert_eq!(token.balance(&contract_id), 0);

  let user_id = symbol_short!("user1");
  let out1 = client.add_user(&user1, &user_id);
  assert_eq!(out1, 0);
  let user1u = client.get_user(&user1);
  assert_eq!(user1u.id, user_id);

  client.approve_token(&token.address, &user1, &700, &100);
  client.deposit_token(&token.address, &user1, &700);
  assert_eq!(token.balance(&user1), 300);
  assert_eq!(token.balance(&contract_id), 700);

  client.withdraw_token(&token.address, &user1, &500);
  assert_eq!(token.balance(&user1), 800);
  assert_eq!(token.balance(&contract_id), 200);

  let user1u = client.get_user(&user1);
  ll!("user1u: {:?}", user1u);
  assert_eq!(user1u.balance, 200);

  client.withdraw_token(&token.address, &user1, &200);
  client.delete_user(&user1);
  let user1u = client.get_user(&user1);
  ll!("user1u: {:?}", user1u);
  assert_eq!(user1u.updated_at, 0);
}
#[test]
fn test_get_state() {
  let env = Env::default();
  let (contract_id, client) = new_ctrt(&env);

  assert_eq!(client.increment(&3), 3);
  assert_eq!(
    env.events().all(),
    vec![
      &env,
      (
        contract_id.clone(),
        (symbol_short!("STATE"), symbol_short!("increment")).into_val(&env),
        3u32.into_val(&env)
      ),
    ]
  );
  assert_eq!(client.increment(&2), 5);
  let state = client.get_state();
  assert_eq!(state.count, 5);

  assert_eq!(client.reset_count(&99), 99);
  let state = client.get_state();
  ll!("state.count: {:?}", state.count);
  //log!(&env, "state.count: {:?}", state.count);
  assert_eq!(state.count, 99);

  //assert_eq!(client.increment(&1), Error::LimitReached);
}

#[test]
fn testf_max_count() {
  let env = Env::default();
  let (_, client) = new_ctrt(&env);
  ll!("test_fail1");
  //log!(&env, "state.count: {:?}", "John");
  assert_eq!(client.increment(&5), 5);
  assert_eq!(client.try_increment(&1), Err(Ok(Error::MaxCountReached)));
  ll!("{:?}", env.logs());
  //ll!("{}", env.logs().all().join("\n"));
}
#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn testf_max_count2() {
  let env = Env::default();
  let (_, client) = new_ctrt(&env);

  assert_eq!(client.increment(&5), 5);
  let state = client.get_state();
  assert_eq!(state.count, 5);
  client.increment(&1);
}
