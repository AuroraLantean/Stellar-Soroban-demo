#![no_std]
use sep_41_token::TokenClient;
use soroban_sdk::{contract, contractimpl, log, symbol_short, token, Address, Env, Symbol};

use crate::types::{Error, Registry, State, User, MAX_COUNT, STATE};
mod types;

//TODO: simpleAccount
#[contract]
pub struct Prediction;

#[contractimpl]
impl Prediction {
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
    log!(&env, "about to get_user");
    let mut user = Self::get_user(env.clone(), sender.clone())?;

    user.balance += amount;
    user.updated_at = env.ledger().timestamp();
    //log!(&env, "user:{:?}", user);
    let key = Registry::Users(sender.clone());
    env.storage().persistent().set(&key, &user);
    env
      .storage()
      .persistent()
      .extend_ttl(&key, 50, env.storage().max_ttl());

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
    log!(&env, "about to get_user");
    let mut user = Self::get_user(env.clone(), sender.clone())?;

    user.balance -= amount;
    user.updated_at = env.ledger().timestamp();
    //log!(&env, "user:{:?}", user);
    let key = Registry::Users(sender.clone());
    env.storage().persistent().set(&key, &user);

    env
      .storage()
      .persistent()
      .extend_ttl(&key, 50, env.storage().max_ttl());

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

  pub fn get_user(env: Env, addr: Address) -> Result<User, Error> {
    log!(&env, "get_user");
    let key = Registry::Users(addr.clone());
    env
      .storage()
      .persistent()
      .get(&key)
      .unwrap_or(Err(Error::UserDoesNotExist))
  }
  pub fn add_user(env: Env, addr: Address, id: Symbol) -> Result<u32, Error> {
    log!(&env, "add_user");
    let key = Registry::Users(addr.clone());
    let user_opt: Option<User> = env.storage().persistent().get(&key);
    if user_opt.is_some() {
      return Err(Error::UserExists);
    }
    let user = User {
      addr: addr.clone(),
      id: id,
      balance: 0,
      updated_at: env.ledger().timestamp(),
    };
    //log!(&env, "user:{:?}", user);
    env.storage().persistent().set(&key, &user);
    Ok(0u32)
  }
  pub fn delete_user(env: Env, addr: Address) -> Result<u32, Error> {
    let user = Self::get_user(env.clone(), addr.clone())?;
    if user.balance > 0 {
      return Err(Error::BalanceExists);
    }
    env.storage().persistent().remove(&user);
    Ok(0u32)
  }

  pub fn increment(env: Env, incr: u32) -> Result<u32, Error> {
    let mut state = Self::get_state(env.clone());
    log!(&env, "increment: {}", state);
    state.count += incr;
    state.last_incr = incr;
    if state.count <= MAX_COUNT {
      env.storage().persistent().set(&STATE, &state);
      env
        .storage()
        .persistent()
        .extend_ttl(&STATE, 50, env.storage().max_ttl());
      env
        .events()
        .publish((STATE, symbol_short!("increment")), state.count);
      Ok(state.count)
    } else {
      log!(&env, "failure here!");
      Err(Error::MaxCountReached)
      //panic_with_error!(&env, Error::MaxCountReached);
    }
  }

  pub fn reset_count(env: Env, value: u32) -> Result<u32, Error> {
    let mut state = Self::get_state(env.clone());
    log!(&env, "reset_count: {}", state);

    state.count = value;
    state.last_incr = value;
    env.storage().persistent().set(&STATE, &state);
    env
      .storage()
      .persistent()
      .extend_ttl(&STATE, 50, env.storage().max_ttl());
    Ok(state.count)
  }
  pub fn get_state(env: Env) -> State {
    let time = env.ledger().timestamp();
    log!(&env, "time: {}", time);
    //vec![&env, String::from_str(&env, "Hello"), nameString] // -> Vec<String>

    env.storage().persistent().get(&STATE).unwrap_or(State {
      count: 0,
      last_incr: 0,
    }) // If no value set, assume 0.
  }
}

mod test;
