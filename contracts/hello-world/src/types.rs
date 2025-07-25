use soroban_sdk::{contracterror, contracttype, Address, Symbol};
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
}
//----------== Bank
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
  pub count: u32,
  pub last_incr: u32,
}

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
//----------== Prediction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[contracttype]
pub enum Outcome {
  Undecided,
  TrueOutcome,
  FalseOutcome,
}
#[contracttype]
pub struct Bets {
  pub bettor: Address,
  pub amount: i128,
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
