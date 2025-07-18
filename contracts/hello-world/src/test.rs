#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String};
extern crate std;
use std::println as ll;

#[test]
fn test_success1() {
  let env = Env::default();
  let contract_id = env.register(Contract, ());
  let client = ContractClient::new(&env, &contract_id);

  let words = client.hello(&String::from_str(&env, "John Doe"));
  assert_eq!(
    words,
    vec![
      &env,
      String::from_str(&env, "Hello"),
      String::from_str(&env, "John Doe"),
    ]
  );

  assert_eq!(client.increment(&3), 3);
  assert_eq!(client.increment(&2), 5);
  let state = client.get_state();
  assert_eq!(state.count, 5);

  assert_eq!(client.reset_count(&99), 99);
  let state = client.get_state();
  ll!("state.count: {:?}", state.count);
  //log!(&env, "state.count: {:?}", state.count);
  assert_eq!(state.count, 99);

  //assert_eq!(client.increment(&1), Error::LimitReached);
  //ll!("{}", env.logs().all().join("\n"));
}

#[test]
fn test_fail1() {
  let env = Env::default();
  let contract_id = env.register(Contract, ());
  let client = ContractClient::new(&env, &contract_id);
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
  let contract_id = env.register(Contract, ());
  let client = ContractClient::new(&env, &contract_id);

  assert_eq!(client.increment(&5), 5);
  let state = client.get_state();
  assert_eq!(state.count, 5);
  client.increment(&1);
}
#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_debugging() {
  let env = Env::default();
  let contract_id = env.register(Contract, ());
  let client = ContractClient::new(&env, &contract_id);
  client.debugging(&1);
}
