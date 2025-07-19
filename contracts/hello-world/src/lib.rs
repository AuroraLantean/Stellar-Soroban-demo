#![no_std]
use soroban_sdk::{
  contract, contracterror, contractimpl, contracttype, log, panic_with_error, symbol_short, vec,
  Env, String, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
  LimitReached = 1,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
  pub count: u32,
  pub last_incr: u32,
}
const STATE: Symbol = symbol_short!("STATE");
const MAX_COUNT: u32 = 5;

#[contracttype]
pub enum Registry {
  User(Symbol),
}
//TODO: simpleAccount
#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
  pub fn hello(env: Env, name: Symbol) {
    let time = env.ledger().timestamp();
    log!(&env, "time: {}", time);
    //vec![&env, String::from_str(&env, "Hello"), nameString] // -> Vec<String>
    log!(&env, "Hello {}", name);
  }
  pub fn increment(env: Env, incr: u32) -> Result<u32, Error> {
    let mut state = Self::get_state(env.clone());
    log!(&env, "increment: {}", state);
    state.count += incr;
    state.last_incr = incr;
    if state.count <= MAX_COUNT {
      env.storage().instance().set(&STATE, &state);
      env.storage().instance().extend_ttl(50, 100);
      env
        .events()
        .publish((STATE, symbol_short!("increment")), state.count);
      Ok(state.count)
    } else {
      Err(Error::LimitReached)
    }
  }
  pub fn debugging(env: Env, value: u32) -> u32 {
    match value {
      0 => 0,
      _ => {
        log!(&env, "fail");
        panic_with_error!(&env, Error::LimitReached);
      }
    }
  }
  pub fn reset_count(env: Env, value: u32) -> Result<u32, Error> {
    let mut state = Self::get_state(env.clone());
    log!(&env, "reset_count: {}", state);

    state.count = value;
    state.last_incr = value;
    env.storage().instance().set(&STATE, &state);
    env.storage().instance().extend_ttl(50, 100);
    Ok(state.count)
  }
  pub fn get_state(env: Env) -> State {
    env.storage().instance().get(&STATE).unwrap_or(State {
      count: 0,
      last_incr: 0,
    }) // If no value set, assume 0.
  }
}

mod test;
