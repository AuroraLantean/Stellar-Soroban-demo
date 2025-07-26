use soroban_sdk::{contracterror, contracttype, symbol_short, Address, String, Symbol, Vec};

//----------== Error
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
  StateNotInitialized = 1,
  InsufficientBalance = 2,
  InsufficientAllowance = 3,
  UserExists = 4,
  UserDoesNotExist = 6,
  UserBalanceExists = 7,
  GameDoesNotExist = 8,
  GameAdminUnauthorized = 9,
  GameBalcInvalid = 10,
  BeforeStartTime = 11,
  AfterEndTime = 12,
  BetDoesNotExist = 13,
  BetIndexInvalid = 14,
  BetValueInvalid = 15,
  MaxCountReached = 16,
}
//----------== Config
pub const MAX_COUNT: u32 = 5;
//----------== Bank
pub const STATE: Symbol = symbol_short!("STATE");
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
  pub count: u32,
  pub last_incr: u32,
  pub admin: Address,
  pub token: Address,
  pub market_name: String,
  pub status: Status,
  pub bet_values: Vec<u128>,
  pub bet_numbers: Vec<u128>,
}

#[contracttype]
pub enum Registry {
  Users(Address),
  Games(u32),         // game_id
  Bets(Address, u32), //user, game_id
}
//if env.storage().instance().has(&DataKey::Owner) {  panic!("owner is already set"); }
#[contracttype]
#[derive(Clone, Debug)]
pub struct User {
  pub addr: Address,
  pub id: Symbol,
  pub balance: u128,
  pub updated_at: u64,
}
//----------== Prediction
#[contracttype]
#[derive(Clone, Debug)]
pub struct Game {
  pub game_admin: Address,
  pub balances: Vec<u128>, //[u128; 4],
  pub time_start: u64,
  pub time_end: u64,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct Bet {
  pub bet_values: Vec<u128>, //[u128; 4],
  pub claimed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[contracttype]
pub enum Status {
  Initial,
  Ready,
  Active,
  Ended,
  Paused,
}
