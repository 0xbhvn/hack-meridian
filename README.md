# Retroactive Public Goods Funding (RPGF) Contract

This repository contains the source code for the Retroactive Public Goods Funding (RPGF) smart contract implemented for the Soroban platform.

## Overview

The RPGF contract is designed to manage funding rounds where projects can submit proposals and voters can allocate votes to these submissions. Based on the votes, the funding is distributed proportionally to the submissions after the round ends.

Key features include:

- Creation of funding rounds by an admin.
- Submission of projects to active rounds.
- Voters allocate a fixed number of votes across submissions.
- Calculation of funding allocations based on votes.
- Disbursement of funds to submissions after the round ends.

## Video Demo

[Watch the video walkthrough of the contract here.](https://www.loom.com/share/cf4f5a61cd434a62944b05945f886fb7?sid=43073d6b-fdae-4ce2-9132-1e6c9866ef25)

## Contract Structure

### Modules and Imports

The contract uses several modules from the Soroban SDK:

- `contract`, `contracterror`, `contractimpl`, `contracttype`, `log`, `symbol_short`
- `token::TokenClient`
- `Address`, `Bytes`, `Env`, `Map`, `Symbol`, `Vec`

### Custom Errors

The contract defines a set of custom errors using the `ContractError` enum to handle various error conditions.

```rust
#[contracterror]
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
```

### Data Structures

The contract defines two main data structures:

- `Round`: Represents a funding round.
- `Submission`: Represents a project submission to a round.

#### Round

```rust
#[contracttype]
pub struct Round {
    id: u64,
    funding_amount: u64,
    deadline: u64, // Unix timestamp
    is_active: bool,
    submissions: Vec<u64>, // List of submission IDs
    funds_disbursed: bool,
}
```

#### Submission

```rust
#[contracttype]
pub struct Submission {
    id: u64,
    round_id: u64,
    submitter: Address,
    total_votes: u64,
}
```

## Contract Functions

### Initialization

#### `initialize(env: Env, admin: Address)`

Initializes the contract by setting the admin address.

Usage:

```rust
RetroPGFContract::initialize(env, admin_address);
```

### Admin Functions

#### `create_round(env: Env, funding_amount: u64, deadline: u64) -> Result<u64, ContractError>`

Creates a new funding round.

- **Parameters:**
  - `funding_amount`: Total amount of tokens to be distributed in this round.
  - `deadline`: Unix timestamp indicating when the round ends.

- **Returns:**
  - `round_id`: The ID of the newly created round.

- **Errors:**
  - `Unauthorized`: If the caller is not the admin.

Usage:

```rust
let round_id = RetroPGFContract::create_round(env, funding_amount, deadline)?;
```

#### `disburse_funds(env: Env, round_id: u64, token_address: Address) -> Result<(), ContractError>`

Disburses funds to submissions based on the allocations calculated after voting.

- **Parameters:**
  - `round_id`: The ID of the round to disburse funds for.
  - `token_address`: The address of the token contract.

- **Errors:**
  - `Unauthorized`: If the caller is not the admin.
  - `RoundNotFound`: If the round does not exist.
  - `FundsAlreadyDisbursed`: If funds have already been disbursed for this round.
  - `InsufficientFunds`: If the contract's balance is less than the funding amount.

Usage:

```rust
RetroPGFContract::disburse_funds(env, round_id, token_address)?;
```

### Voter Functions

#### `set_voter(env: Env, voter: Address)`

Sets the address of the voter. This would typically be called during voter registration.

Usage:

```rust
RetroPGFContract::set_voter(env, voter_address);
```

#### `submit_project(env: Env, round_id: u64) -> Result<u64, ContractError>`

Allows a user to submit a project to an active round.

- **Parameters:**
  - `round_id`: The ID of the round to submit the project to.

- **Returns:**
  - `submission_id`: The ID of the newly created submission.

- **Errors:**
  - `RoundNotFound`: If the round does not exist.
  - `RoundNotActive`: If the round is not active.
  - `SubmissionDeadlinePassed`: If the current time is past the round's deadline.

Usage:

```rust
let submission_id = RetroPGFContract::submit_project(env, round_id)?;
```

#### `allocate_votes(env: Env, round_id: u64, allocations: Map<u64, u64>) -> Result<(), ContractError>`

Allows a voter to allocate their votes to submissions in a round.

- **Parameters:**
  - `round_id`: The ID of the round to allocate votes in.
  - `allocations`: A map where keys are submission IDs and values are the number of votes allocated.

- **Errors:**
  - `ExceededVoteLimit`: If the total votes allocated exceed the voter's allowed vote credits.

Usage:

```rust
let mut allocations = Map::new(&env);
allocations.set(submission_id1, 10);
allocations.set(submission_id2, 10);

RetroPGFContract::allocate_votes(env, round_id, allocations)?;
```

### Round Management

#### `close_voting(env: Env, round_id: u64) -> Result<(), ContractError>`

Closes the voting for a round and calculates funding allocations.

- **Parameters:**
  - `round_id`: The ID of the round to close voting for.

- **Errors:**
  - `RoundNotFound`: If the round does not exist.
  - `RoundNotActive`: If the round is not active.

Usage:

```rust
RetroPGFContract::close_voting(env, round_id)?;
```

## Helper Functions

### Storage Keys

The contract uses helper functions to generate storage keys for rounds, submissions, and voter allocations.

- `round_key(round_id: u64) -> (Symbol, u64)`
- `submission_key(submission_id: u64) -> (Symbol, u64)`
- `voter_allocation_key(round_id: u64, voter: &Address) -> (Symbol, u64, Address)`
- `allocations_key(round_id: u64) -> (Symbol, u64)`

### Data Retrieval

- `get_round(env: Env, round_id: u64) -> Result<Round, ContractError>`
- `get_submission(env: Env, submission_id: u64) -> Result<Submission, ContractError>`

## Constants

### `VOTE_CREDITS`

Each voter has a fixed number of votes to allocate per round.

```rust
const VOTE_CREDITS: u64 = 20;
```

## Error Handling

The contract defines specific errors to handle various failure cases. These errors are returned as `ContractError` enum variants.

Examples:

- `ContractError::Unauthorized`: When a caller is not authorized to perform an action.
- `ContractError::RoundNotFound`: When the specified round does not exist.
- `ContractError::SubmissionDeadlinePassed`: When attempting to submit a project after the deadline.

## Events

The contract emits events for important actions:

- `RND_CREAT`: When a new round is created.
- `PROJ_SUB`: When a new project submission is made.
- `VOTE_ALC`: When a voter allocates votes.
- `VOTE_CLSD`: When voting is closed for a round.
- `FUND_DISB`: When funds are disbursed to submissions.

## Usage Example

Below is an example of how the contract might be used:

```rust
// Initialize the contract with the admin address
RetroPGFContract::initialize(env.clone(), admin_address);

// Admin creates a new funding round
let round_id = RetroPGFContract::create_round(env.clone(), 100_000, deadline_timestamp)?;

// User submits a project to the round
let submission_id = RetroPGFContract::submit_project(env.clone(), round_id)?;

// Voter allocates votes to submissions
let mut allocations = Map::new(&env);
allocations.set(submission_id, 20); // Allocate all 20 votes to this submission

RetroPGFContract::allocate_votes(env.clone(), round_id, allocations)?;

// Admin closes voting and calculates allocations
RetroPGFContract::close_voting(env.clone(), round_id)?;

// Admin disburses funds to submissions
RetroPGFContract::disburse_funds(env.clone(), round_id, token_address)?;
```

## Deployment

To deploy the contract:

1. Compile the contract using the Rust compiler targeting the Wasm32 environment.
2. Deploy the compiled Wasm binary to the Soroban network.
3. Initialize the contract by calling the `initialize` function with the admin address.

## Development

### Prerequisites

- Rust programming language
- Soroban SDK
- Soroban CLI tools

### Building

To build the contract, run:

```sh
cargo build --target wasm32-unknown-unknown --release
```

### Testing

Write unit tests to cover the contract's functionality.

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

---