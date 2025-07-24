#![no_std]
use soroban_sdk::{
  contract, contractimpl, contracttype, log, panic_with_error, symbol_short, token, vec, Address,
  Env, String, Symbol, Vec,
};

use crate::err::Error;
mod err;
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
  Users(Address),
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct User {
  pub addr: Address,
  pub id: Symbol,
  pub balance: u128,
  pub updated_at: u64,
}
//TODO: simpleAccount
#[contract]
pub struct Hello;

#[contractimpl]
impl Hello {
  pub fn approve_token(
    env: Env,
    token: Address,
    sender: Address,
    amount: u128,
    expiration_ledger: u32,
  ) {
    log!(&env, "approve_token");
    sender.require_auth();
    let token = token::Client::new(&env, &token);
    let ctrt_addr = env.current_contract_address();

    let amount_i128 = amount.try_into().unwrap();
    token.approve(&sender, &ctrt_addr, &amount_i128, &expiration_ledger);
  }
  pub fn deposit_token(
    env: Env,
    token: Address,
    sender: Address,
    amount: u128,
  ) -> Result<u32, Error> {
    log!(&env, "deposit_token");
    sender.require_auth(); // Check if the caller  == sender argument

    let token = token::Client::new(&env, &token);
    let ctrt_addr = env.current_contract_address();

    let amount_i128 = amount.try_into().unwrap();
    let sender_balance = token.balance(&sender);
    if sender_balance < amount_i128 {
      return Err(Error::InsufficientBalance);
    }
    let allowance = token.allowance(&sender, &ctrt_addr);
    if allowance < amount_i128 {
      return Err(Error::InsufficientAllowance);
    }

    token.transfer_from(&ctrt_addr, &sender, &ctrt_addr, &amount_i128);
    Ok(0u32)
  }
  pub fn withdraw_token(
    env: Env,
    token: Address,
    sender: Address,
    amount: u128,
  ) -> Result<u32, Error> {
    log!(&env, "withdraw_token");
    sender.require_auth(); // Check if the caller  == sender argument

    let token = token::Client::new(&env, &token);
    let ctrt_addr = env.current_contract_address();

    let amount_i128 = amount.try_into().unwrap();
    let sender_balance = token.balance(&ctrt_addr);
    if sender_balance < amount_i128 {
      return Err(Error::InsufficientBalance);
    }
    token.transfer(&ctrt_addr, &sender, &amount_i128);
    Ok(0u32)
  }
  pub fn balance_allowance(
    env: Env,
    token: Address,
    target: Address,
  ) -> Result<(i128, i128), Error> {
    log!(&env, "balance_allowance");
    let token = token::Client::new(&env, &token);
    let ctrt_addr = env.current_contract_address();
    let balance = token.balance(&target);
    let allowance = token.allowance(&target, &ctrt_addr);
    Ok((balance, allowance))
  }

  pub fn get_user(env: Env, addr: Address) -> User {
    log!(&env, "get_user");
    let registry = Registry::Users(addr.clone());
    env.storage().instance().get(&registry).unwrap_or(User {
      addr: addr,
      id: symbol_short!("none"),
      balance: 0,
      updated_at: 0,
    })
  }
  pub fn add_user(env: Env, addr: Address, id: Symbol) -> Result<u32, Error> {
    log!(&env, "add_user");
    let mut user = Self::get_user(env.clone(), addr.clone());
    if user.updated_at != 0 {
      return Err(Error::UserExists);
    }
    user.id = id;
    user.updated_at = env.ledger().timestamp();
    //log!(&env, "user:{:?}", user);
    //log!(&env, "timestamp:{:?}", env.ledger().timestamp());
    env.storage().instance().set(&Registry::Users(addr), &user);
    Ok(0u32)
  }
  pub fn delete_user(env: Env, addr: Address, id: Symbol) -> Result<u32, Error> {
    let mut user = Self::get_user(env.clone(), addr.clone());
    if user.updated_at == 0 {
      return Err(Error::UserDoesNotExists);
    }
    user.id = symbol_short!("none");
    user.balance = 0;
    user.updated_at = 0;
    env.storage().instance().set(&Registry::Users(addr), &user);
    Ok(0u32)
  }
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
