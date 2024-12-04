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
