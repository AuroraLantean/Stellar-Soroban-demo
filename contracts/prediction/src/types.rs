use soroban_sdk::{contracterror, contracttype, symbol_short, Address, String, Symbol, Vec};

//----------== Error
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
  AmountInvalid = 1,
  InsufficientBalance = 2,
  InsufficientAllowance = 3,
  StateNotInitialized = 10,
  StateStatusInvalid = 11,
  UserExists = 21,
  UserDoesNotExist = 22,
  UserBalanceExists = 23,
  UserClaimsZero = 24,
  GameDoesNotExist = 30,
  GameAdminUnauthorized = 31,
  GameStatusInvalid = 32,
  GameValueInvalid = 33,
  GameNumberInvalid = 34,
  GameOutcomeNone = 35,
  BeforeStartTime = 36,
  AfterEndTime = 37,
  BeforeEndTime = 38,
  EndTimeTooSmall = 39,
  BetDoesNotExist = 40,
  BetIndexInvalid = 41,
  BetValueInvalid = 42,
  BetClaimedAlready = 43,
  MaxCountReached = 99,
}
//----------== Config
pub const MAX_COUNT: u32 = 5;
//----------== Bank
pub const STATE: Symbol = symbol_short!("STATE");
#[contracttype] //no Copy
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
  pub count: u32,
  pub last_incr: u32,
  pub admin: Address,
  pub token: Address,
  pub market_name: String,
  pub status: Status,
}

#[contracttype]
pub enum Registry {
  Users(Address),
  Games(u32),         // game_id
  Bets(Address, u32), //user, game_id
}
//if env.storage().instance().has(&DataKey::Owner) {  panic!("owner is already set"); }
pub const USER: Symbol = symbol_short!("USER");
#[contracttype]
#[derive(Clone, Debug)]
pub struct User {
  pub addr: Address,
  pub id: Symbol,
  pub balance: u128,
  pub updated_at: u64,
}
//----------== Prediction
pub const GAME: Symbol = symbol_short!("GAME");
#[contracttype]
#[derive(Clone, Debug)]
pub struct Game {
  pub game_admin: Address,
  pub time_start: u64,
  pub time_end: u64,
  pub commission_rate: u128,
  pub users_profit: u128,
  pub total_wins: u128,
  pub status: Status,
  pub values: Vec<u128>,
  pub numbers: Vec<u32>,
  pub outcome: Vec<u32>,
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
  Active,
  Settled,
  Paused,
}
