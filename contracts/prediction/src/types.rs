use soroban_sdk::{contracterror, contracttype, symbol_short, Address, String, Symbol, Vec};

//----------== Error
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
  MaxCountReached = 1,
  UserExists = 2,
  UserDoesNotExist = 3,
  InsufficientBalance = 4,
  InsufficientAllowance = 5,
  BalanceExists = 6,
  StateNotInitialized = 7,
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
  pub bet_values: Vec<u128>,
  pub bet_numbers: Vec<u128>,
}

#[contracttype]
pub enum Registry {
  Users(Address),
  Owner,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[contracttype]
pub enum Outcome {
  Undecided,
  TrueOutcome,
  FalseOutcome,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct Bets {
  pub bettor: Address,
  pub amount: u128,
  pub bet_on_true: bool,
  pub claimed: bool,
}

#[contracttype]
pub enum StorageKey {
  Oracle,
  Token,
  TrueTotal,
  FalseTotal,
  Market,
  State,
  Bets(Address),
}
