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

  assert_eq!(client.increment(), 1);
  assert_eq!(client.increment(), 2);
  let count = client.get_count();
  assert_eq!(count, 2);

  assert_eq!(client.reset_count(&99), 99);
  let count = client.get_count();
  assert_eq!(count, 99);
}
