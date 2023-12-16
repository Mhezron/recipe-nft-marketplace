# Recipe Marketplace

## Overview

This Rust Smart contract implements a recipe marketplace on the Internet Computer, allowing users to create, edit, and trade recipes. It leverages the Internet Computer's smart contract capabilities to manage user accounts, recipe storage, and transactions. The primary use case involves creating, sharing, and trading recipes within a decentralized marketplace. Users can contribute to community recipes, purchase non-community recipes, and manage their balances.

## Prerequisites

- Rust programming language (version X.X.X)
- DFINITY Canister SDK
- Internet Computer access

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/Mhezron/recipe-nft-marketplace.git
   cd recipe-nft-marketplace

## Memory Management

Memory is allocated using a `MemoryManager` from the `ic-stable-structures` crate:

```rust
static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = // initialized
```

This manages allocating `VirtualMemory` for storages.

## ID Generation

Unique IDs are generated using a thread-local `IdCell`:

```rust
static ID_COUNTER: RefCell<IdCell> = // initialized
```

The counter is incremented when adding new records.

## Record Storage

Records are stored in thread-local `StableBTreeMap`s:

The system utilizes thread-local static variables for memory management and storage. These include:

- **MEMORY_MANAGER**: Manages virtual memory.
- **ID_COUNTER**: Manages unique IDs.
- **USER_STORAGE**: Stores clients.

```rust
static CLIENT_STORAGE: RefCell<StableBTreeMap<u64, User>> = // initialized
```

## Traits

The `Storable` and `BoundedStorable` traits are implemented for serialization and bounding record sizes during storage.

### Main Structs

- **User:** Represents a user in the marketplace with details such as name, email, balance, and recipes.
- **Recipe:** Defines a recipe with attributes like title, description, category, and price.

### Payload Structs

- **RecipePayload:** Used for adding and editing recipes.
- **UserPayload:** Payload for adding new users.
- **ReviewPayload:** Payload for adding recipe reviews.
- **InitPayload:** Initial payload for contract initialization.
- **EditRecipePayload:** Payload for editing owned recipes.
- **EditCommunityRecipe:** Payload for editing community recipes.
- **ReturnUser:** Struct for returning user information.
- **BuyNftPayload:** Payload for buying a non-community recipe.

### Core Functions

#### Query Functions

1. `get_all_recipes:` Retrieve all recipes from the marketplace.
2. `get_all_for_sale_recipes:` Retrieve recipes available for sale.
3. `get_recipe_by_category:` Retrieve recipes by category or title.
4. `get_recipe_by_id:` Retrieve a specific recipe by its unique ID.
5. `get_user:` Retrieve user information by ID.

#### Update Functions

1. `add_recipe:` Add a new recipe to the marketplace.
2. `edit_owned_recipe:` Edit owned recipes (title, is_community, price, description).
3. `edit_community_recipe:` Edit community recipes (description only).
4. `buy_recipe_nft:` Buy a non-community recipe.
5. `transfer_recipe_to_user:` Transfer ownership of a recipe to a new user.
6. `add_user:` Add a new user to the marketplace.

### Error Handling

- **Error:** Enum for handling various error scenarios, including not found, already paid, invalid payload, and unauthorized access.

### Candid Interface

- Exported Candid interface for seamless interaction with the Internet Computer.

## More

To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

To learn more before you start working with recipe_nft, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.icp0.io)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd recipe_nft/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.

### Note on frontend environment variables

If you are hosting frontend code somewhere without using DFX, you may need to make one of the following adjustments to ensure your project does not fetch the root key in production:

- set`DFX_NETWORK` to `production` if you are using Webpack
- use your own preferred method to replace `process.env.DFX_NETWORK` in the autogenerated declarations
  - Setting `canisters -> {asset_canister_id} -> declarations -> env_override to a string` in `dfx.json` will replace `process.env.DFX_NETWORK` with the string in the autogenerated declarations
- Write your own `createActor` constructor
