#![no_std]
//use sep_41_token::TokenClient;
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
      status: Status::Active,
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
    commission_rate: u128, //0.1% as 1
  ) -> Result<u32, Error> {
    log!(&env, "setup_game");
    game_admin.require_auth();
    if time_end <= time_start {
      return Err(Error::EndTimeTooSmall);
    }
    let time = env.ledger().timestamp();
    log!(&env, "time: {}", time);

    let key = Registry::Games(game_id);
    let game_opt: Option<Game> = env.storage().persistent().get(&key);
    let empty_vec: Vec<u128> = vec![&env, 0, 0, 0, 0];
    let empty_vec32: Vec<u32> = vec![&env, 0, 0, 0, 0];
    let game = if let Some(mut prev) = game_opt {
      if prev.game_admin != game_admin {
        return Err(Error::GameAdminUnauthorized);
      }
      if time < prev.time_end {
        return Err(Error::BeforeEndTime);
      }
      if prev.status != Status::Settled {
        return Err(Error::GameStatusInvalid);
      }
      prev.time_start = time_start;
      prev.time_end = time_end;
      prev.status = Status::Active;
      prev.values = empty_vec.clone();
      prev.numbers = empty_vec32.clone();
      prev.outcome = empty_vec32.clone();
      prev
    } else {
      if time > time_end {
        return Err(Error::AfterEndTime);
      }
      Game {
        game_admin,
        time_start,
        time_end,
        commission_rate,
        status: Status::Active,
        values: empty_vec.clone(),
        numbers: empty_vec32.clone(),
        outcome: empty_vec32.clone(),
      }
    };
    env.storage().persistent().set(&key, &game);
    env
      .storage()
      .persistent()
      .extend_ttl(&key, 50, env.storage().max_ttl());
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
    value128: u128,
    bet_index: u32,
  ) -> Result<u32, Error> {
    log!(&env, "bet");
    user.require_auth();
    let amount: i128 = value128.try_into().unwrap();
    if amount <= 0 {
      panic!("amount invalid");
    }
    if bet_index > 3 {
      panic!("bet index out of bound");
    }
    let ctrt_addr = env.current_contract_address();

    //check state
    let state = Self::get_state(env.clone())?;
    log!(&env, "state: {}", state);
    if state.status != Status::Active {
      return Err(Error::StateStatusInvalid);
    };

    let token = token::Client::new(&env, &state.token);
    let sender_balance = token.balance(&user);
    if sender_balance < amount {
      return Err(Error::InsufficientBalance);
    }
    let allowance = token.allowance(&user, &ctrt_addr);
    if allowance < amount {
      return Err(Error::InsufficientAllowance);
    }

    //check time
    let time = env.ledger().timestamp();
    log!(&env, "time: {}", time);

    //check game
    let key = Registry::Games(game_id);
    let game_opt: Option<Game> = env.storage().persistent().get(&key);
    if game_opt.is_none() {
      return Err(Error::GameDoesNotExist);
    }
    let mut game = game_opt.unwrap();
    if game.status != Status::Active {
      return Err(Error::GameStatusInvalid);
    }
    if time < game.time_start {
      return Err(Error::BeforeStartTime);
    } //TODO: test on time travel
    if time >= game.time_end {
      return Err(Error::AfterEndTime);
    }
    //add Game value
    let value_opt = game.values.get(bet_index);
    if value_opt.is_none() {
      return Err(Error::GameValueInvalid);
    }
    let value = value_opt.unwrap();
    game.values.set(bet_index, value + value128);

    //add Game number
    let number_opt = game.numbers.get(bet_index);
    if number_opt.is_none() {
      return Err(Error::GameNumberInvalid);
    }
    let number = number_opt.unwrap();
    game.numbers.set(bet_index, number + 1);

    env.storage().persistent().set(&key, &game);

    // get_bat
    log!(&env, "get bet");
    if bet_index > 3 {
      return Err(Error::BetIndexInvalid);
    }
    let key = Registry::Bets(user.clone(), game_id);
    let bet_opt: Option<Bet> = env.storage().persistent().get(&key);

    //add bet
    let bet = if let Some(mut bet) = bet_opt {
      let value_opt = bet.bet_values.get(bet_index);
      if value_opt.is_none() {
        return Err(Error::BetValueInvalid);
      }
      bet.bet_values.set(bet_index, value_opt.unwrap() + value128);
      bet
    } else {
      let mut bet_values: Vec<u128> = vec![&env, 0, 0, 0, 0];
      bet_values.set(bet_index, value128);
      Bet {
        bet_values,
        claimed: false,
      }
    };
    //log!(&env, "bet:{:?}", bet);
    //save bet
    env.storage().persistent().set(&key, &bet);
    env
      .storage()
      .persistent()
      .extend_ttl(&key, 50, env.storage().max_ttl());
    //transfer token
    token.transfer_from(&ctrt_addr, &user, &ctrt_addr, &amount);
    Ok(0u32)
  }

  pub fn get_bet(env: Env, user: Address, game_id: u32) -> Option<Bet> {
    let key = Registry::Bets(user, game_id);
    let bet_opt: Option<Bet> = env.storage().persistent().get(&key);
    bet_opt
  }

  pub fn settle(
    env: Env,
    admin: Address,
    game_id: u32,
    outcome: Vec<u32>,
    vault: Address,
  ) -> Result<u32, Error> {
    log!(&env, "settle");
    admin.require_auth();
    let time = env.ledger().timestamp();
    log!(&env, "time: {}", time);

    //check and end game
    let key = Registry::Games(game_id);
    let game_opt: Option<Game> = env.storage().persistent().get(&key);
    if game_opt.is_none() {
      return Err(Error::GameDoesNotExist);
    }
    let mut game = game_opt.unwrap();
    if admin != game.game_admin {
      return Err(Error::GameAdminUnauthorized);
    }
    if time < game.time_end {
      return Err(Error::BeforeEndTime);
    }
    if game.status != Status::Active && game.status != Status::Paused {
      return Err(Error::GameStatusInvalid);
    }
    game.status = Status::Settled;
    //game.time_start = 0;
    //game.time_end = 0;

    //calculate commission
    let game_value_sum = game.values.iter().sum::<u128>();
    log!(&env, "game_value_sum: {}", game_value_sum);
    let numerator = game_value_sum
      .checked_mul(game.commission_rate)
      .expect("numerator"); //reduce commision rate when this error happens
    let commission_fee = numerator.div_ceil(1000); //rounding up here may balance users' rounding down
    log!(&env, "commission_fee: {}", commission_fee);

    game.outcome = outcome;
    env.storage().persistent().set(&key, &game);
    env
      .storage()
      .persistent()
      .extend_ttl(&key, 50, env.storage().max_ttl());

    //check token amount
    let ctrt_addr = env.current_contract_address();

    //check state
    let state = Self::get_state(env.clone())?;
    log!(&env, "state: {}", state);
    let token = token::Client::new(&env, &state.token);
    let balc = token.balance(&ctrt_addr);
    let fee = commission_fee.cast_signed();
    if balc < fee {
      return Err(Error::InsufficientBalance);
    }
    token.transfer(&ctrt_addr, &vault, &fee);
    Ok(0u32)
  }
  /*pub struct Game {
    pub game_admin: Address,
    pub time_start: u64,
    pub time_end: u64,
    pub status: Status,
    pub values: Vec<u128>,
    pub numbers: Vec<u128>,
  } */
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
