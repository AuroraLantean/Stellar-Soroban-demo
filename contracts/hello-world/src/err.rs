use soroban_sdk::contracterror;
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
