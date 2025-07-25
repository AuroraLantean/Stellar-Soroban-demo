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
fn new_ctrt(
  e: &Env,
  admin: Address,
  token: Address,
  market_name: String,
) -> (Address, PredictionClient) {
  let contract_id = e.register(Prediction, (admin, token, market_name));
  let client = PredictionClient::new(e, &contract_id);
  (contract_id, client)
}
fn setup(
  env: &Env,
) -> (
  PredictionClient,
  Address,
  TokenClient,
  Address,
  Address,
  Address,
  Address,
) {
  let admin = Address::generate(&env);
  let user1 = Address::generate(&env);
  let user2 = Address::generate(&env);
  let (token, asset) = new_token_ctrt(&env, &admin);
  let token_id = token.address.clone();
  let market_name = String::from_str(&env, "prediction");
  let (ctrt_id, ctrt) = new_ctrt(&env, admin.clone(), token_id.clone(), market_name);
  env.mock_all_auths();

  let init_balc = 1_000_i128;
  asset.mint(&user1, &init_balc);
  asset.mint(&user2, &init_balc);
  token.approve(&user1, &ctrt_id, &init_balc, &100);
  token.approve(&user2, &ctrt_id, &init_balc, &100);
  (ctrt, ctrt_id, token, token_id, admin, user1, user2)
}
#[test]
fn test_init_conditions() {
  let env = Env::default();
  let (ctrt, ctrt_id, token, token_id, admin, user1, user2) = setup(&env);

  let state = ctrt.get_state();
  ll!("state: {:?}", state);
  assert_eq!(state.count, 0);
  assert_eq!(state.admin, admin);
  assert_eq!(state.token, token_id);
  assert_eq!(state.market_name, String::from_str(&env, "prediction"));
  assert_eq!(state.status, Status::Initial);

  assert_eq!(token.balance(&user1), 1000);
  assert_eq!(token.balance(&user2), 1000);
  assert_eq!(token.balance(&ctrt_id), 0);
}
#[test]
fn test_token() {
  let env = Env::default();
  let (ctrt, ctrt_id, token, token_id, _admin, user1, user2) = setup(&env);

  let user_id = symbol_short!("user1");
  let out1 = ctrt.add_user(&user1, &user_id);
  assert_eq!(out1, 0);
  let user1u = ctrt.get_user(&user1);
  assert_eq!(user1u.id, user_id);

  ctrt.deposit_token(&token_id, &user1, &700);
  assert_eq!(token.balance(&user1), 300);
  assert_eq!(token.balance(&ctrt_id), 700);

  ctrt.withdraw_token(&token_id, &user1, &500);
  assert_eq!(token.balance(&user1), 800);
  assert_eq!(token.balance(&ctrt_id), 200);

  let user1u = ctrt.get_user(&user1);
  ll!("user1u: {:?}", user1u);
  assert_eq!(user1u.balance, 200);

  ctrt.withdraw_token(&token_id, &user1, &200);
  ctrt.delete_user(&user1);
  let user1u = ctrt.get_user(&user1);
  ll!("user1u: {:?}", user1u);
  assert_eq!(user1u.updated_at, 0);
}
#[test]
fn test_state() {
  let env = Env::default();
  let (ctrt, ctrt_id, _, _, _, _, _) = setup(&env);

  let state = ctrt.get_state();
  ll!("state: {:?}", state);
  assert_eq!(state.count, 0);

  assert_eq!(ctrt.increment(&3), 3);
  assert_eq!(
    env.events().all(),
    vec![
      &env,
      (
        ctrt_id.clone(),
        (symbol_short!("STATE"), symbol_short!("increment")).into_val(&env),
        3u32.into_val(&env)
      ),
    ]
  );
  assert_eq!(ctrt.increment(&2), 5);
  let state = ctrt.get_state();
  assert_eq!(state.count, 5);

  assert_eq!(ctrt.reset_count(&99), 99);
  let state = ctrt.get_state();
  ll!("state.count: {:?}", state.count);
  //log!(&env, "state.count: {:?}", state.count);
  assert_eq!(state.count, 99);

  //assert_eq!(ctrt.increment(&1), Error::LimitReached);
}

#[test]
fn testf_max_count() {
  let env = Env::default();
  let (ctrt, _, _, _, _, _, _) = setup(&env);

  ll!("testf_max_count");
  //log!(&env, "state.count: {:?}", "John");
  assert_eq!(ctrt.increment(&5), 5);
  assert_eq!(ctrt.try_increment(&1), Err(Ok(Error::MaxCountReached)));
  ll!("{:?}", env.logs());
  //ll!("{}", env.logs().all().join("\n"));
}
#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn testf_max_count2() {
  let env = Env::default();
  let (ctrt, _, _, _, _, _, _) = setup(&env);

  assert_eq!(ctrt.increment(&5), 5);
  let state = ctrt.get_state();
  assert_eq!(state.count, 5);
  ctrt.increment(&1);
}
