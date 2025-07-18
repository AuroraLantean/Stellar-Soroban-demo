#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String};

#[test]
fn test() {
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
  assert_eq!(state.count, 99);
}
