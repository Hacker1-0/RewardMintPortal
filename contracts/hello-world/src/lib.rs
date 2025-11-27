#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Env, Symbol,
};

const REWARD_NS: Symbol = symbol_short!("RWRD"); // namespace for per-user balances

#[contracttype]
#[derive(Clone)]
pub struct RewardBalance {
    pub user: Symbol,
    pub points: i128, // fungible-style integer amount
}

#[contract]
pub struct RewardMintPortal;

#[contractimpl]
impl RewardMintPortal {
    /// Mint reward points to a user (called by HR / rewards engine).
    pub fn mint_rewards(env: Env, user: Symbol, amount: i128) {
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let key = Self::user_key(user.clone());
        let mut bal: RewardBalance = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or(RewardBalance { user: user.clone(), points: 0 });

        bal.points += amount;
        env.storage().instance().set(&key, &bal);
    }

    /// Redeem (burn) reward points for a user.
    pub fn redeem_rewards(env: Env, user: Symbol, amount: i128) {
        if amount <= 0 {
            panic!("amount must be positive");
        }
        let key = Self::user_key(user.clone());
        let mut bal: RewardBalance = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| panic!("no rewards for user"));

        if bal.points < amount {
            panic!("insufficient reward points");
        }

        bal.points -= amount;
        env.storage().instance().set(&key, &bal);
    }

    /// View current reward balance for a user.
    pub fn get_balance(env: Env, user: Symbol) -> i128 {
        let key = Self::user_key(user.clone());
        let bal: RewardBalance =
            env.storage().instance().get(&key).unwrap_or(RewardBalance { user, points: 0 });
        bal.points
    }

    /// Reset a user's reward balance to zero (e.g., for expiry / policy reset).
    pub fn reset_balance(env: Env, user: Symbol) {
        let key = Self::user_key(user.clone());
        let mut bal: RewardBalance =
            env.storage().instance().get(&key).unwrap_or(RewardBalance { user, points: 0 });
        bal.points = 0;
        env.storage().instance().set(&key, &bal);
    }

    /// Internal helper: composite storage key under REWARD_NS.
    fn user_key(user: Symbol) -> (Symbol, Symbol) {
        (REWARD_NS, user)
    }
}
