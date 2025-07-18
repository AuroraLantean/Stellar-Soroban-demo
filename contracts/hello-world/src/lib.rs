#![no_std]
use soroban_sdk::{contract, contractimpl, log, symbol_short, vec, Env, String, Symbol, Vec};

const COUNT: Symbol = symbol_short!("COUNT");

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
  pub fn hello(env: Env, name: String) -> Vec<String> {
    let time = env.ledger().timestamp();
    log!(&env, "Name: {}. time: {}", name, time);
    vec![&env, String::from_str(&env, "Hello"), name]
  }
  pub fn increment(env: Env) -> u32 {
    let mut count: u32 = env.storage().instance().get(&COUNT).unwrap_or(0);
    log!(&env, "count: {}", count);

    count += 1;
    env.storage().instance().set(&COUNT, &count);
    env.storage().instance().extend_ttl(50, 100);
    count
  }
  pub fn reset_count(env: Env, value: u32) -> u32 {
    let mut count: u32 = env.storage().instance().get(&COUNT).unwrap_or(0);
    log!(&env, "count: {}", count);

    count = value;
    env.storage().instance().set(&COUNT, &count);
    env.storage().instance().extend_ttl(50, 100);
    count
  }
  pub fn get_count(env: Env) -> u32 {
    env.storage().instance().get(&COUNT).unwrap_or(0)
  }
}

mod test;
