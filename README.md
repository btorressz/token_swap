# token_swap

# Overview

Token Swap program is a decentralized finance (DeFi) application built on the Solana blockchain using the Anchor framework. This program enables users to:

- Create and manage liquidity pools for token pairs.
- Add and remove liquidity to pools, earning liquidity tokens representing pool shares.
- Swap tokens using dynamic pricing based on the Constant Product Market Maker (CPMM) model.

The program is will be secure, efficient, and designed for real-world DeFi scenarios.

# Features

## Dynamic Token Pricing
Prices adjust based on the pool's token reserves using the CPMM formula.

## Swap Fees
A configurable fee (e.g., 1%) is deducted from swaps, which can either:
- Increase pool liquidity, or
- Be collected by an admin account.

## Liquidity Tokens
Users receive tokens representing their share of the pool when they add liquidity and can redeem these tokens for underlying assets.

## Time-Locked Withdrawals
Ensures pool stability by imposing a withdrawal delay after liquidity deposits.

## Secure PDAs
Pools are managed via Program Derived Addresses (PDAs) for secure, trustless operations.

# Usage

## 1. Initialize Liquidity Pool
Call the `initialize_pool` instruction to set up a new liquidity pool.  
Specify parameters such as fee percentage and admin fee accounts.

## 2. Add Liquidity
Deposit two tokens into the pool.  
Receive liquidity tokens representing your share of the pool.

## 3. Remove Liquidity
Redeem liquidity tokens for the underlying assets.  
Enforce time-locked withdrawals for security.

## 4. Swap Tokens
Swap one token for another based on the pool's current state.  
A swap fee is applied automatically.

---

# Program Details

## Key Components

### Accounts
- **InitializePool**: Sets up the initial parameters for the pool.
- **AddLiquidity**: Allows users to deposit tokens into the pool.
- **RemoveLiquidity**: Enables users to withdraw their liquidity.
- **Swap**: Facilitates token swapping.

### State
 maintains the state of the liquidity pool, including:
- Pool authority
- Token pair details
- Total liquidity
- Fee percentage
- Admin fee account
- Timestamps for time-locked operations

### Custom Errors
- **InvalidTokenPair**: Triggered when incorrect tokens are used for the pool.
- **InsufficientOutput**: Occurs when the output token amount is too low.
- **WithdrawalTooSoon**: Enforced to prevent immediate liquidity withdrawal.

