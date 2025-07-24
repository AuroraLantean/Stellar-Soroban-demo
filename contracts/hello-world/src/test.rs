#![cfg(test)]

use super::*;
//use crate::token::TokenClient;
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;
//soroban-token-sdk
//use sep_41_token::Token;
//use soroban_sdk::token::TokenClient;
//use crate::{contract::Token, TokenClient};
use soroban_sdk::{
  testutils::{self, Address as TestAddr, AuthorizedFunction, AuthorizedInvocation, Events},
  vec, Address, Env, FromVal, IntoVal, String,
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
fn new_hello(e: &Env) -> (Address, HelloClient) {
  let contract_id = e.register(Hello, ());
  let client = HelloClient::new(e, &contract_id);
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
  let (contract_id, client) = new_hello(&env);

  asset.mint(&user1, &1000);
  asset.mint(&user2, &2000);

  assert_eq!(token.balance(&user1), 1000);
  assert_eq!(token.balance(&user2), 2000);
  assert_eq!(token.balance(&contract_id), 0);
}
#[test]
fn test_success1() {
  let env = Env::default();
  let (contract_id, client) = new_hello(&env);

  client.hello(&symbol_short!("Dev"));
  //let logs = env.logs().all();
  //std::println!("logs: {}", logs.join("\n"));
  //assert_eq!(logs, std::vec!["Hello Dev"]);
  /*let words = client.hello(&String::from_str(&env, "John Doe"));
  assert_eq!(
    words,
    vec![
      &env,
      String::from_str(&env, "Hello"),
      String::from_str(&env, "John Doe"),
    ]
  );*/

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
fn test_user() {
  let env = Env::default();
  let (_, client) = new_hello(&env);

  let addr1 = Address::generate(&env);

  let user_out = client.get_user(&addr1);
  ll!("user_out: {:?}", user_out);

  let adam_id = symbol_short!("adam_id");
  let out1 = client.add_user(&addr1, &adam_id);
  assert_eq!(out1, 0);
  let adam_user2 = client.get_user(&addr1);
  ll!("adam_user2: {:?}", adam_user2);
  assert_eq!(adam_user2.id, adam_id);
}
#[test]
fn test_fail1() {
  let env = Env::default();
  let (_, client) = new_hello(&env);
  ll!("test_fail1");
  //log!(&env, "state.count: {:?}", "John");
  assert_eq!(client.increment(&5), 5);
  assert_eq!(client.try_increment(&1), Err(Ok(Error::LimitReached)));
  ll!("{:?}", env.logs());
  //ll!("{}", env.logs().all().join("\n"));
}
#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_fail2() {
  let env = Env::default();
  let (_, client) = new_hello(&env);

  assert_eq!(client.increment(&5), 5);
  let state = client.get_state();
  assert_eq!(state.count, 5);
  client.increment(&1);
}
#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_debugging() {
  let env = Env::default();
  let (_, client) = new_hello(&env);
  client.debugging(&1);
}
