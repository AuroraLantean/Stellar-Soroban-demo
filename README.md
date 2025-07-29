# Stellar-Soroban-demo

## Installation
Install Rust

Install the target
For Rust v1.85.0 or higher: `rustup target add wasm32v1-none`

Install Stellar CLI
```
cargo install --locked stellar-cli --features opt
echo "source <(stellar completion --shell zsh)" >> ~/.zshrc
stellar -V
```

Reference of Stellar Soroban Setup
https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup

Prediction Game Rules:
- game commission_fee = (sum of game.values) * commission
- game users_profit = sum of game.values - commission_fee
- game total_wins = sum of winning choices
- user claim amount = users_profit * sum of user's winning bet amounts / total_wins