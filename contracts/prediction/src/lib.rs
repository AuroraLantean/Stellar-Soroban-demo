#![no_std]
use sep_41_token::TokenClient;
use soroban_sdk::{
  contract, contractimpl, log, symbol_short, token, vec, Address, Env, String, Symbol, Vec,
};

use crate::types::{Bet, Error, Game, Registry, State, Status, User, MAX_COUNT, STATE};
mod types;

//TODO: simpleAccount
#[contract]
pub struct Prediction;

#[contractimpl]
impl Prediction {
  pub fn __constructor(env: Env, admin: Address, token: Address, market_name: String) {
    log!(&env, "__constructor");
    //signers: Vec<BytesN<32>>
    //check_string_not_empty(&env, &market_name);
    let state = State {
      count: 0,
      last_incr: 0,
      admin,
      token,
      market_name,
      status: Status::Initial,
      bet_values: vec![&env, 0, 0, 0, 0],
      bet_numbers: vec![&env, 0, 0, 0, 0],
    };
    env.storage().persistent().set(&STATE, &state);
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
      return Err(Error::UserBalanceExists);
    }
    env.storage().persistent().remove(&user);
    Ok(0u32)
  }

  pub fn set_game(
    env: Env,
    game_admin: Address,
    game_id: u32,
    time_start: u64,
    time_end: u64,
    //prices: Vec<u128>, //[u128; 4],
  ) -> Result<u32, Error> {
    log!(&env, "setup_game");
    game_admin.require_auth();
    let time = env.ledger().timestamp();
    log!(&env, "time: {}", time);

    let key = Registry::Games(game_id);
    let game_opt: Option<Game> = env.storage().persistent().get(&key);
    let game = if let Some(mut prev) = game_opt {
      if prev.game_admin != game_admin {
        return Err(Error::GameAdminUnauthorized);
      }
      if time > prev.time_end {
        prev.balances = vec![&env, 0, 0, 0, 0];
      }
      prev.time_start = time_start;
      prev.time_end = time_end;
      prev
    } else {
      if time > time_end {
        return Err(Error::AfterEndTime);
      }
      Game {
        game_admin,
        balances: vec![&env, 0, 0, 0, 0],
        time_start,
        time_end,
      }
    };
    env.storage().persistent().set(&key, &game);
    Ok(0u32)
  }
  pub fn get_game(env: Env, game_id: u32) -> Option<Game> {
    let key = Registry::Games(game_id);
    let game_opt: Option<Game> = env.storage().persistent().get(&key);
    game_opt
  }
  pub fn bet(
    env: Env,
    user: Address,
    game_id: u32,
    amount_u128: u128,
    bet_index: u32,
  ) -> Result<u32, Error> {
    log!(&env, "bet");
    user.require_auth();
    let amount: i128 = amount_u128.try_into().unwrap();
    let state = Self::get_state(env.clone())?;
    log!(&env, "bet: {}", state);
    //assert_eq!(state.status, Status::Active, "not active");

    if amount <= 0 {
      panic!("amount invalid");
    }
    let ctrt_id = env.current_contract_address();

    let token = token::Client::new(&env, &state.token);
    let sender_balance = token.balance(&user);
    if sender_balance < amount {
      return Err(Error::InsufficientBalance);
    }
    let allowance = token.allowance(&user, &ctrt_id);
    if allowance < amount {
      return Err(Error::InsufficientAllowance);
    }

    //check game_id
    log!(&env, "get game");
    let time = env.ledger().timestamp();
    log!(&env, "time: {}", time);

    let key = Registry::Games(game_id);
    let game_opt: Option<Game> = env.storage().persistent().get(&key);
    if game_opt.is_none() {
      return Err(Error::GameDoesNotExist);
    }
    let mut game = game_opt.unwrap();
    if time < game.time_start {
      return Err(Error::BeforeStartTime);
    } //TODO: time travel
    if time > game.time_end {
      return Err(Error::AfterEndTime);
    }
    let balc_opt = game.balances.get(bet_index);
    if balc_opt.is_none() {
      return Err(Error::GameBalcInvalid);
    }
    let balc = balc_opt.unwrap();
    game.balances.set(bet_index, balc + amount_u128);
    env.storage().persistent().set(&key, &game);

    log!(&env, "get bet");
    if bet_index > 3 {
      return Err(Error::BetIndexInvalid);
    }
    let key = Registry::Bets(user.clone(), game_id);
    let bet_opt: Option<Bet> = env.storage().persistent().get(&key);

    let bet = if let Some(mut bet) = bet_opt {
      let prev_amt_opt = bet.bet_values.get(bet_index);
      if prev_amt_opt.is_none() {
        return Err(Error::BetValueInvalid);
      }
      bet
        .bet_values
        .set(bet_index, prev_amt_opt.unwrap() + amount_u128);
      bet
    } else {
      let mut bet_values: Vec<u128> = vec![&env, 0, 0, 0, 0];
      bet_values.set(bet_index, amount_u128);
      Bet {
        bet_values: bet_values,
        claimed: false,
      }
    };
    //log!(&env, "bet:{:?}", bet);
    env.storage().persistent().set(&key, &bet);
    env
      .storage()
      .persistent()
      .extend_ttl(&key, 50, env.storage().max_ttl());

    token.transfer_from(&ctrt_id, &user, &ctrt_id, &amount);
    Ok(0u32)
  }
  pub fn get_bet(env: Env, user: Address, game_id: u32) -> Option<Bet> {
    let key = Registry::Bets(user, game_id);
    let bet_opt: Option<Bet> = env.storage().persistent().get(&key);
    bet_opt
  }
  pub fn settle(_env: Env, admin: Address) {
    admin.require_auth();
  }
  pub fn increment(env: Env, incr: u32) -> Result<u32, Error> {
    let mut state = Self::get_state(env.clone())?;
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
    let mut state = Self::get_state(env.clone())?;
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
  pub fn get_state(env: Env) -> Result<State, Error> {
    let state_opt = env.storage().persistent().get(&STATE);
    if state_opt.is_none() {
      return Err(Error::StateNotInitialized);
    }
    state_opt.unwrap()
  }
}

mod test;
