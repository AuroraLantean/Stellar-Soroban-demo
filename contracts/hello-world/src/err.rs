use soroban_sdk::contracterror;
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
  LimitReached = 1,
  UserExists = 2,
  UserDoesNotExists = 3,
  InsufficientBalance = 4,
  InsufficientAllowance = 5,
}
