#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Ledger;
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;
//soroban-token-sdk
//use sep_41_token::Token;
use soroban_sdk::{
  testutils::{Address as TestAddr, Events},
  vec, Address, Env, IntoVal,
}; //Logs
extern crate std;
use core::fmt::Debug;
use std::println as ll;
fn llc<T: Debug>(name: &str, input: T) {
  ll!("\x1b[32m {}: {:?}\x1b[0m", name, input);
}
const INITBALC: i128 = 9_000_i128;

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
  let ctrt_addr = e.register(Prediction, (admin, token, market_name));
  let client = PredictionClient::new(e, &ctrt_addr);
  (ctrt_addr, client)
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
  Address,
  Address,
) {
  let admin = Address::generate(&env);
  let user1 = Address::generate(&env);
  let user2 = Address::generate(&env);
  let user3 = Address::generate(&env);
  let vault = Address::generate(&env);
  let (token, asset) = new_token_ctrt(&env, &admin);
  let token_id = token.address.clone();
  let market_name = String::from_str(&env, "prediction");
  let (ctrt_addr, ctrt) = new_ctrt(&env, admin.clone(), token_id.clone(), market_name);
  env.mock_all_auths();

  asset.mint(&admin, &INITBALC);
  asset.mint(&user1, &INITBALC);
  asset.mint(&user2, &INITBALC);
  asset.mint(&user3, &INITBALC);
  token.approve(&user1, &ctrt_addr, &INITBALC, &100);
  token.approve(&user2, &ctrt_addr, &INITBALC, &100);
  token.approve(&user3, &ctrt_addr, &INITBALC, &100);
  (
    ctrt, ctrt_addr, token, token_id, admin, user1, user2, user3, vault,
  )
}
#[test]
fn test_init_conditions() {
  let env = Env::default();
  let (ctrt, ctrt_addr, token, token_id, admin, user1, user2, user3, _) = setup(&env);

  let state = ctrt.get_state();
  ll!("state: {:?}", state);
  assert_eq!(state.count, 0);
  assert_eq!(state.admin, admin);
  assert_eq!(state.token, token_id);
  assert_eq!(state.market_name, String::from_str(&env, "prediction"));
  assert_eq!(state.status, Status::Active);
  assert_eq!(token.balance(&user1), INITBALC);
  assert_eq!(token.balance(&user2), INITBALC);
  assert_eq!(token.balance(&user3), INITBALC);
  assert_eq!(token.allowance(&user1, &ctrt_addr), INITBALC);
  assert_eq!(token.allowance(&user2, &ctrt_addr), INITBALC);
  assert_eq!(token.allowance(&user3, &ctrt_addr), INITBALC);
  assert_eq!(token.balance(&ctrt_addr), 0);

  token.transfer(&user1, &user2, &137);
  assert_eq!(token.balance(&user1), INITBALC - 137);
}
#[test]
fn test_token() {
  let env = Env::default();
  let (ctrt, ctrt_addr, token, token_id, _admin, user1, _user2, _, _) = setup(&env);

  let user_id = symbol_short!("user1");
  let out1 = ctrt.add_user(&user1, &user_id);
  assert_eq!(out1, 0);
  let user1u = ctrt.get_user(&user1);
  assert_eq!(user1u.id, user_id);

  ctrt.deposit_token(&token_id, &user1, &700);
  assert_eq!(token.balance(&user1), INITBALC - 700);
  assert_eq!(token.balance(&ctrt_addr), 700);

  ctrt.withdraw_token(&token_id, &user1, &500);
  assert_eq!(token.balance(&user1), INITBALC - 200);
  assert_eq!(token.balance(&ctrt_addr), 200);

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
fn test_game() {
  let env = Env::default();
  let (ctrt, ctrt_addr, token, _token_id, admin, user1, user2, user3, vault) = setup(&env);
  let game_id = 1u32;
  let time_start = 0u64;
  let time_end = 100u64;
  let commission_rate = 10u128; // 10 as 1%
  let game = ctrt.get_game(&game_id);
  llc("game", game);
  ctrt.set_game(&admin, &game_id, &time_start, &time_end, &commission_rate);

  let bet_idx1 = 0u32;
  let value1 = 100u128;
  let game = ctrt.get_game(&game_id);
  let game_value = game.clone().unwrap().values.get(bet_idx1).unwrap();
  let game_number = game.clone().unwrap().numbers.get(bet_idx1).unwrap();
  llc("game", game);

  let bet = ctrt.get_bet(&user1, &game_id);
  llc("bet", bet.clone());
  let bet_value = 0;

  //user1 bets
  llc("user1 to bet", value1);
  ctrt.bet(&user1, &game_id, &value1, &bet_idx1);
  let bet = ctrt.get_bet(&user1, &game_id);
  llc("user1's bet", bet.clone());
  assert_eq!(
    bet_value + value1,
    bet.unwrap().bet_values.get(bet_idx1).unwrap()
  );
  //check game value
  let game = ctrt.get_game(&game_id);
  llc("game", game.clone());
  assert_eq!(
    game_value + value1,
    game.clone().unwrap().values.get(bet_idx1).unwrap()
  );
  //check game number
  assert_eq!(
    game_number + 1,
    game.unwrap().numbers.get(bet_idx1).unwrap()
  );

  //check contract balance
  let balc1 = token.balance(&ctrt_addr);
  llc("ctrt balc:", balc1.clone());
  assert_eq!(balc1.cast_unsigned(), value1);

  //user2 bets on losing
  let bet_idx2 = 3u32;
  let value2 = 237u128;
  llc("user2 to bet", value2);
  ctrt.bet(&user2, &game_id, &value2, &bet_idx2);
  let bet = ctrt.get_bet(&user2, &game_id);
  llc("user2's bet", bet.clone());

  //user1 bets again on winning
  let bet_idx1 = 0u32;
  let value1 = 248u128;
  llc("user1 to bet", value1);
  ctrt.bet(&user1, &game_id, &value1, &bet_idx1);
  let bet = ctrt.get_bet(&user1, &game_id);
  llc("user1's bet", bet.clone());

  //user3 bets on winning
  let bet_idx3 = 0u32;
  let value3 = 569u128;
  llc("user3 to bet", value3);
  ctrt.bet(&user3, &game_id, &value3, &bet_idx3);
  let bet = ctrt.get_bet(&user3, &game_id);
  llc("user3's bet", bet.clone());

  //time travel to the end time
  env.ledger().set_timestamp(100);

  //check vault balance
  let balc = token.balance(&vault);
  llc("vault balc b4:", balc.clone());
  //game admin to settle
  let outcome: Vec<u32> = vec![&env, 1, 0, 0, 0];
  llc("admin to settle with commission rate", commission_rate);
  ctrt.settle(&admin, &game_id, &outcome, &vault);
  let game = ctrt.get_game(&game_id);
  llc("game", game.clone());

  let balc = token.balance(&vault);
  llc("vault balc Af:", balc.clone());

  let balc1bf = token.balance(&user1);
  llc("user1 balc B4:", balc1bf.clone());
  //user1 to claim
  ctrt.claim(&user1, &game_id);
  let balc1af = token.balance(&user1);
  llc("user1 balc Af:", balc1af.clone());
  llc("user1 balc change:", balc1af - balc1bf);

  //user2 to claim
  assert_eq!(
    ctrt.try_claim(&user2, &game_id),
    Err(Ok(Error::UserClaimsZero))
  );
}
#[test]
fn test_state() {
  let env = Env::default();
  let (ctrt, ctrt_addr, _, _, _, _, _, _, _) = setup(&env);

  let state = ctrt.get_state();
  ll!("state: {:?}", state);
  assert_eq!(state.count, 0);

  assert_eq!(ctrt.increment(&3), 3);
  assert_eq!(
    env.events().all(),
    vec![
      &env,
      (
        ctrt_addr.clone(),
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
  let (ctrt, _, _, _, _, _, _, _, _) = setup(&env);

  ll!("testf_max_count");
  //log!(&env, "state.count: {:?}", "John");
  assert_eq!(ctrt.increment(&5), 5);
  assert_eq!(ctrt.try_increment(&1), Err(Ok(Error::MaxCountReached)));
  ll!("{:?}", env.logs());
  //ll!("{}", env.logs().all().join("\n"));
}
#[test]
#[should_panic(expected = "HostError: Error(Contract, #43)")] //depending on Error enum index
fn testf_max_count2() {
  let env = Env::default();
  let (ctrt, _, _, _, _, _, _, _, _) = setup(&env);

  assert_eq!(ctrt.increment(&5), 5);
  let state = ctrt.get_state();
  assert_eq!(state.count, 5);
  ctrt.increment(&1);
}
