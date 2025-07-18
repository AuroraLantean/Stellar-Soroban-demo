#![no_std]
use soroban_sdk::{
  contract, contractimpl, contracttype, log, symbol_short, vec, Env, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
  pub count: u32,
  pub last_incr: u32,
}
const STATE: Symbol = symbol_short!("STATE");

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
  pub fn hello(env: Env, name: String) -> Vec<String> {
    let time = env.ledger().timestamp();
    log!(&env, "Name: {}. time: {}", name, time);
    vec![&env, String::from_str(&env, "Hello"), name]
  }
  pub fn increment(env: Env, incr: u32) -> u32 {
    let mut state = Self::get_state(env.clone());
    log!(&env, "state: {}", state);
    state.count += incr;
    state.last_incr = incr;

    env.storage().instance().set(&STATE, &state);
    env.storage().instance().extend_ttl(50, 100);
    state.count
  }

  pub fn reset_count(env: Env, value: u32) -> u32 {
    let mut state = Self::get_state(env.clone());
    log!(&env, "state: {}", state);

    state.count = value;
    state.last_incr = value;
    env.storage().instance().set(&STATE, &state);
    env.storage().instance().extend_ttl(50, 100);
    state.count
  }
  pub fn get_state(env: Env) -> State {
    env.storage().instance().get(&STATE).unwrap_or(State {
      count: 0,
      last_incr: 0,
    }) // If no value set, assume 0.
  }
}

mod test;
