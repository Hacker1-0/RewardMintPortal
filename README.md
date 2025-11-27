#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Env, Symbol,
};

const GOV_NS: Symbol = symbol_short!("RSGOV"); // proposal namespace

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub proposal_id: u64,
    pub creator: Symbol,
    pub title: Symbol,        // short identifier for the proposal
    pub yes_votes: i128,
    pub no_votes: i128,
    pub active: bool,
}

#[contract]
pub struct RideShareGovernance;

#[contractimpl]
impl RideShareGovernance {
    /// Create a new governance proposal (e.g., change fee policy, safety rule).
    pub fn create_proposal(env: Env, proposal_id: u64, creator: Symbol, title: Symbol) {
        let key = Self::proposal_key(proposal_id);
        let existing: Option<Proposal> = env.storage().instance().get(&key);
        if existing.is_some() {
            panic!("Proposal id already exists");
        }

        let proposal = Proposal {
            proposal_id,
            creator,
            title,
            yes_votes: 0,
            no_votes: 0,
            active: true,
        };

        env.storage().instance().set(&key, &proposal);
    }

    /// Cast a vote: `support = true` for YES, `false` for NO.
    /// `weight` can represent voting power (e.g., governance tokens or reputation).
    pub fn vote(env: Env, proposal_id: u64, support: bool, weight: i128) {
        if weight <= 0 {
            panic!("Weight must be positive");
        }

        let key = Self::proposal_key(proposal_id);
        let mut prop: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| panic!("Proposal not found"));

        if !prop.active {
            panic!("Proposal is closed");
        }

        if support {
            prop.yes_votes += weight;
        } else {
            prop.no_votes += weight;
        }

        env.storage().instance().set(&key, &prop);
    }

    /// Close a proposal so it cannot receive more votes.
    pub fn close_proposal(env: Env, proposal_id: u64) {
        let key = Self::proposal_key(proposal_id);
        let mut prop: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| panic!("Proposal not found"));

        prop.active = false;
        env.storage().instance().set(&key, &prop);
    }

    /// Check if a proposal passed (more YES than NO) after closing.
    pub fn is_proposal_passed(env: Env, proposal_id: u64) -> bool {
        let key = Self::proposal_key(proposal_id);
        let prop: Option<Proposal> = env.storage().instance().get(&key);

        match prop {
            Some(p) => !p.active && p.yes_votes > p.no_votes,
            None => false,
        }
    }

    /// Get full proposal data for UI or analytics.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<Proposal> {
        let key = Self::proposal_key(proposal_id);
        env.storage().instance().get(&key)
    }

    /// Internal helper: composite storage key under GOV_NS.
    fn proposal_key(proposal_id: u64) -> (Symbol, u64) {
        (GOV_NS, proposal_id)
    }
}
