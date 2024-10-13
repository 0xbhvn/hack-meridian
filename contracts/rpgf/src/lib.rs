// contracts/rpgf/src/lib.rs
#![no_std]

use core::u64;

// Import necessary Soroban modules
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, token::TokenClient, Address, Bytes, Env, Map, Symbol, Vec
};

// Define custom errors for the contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 1,
    RoundNotFound = 2,
    RoundNotActive = 3,
    SubmissionNotFound = 4,
    SubmissionDeadlinePassed = 5,
    ExceededVoteLimit = 6,
    AlreadyVoted = 7,
    VotingClosed = 8,
    FundsAlreadyDisbursed = 9,
    InvalidAllocations = 10,
    TransferFailed = 11,
    InsufficientFunds = 12,
    AdminNotSet = 13,
}

// Define the Round struct
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Round {
    id: u64,
    funding_amount: u64,
    deadline: u64, // Unix timestamp
    is_active: bool,
    submissions: Vec<u64>, // List of submission IDs
    funds_disbursed: bool,
}

// Define the Submission struct
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Submission {
    id: u64,
    round_id: u64,
    submitter: Address,
    total_votes: u64,
}

// Define the main contract structure
#[contract]
pub struct RetroPGFContract;

#[contractimpl]
impl RetroPGFContract {
    pub fn initialize(env: Env, admin: Address) {
        let admin_key = symbol_short!("ADMIN");
        env.storage().instance().set(&admin_key, &admin);
    }

    // Each voter has a fixed number of votes to allocate
    const VOTE_CREDITS: u64 = 20;

    pub fn set_voter(env: Env, voter: Address) {
        let voter_key = symbol_short!("VOTER");
        env.storage().instance().set(&voter_key, &voter);
    }

    // Function to create a new round
    pub fn create_round(env: Env, funding_amount: u64, deadline: u64) -> Result<u64, ContractError> {
        let admin_key = symbol_short!("ADMIN");
        let admin = env
            .storage()
            .instance()
            .get::<Symbol, Address>(&admin_key)
            .ok_or(ContractError::Unauthorized)?;

        // Require authorization from the admin
        admin.require_auth();

        // Generate a new round ID
        let next_round_id_key = symbol_short!("NEXT_RND");
        let mut round_id = env
            .storage()
            .instance()
            .get::<Symbol, u64>(&next_round_id_key)
            .unwrap_or(0);

        round_id += 1;
        env.storage()
            .instance()
            .set(&next_round_id_key, &round_id);

        // Create a new round
        let round = Round {
            id: round_id,
            funding_amount,
            deadline,
            is_active: true,
            submissions: Vec::new(&env),
            funds_disbursed: false,
        };

        // Store the round
        env.storage()
            .persistent()
            .set(&Self::round_key(round_id), &round);

        // Emit event
        env.events()
            .publish((symbol_short!("RND_CREAT"), round_id), round_id);

        Ok(round_id)
    }

    // Helper function to get a round
    fn get_round(env: Env, round_id: u64) -> Result<Round, ContractError> {
        env.storage()
            .persistent()
            .get::<(Symbol, u64), Round>(&Self::round_key(round_id))
            .ok_or(ContractError::RoundNotFound)
    }

    // Helper function to generate storage key for rounds
    fn round_key(round_id: u64) -> (Symbol, u64) {
        (symbol_short!("ROUND"), round_id)
    }

    // Function to submit a project to a round
    pub fn submit_project(
        env: Env,
        round_id: u64,
    ) -> Result<u64, ContractError> {
        // Check if the round exists and is active
        let mut round = Self::get_round(env.clone(), round_id)?;

        if !round.is_active {
            return Err(ContractError::RoundNotActive);
        }

        // Check if the submission deadline has not passed
        let current_timestamp = env.ledger().timestamp();
        if current_timestamp > round.deadline {
            return Err(ContractError::SubmissionDeadlinePassed);
        }

        // Generate a new submission ID
        let next_submission_id_key = symbol_short!("NEXT_SUB");
        let mut submission_id = env
            .storage()
            .instance()
            .get::<Symbol, u64>(&next_submission_id_key)
            .unwrap_or(0);

        submission_id += 1;
        env.storage()
            .instance()
            .set(&next_submission_id_key, &submission_id);

        let voter_key = symbol_short!("VOTER");
        // Create a new submission
        let submission = Submission {
            id: submission_id,
            round_id,            
            submitter: env
                .storage()
                .instance()
                .get::<Symbol, Address>(&voter_key)
                .unwrap(),
            total_votes: 0,
        };

        // Store the submission
        env.storage()
            .persistent()
            .set(&Self::submission_key(submission_id), &submission);

        // Add submission ID to the round
        round.submissions.push_back(submission_id);
        env.storage()
            .persistent()
            .set(&Self::round_key(round_id), &round);

        // Emit event
        env.events()
            .publish((symbol_short!("PROJ_SUB"), submission_id), submission_id);

        Ok(submission_id)
    }

    // Helper function to get a submission
    fn get_submission(env: Env, submission_id: u64) -> Result<Submission, ContractError> {
        env.storage()
            .persistent()
            .get::<(Symbol, u64), Submission>(&Self::submission_key(submission_id))
            .ok_or(ContractError::SubmissionNotFound)
    }

    // Helper function to generate storage key for submissions
    fn submission_key(submission_id: u64) -> (Symbol, u64) {
        (symbol_short!("SUBMISSN"), submission_id)
    }

    // Function for voters to allocate votes to submissions
    pub fn allocate_votes(
        env: Env,
        round_id: u64,
        allocations: Map<u64, u64>,
    ) -> Result<(), ContractError> {
        let voter_key = symbol_short!("VOTER");
        let voter = env
            .storage()
            .instance()
            .get::<Symbol, Address>(&voter_key)
            .unwrap();

        // Calculate total votes allocated
        let mut total_votes_allocated: u64 = 0;
        for vote in allocations.values().iter() {
            total_votes_allocated += vote;
        }

        if total_votes_allocated > Self::VOTE_CREDITS {
            return Err(ContractError::ExceededVoteLimit);
        }

        // Store voter allocations
        env.storage().persistent().set(
            &Self::voter_allocation_key(round_id, &voter),
            &allocations,
        );

        // Update total votes for each submission
        for (submission_id, votes) in allocations.iter() {
            let mut submission = Self::get_submission(env.clone(), submission_id)?;
            submission.total_votes += votes;
            env.storage()
                .persistent()
                .set(&Self::submission_key(submission_id), &submission);
        }

        // Emit event
        env.events()
            .publish((symbol_short!("VOTE_ALC"), voter.clone()), voter.clone());

        Ok(())
    }

    // Helper function to generate storage key for voter allocations
    fn voter_allocation_key(round_id: u64, voter: &Address) -> (Symbol, u64, Address) {
        (symbol_short!("VOTR_ALC"), round_id, voter.clone())
    }

    // Function to close voting and calculate funding allocations
    pub fn close_voting(env: Env, round_id: u64) -> Result<(), ContractError> {
        let admin_key = symbol_short!("ADMIN");
        let admin = env
            .storage()
            .instance()
            .get::<Symbol, Address>(&admin_key)
            .ok_or(ContractError::Unauthorized)?;

        // Require authorization from the admin
        admin.require_auth();

        let mut round = Self::get_round(env.clone(), round_id)?;

        if !round.is_active {
            return Err(ContractError::RoundNotActive);
        }

        // Close the round
        round.is_active = false;
        env.storage()
            .persistent()
            .set(&Self::round_key(round_id), &round);

        // Calculate total votes
        let mut total_votes = 0u64;
        for submission_id in round.submissions.iter() {
            let submission = Self::get_submission(env.clone(), submission_id)?;
            total_votes += submission.total_votes;
        }

        // Calculate funding allocations
        let mut allocations = Map::new(&env);
        for submission_id in round.submissions.iter() {
            let submission = Self::get_submission(env.clone(), submission_id)?;
            let allocation = if total_votes > 0 {
                (submission.total_votes * round.funding_amount) / total_votes
            } else {
                0
            };
            allocations.set(submission_id, allocation);
        }

        // Store funding allocations
        env.storage()
            .persistent()
            .set(&Self::allocations_key(round_id), &allocations);

        // Emit event
        env.events()
            .publish((symbol_short!("VOTE_CLSD"), round_id), round_id);

        Ok(())
    }

    // Function to disburse funds to submissions based on allocations
    pub fn disburse_funds(env: Env, round_id: u64, token_address: Address) -> Result<(), ContractError> {
        let admin_key = symbol_short!("ADMIN");
        let admin = env
            .storage()
            .instance()
            .get::<Symbol, Address>(&admin_key)
            .ok_or(ContractError::Unauthorized)?;

        // Require authorization from the admin
        admin.require_auth();

        let mut round = Self::get_round(env.clone(), round_id)?;

        if round.funds_disbursed {
            return Err(ContractError::FundsAlreadyDisbursed);
        }

        // Get funding allocations
        let allocations = env
            .storage()
            .persistent()
            .get::<(Symbol, u64), Map<u64, u64>>(&Self::allocations_key(round_id))
            .ok_or(ContractError::VotingClosed)?;

        // Initialize token client
        let token_client = TokenClient::new(&env, &token_address);

        // Check the contract's token balance
        let contract_balance = token_client.balance(&admin);

        if contract_balance < round.funding_amount as i128 {
            return Err(ContractError::InsufficientFunds);
        }

        // Disburse funds to submitters
        for (submission_id, amount) in allocations.iter() {
            let submission = Self::get_submission(env.clone(), submission_id)?;
            // Convert amount to i128
            let amount_i128 = amount as i128;

            // Transfer tokens from the contract to the submitter
            token_client.transfer(
                &admin,        // From: The contract's own address
                &submission.submitter, // To: The submitter's address
                &amount_i128,          // Amount: The allocation amount as i128
            );
        }

        // Mark funds as disbursed
        round.funds_disbursed = true;
        env.storage()
            .persistent()
            .set(&Self::round_key(round_id), &round);

        // Emit event
        env.events()
            .publish((symbol_short!("FUND_DISB"), round_id), round_id);

        Ok(())
    }

    // Helper function to generate storage key for allocations
    fn allocations_key(round_id: u64) -> (Symbol, u64) {
        (symbol_short!("FUND_ALC"), round_id)
    }
}
