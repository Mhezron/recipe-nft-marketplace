#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use validator::Validate;

// Define type aliases for convenience
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Contract {
    id: u64,
    email: String,
    password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Recipe {
    id: u64,
    title: String,
    category: String,
    description: String,
    price: u32,
    user_id: u64,
    is_community: bool,
    is_for_sale: bool,
    reviews: Vec<String>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct User {
    id: u64,
    name: String,
    password: String,
    email: String,
    balance: u32,
    recipes: Vec<u64>,
}

// Implement the 'Storable' trait for 'Recipe', 'User' and 'CommunityRecipe'

impl Storable for User {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Recipe {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Contract {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement the 'BoundedStorable' trait for 'Recipe', 'User' and 'CommunityRecipe'
impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Recipe {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Contract {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Define thread-local static variables for memory management and storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static USER_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static RECIPE_STORAGE: RefCell<StableBTreeMap<u64, Recipe, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static CONTRACT_STORAGE: RefCell<StableBTreeMap<u64, Contract, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
    ));
}

// Struct for payload date used in update functions
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct RecipePayload {
    #[validate(length(min = 3))]
    title: String,
    category: String,
    #[validate(length(min = 6))]
    description: String,
    is_community: bool,
    is_for_sale: bool,
    price: u32,
    owner_id: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct UserPayload {
    #[validate(length(min = 3))]
    name: String,
    #[validate(length(min = 3))]
    password: String,
    email: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ReviewPayload {
    recipe_id: u64,
    review: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct InitPayload {
    #[validate(length(min = 3))]
    email: String,
    #[validate(length(min = 4))]
    password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct EditRecipePayload {
    recipe_id: u64,
    title: String,
    description: String,
    is_community: bool,
    is_for_sale: bool,
    price: u32,
    password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct EditCommunityRecipe {
    recipe_id: u64,
    user_id: u64,
    description: String,
}

// Structs for return methods
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ReturnUser {
    id: u64,
    name: String,
    email: String,
    recipes: Vec<u64>,
    balance: u32,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct BuyNftPayload {
    recipe_id: u64,
    user_id: u64,
    password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct FundUser {
    user_id: u64,
    amount: u32,
    password: String,
}

// update function to init contract
#[ic_cdk::update]
fn init_contract(payload: InitPayload) -> Result<Contract, Error> {
    // validate payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    let contract = CONTRACT_STORAGE.with(|s| s.borrow().get(&0));
    if contract.is_some() {
        return Err(Error::AlreadyInit {
            msg: "Contract has already been initialized".to_string(),
        });
    }

    let contract = Contract {
        id: 0,
        email: payload.email,
        password: payload.password,
    };

    match CONTRACT_STORAGE.with(|s| s.borrow_mut().insert(0, contract.clone())) {
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("Could not add recipe email: {}", contract.email),
        }),
        None => Ok(contract),
    }
}

// update function to fund user
#[ic_cdk::update]
fn fund_user(payload: FundUser) -> Result<ReturnUser, Error> {
    let contract = CONTRACT_STORAGE.with(|s| s.borrow().get(&0));
    match contract {
        Some(contract) => {
            if contract.password != payload.password {
                return Err(Error::NotFound {
                    msg: "Invalid password please try again".to_string(),
                });
            }

            let user = USER_STORAGE.with(|u| u.borrow_mut().get(&payload.user_id));

            match user {
                Some(user) => {
                    let new_user = User {
                        id: user.id.clone(),
                        balance: user.balance + payload.amount,
                        ..user
                    };

                    match USER_STORAGE.with(|u| u.borrow_mut().insert(user.id, new_user.clone())) {
                        Some(_) => Ok(ReturnUser {
                            id: new_user.id,
                            name: new_user.name,
                            email: new_user.email,
                            recipes: new_user.recipes,
                            balance: new_user.balance,
                        }),
                        None => Err(Error::NotFound {
                            msg: "Could not fund user balance".to_string(),
                        }),
                    }
                }
                None => Err(Error::NotFound {
                    msg: "User could not be found".to_string(),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: "Contract has not been initialized".to_string(),
        }),
    }
}

// Query function to get all recipes
#[ic_cdk::query]
fn get_all_recipes() -> Result<Vec<Recipe>, Error> {
    // Retrieve all Recipes from the storage
    let recipe_map: Vec<(u64, Recipe)> = RECIPE_STORAGE.with(|s| s.borrow().iter().collect());
    // Extract the Recipes from the tuple and create a vector
    let recipes: Vec<Recipe> = recipe_map.into_iter().map(|(_, recipe)| recipe).collect();

    match recipes.len() {
        0 => Err(Error::NotFound {
            msg: format!("no Recipes found"),
        }),
        _ => Ok(recipes),
    }
}

// Query function to get all for sale recipes
#[ic_cdk::query]
fn get_all_for_sale_recipes() -> Result<Vec<Recipe>, Error> {
    // Retrieve all Recipes from the storage
    let recipe_map: Vec<(u64, Recipe)> = RECIPE_STORAGE.with(|s| s.borrow().iter().collect());
    // Extract the Recipes from the tuple and create a vector
    let recipes: Vec<Recipe> = recipe_map.into_iter().map(|(_, recipe)| recipe).collect();

    // Filter the recipes by category
    let recipes_by_category: Vec<Recipe> = recipes
        .into_iter()
        .filter(|recipe| recipe.is_for_sale && !recipe.is_community)
        .collect();

    // Check if any recipes are found
    match recipes_by_category.len() {
        0 => Err(Error::NotFound {
            msg: format!("No recipes up for sale could be found"),
        }),
        _ => Ok(recipes_by_category),
    }
}

// Get Recipes by category and title content
#[ic_cdk::query]
fn get_recipe_by_category(search: String) -> Result<Vec<Recipe>, Error> {
    let query = search.to_lowercase();
    // Retrieve all Recipes from the storage
    let recipe_map: Vec<(u64, Recipe)> = RECIPE_STORAGE.with(|s| s.borrow().iter().collect());
    // Extract the Recipes from the tuple and create a vector
    let recipes: Vec<Recipe> = recipe_map.into_iter().map(|(_, recipe)| recipe).collect();

    // Filter the recipes by category
    let recipes_by_category: Vec<Recipe> = recipes
        .into_iter()
        .filter(|recipe| {
            (recipe.category).to_lowercase().contains(&query)
                || (recipe.title).to_lowercase().contains(&query)
        })
        .collect();

    // Check if any recipes are found
    match recipes_by_category.len() {
        0 => Err(Error::NotFound {
            msg: format!("no Food recipes for category: {} could be found", query),
        }),
        _ => Ok(recipes_by_category),
    }
}

// get recipe by ID
#[ic_cdk::query]
fn get_recipe_by_id(id: u64) -> Result<Recipe, Error> {
    match RECIPE_STORAGE.with(|recipes| recipes.borrow().get(&id)) {
        Some(recipe) => Ok(recipe),
        None => Err(Error::NotFound {
            msg: format!("recipe of id: {} not found", id),
        }),
    }
}

// Create new Recipe
#[ic_cdk::update]
fn add_recipe(payload: RecipePayload) -> Result<Recipe, Error> {
    // validate payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let price = if payload.is_community {
        0
    } else {
        payload.price
    };

    let recipe = Recipe {
        id,
        title: payload.title.clone(),
        description: payload.description,
        category: payload.category,
        is_community: payload.is_community,
        is_for_sale: payload.is_for_sale,
        price,
        user_id: payload.owner_id,
        reviews: vec![],
    };

    // add recipe to user
    let user_id = payload.owner_id;
    let recipe_id = id;

    match add_recipe_to_owner(user_id, recipe_id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    match RECIPE_STORAGE.with(|s| s.borrow_mut().insert(id, recipe.clone())) {
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("Could not add recipe title: {}", payload.title),
        }),
        None => Ok(recipe),
    }
}

// function to add recipe to user
fn add_recipe_to_owner(user_id: u64, recipe_id: u64) -> Result<(), Error> {
    let user = USER_STORAGE.with(|users| users.borrow().get(&user_id));
    match user {
        Some(user) => {
            let mut new_user_recipes = user.recipes.clone();
            new_user_recipes.push(recipe_id);
            let new_user = User {
                recipes: new_user_recipes,
                ..user
            };
            // update user in storage
            match USER_STORAGE.with(|s| s.borrow_mut().insert(user.id, new_user.clone())) {
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not update user recipes"),
                }),

                Some(_) => Ok(()),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("Could not find recipe Buyer"),
        }),
    }
}

// update function to edit a recipe where only owners of recipes can edit title, is_community, price and description. Non owners can only edit descriptions of communtiy recipes. authorizations is by password
#[ic_cdk::update]
fn edit_owned_recipe(payload: EditRecipePayload) -> Result<Recipe, Error> {
    let recipe = RECIPE_STORAGE.with(|recipes| recipes.borrow().get(&payload.recipe_id));

    match recipe {
        Some(recipe) => {
            if recipe.is_community {
                return Err(Error::InvalidPayload { msg: format!("You can only change descriptions of community Recipes. Try edit_community_recipe method") });
            }
            // get recipe writer, by user_id
            let user = USER_STORAGE.with(|recipes| recipes.borrow().get(&recipe.user_id));
            match user {
                Some(user) => {
                    if user.password == payload.password {
                        let price = if payload.is_community {
                            0
                        } else {
                            payload.price
                        };
                        let new_recipe = Recipe {
                            id: recipe.id,
                            title: payload.title.clone(),
                            description: payload.description,
                            category: recipe.category,
                            is_community: payload.is_community,
                            is_for_sale: payload.is_for_sale,
                            price,
                            user_id: recipe.user_id,
                            reviews: recipe.reviews,
                        };

                        match RECIPE_STORAGE
                            .with(|s| s.borrow_mut().insert(recipe.id, new_recipe.clone()))
                        {
                            Some(_) => Ok(new_recipe),
                            None => Err(Error::InvalidPayload {
                                msg: format!("Could not edit recipe title: {}", payload.title),
                            }),
                        }
                    } else {
                        return Err(Error::Unauthorized {
                            msg: format!("Unathorized, only recipe owner can edit this recipe"),
                        });
                    }
                }
                None => {
                    return Err(Error::NotFound {
                        msg: format!("Recipe owner id: {} could not be found", recipe.user_id),
                    })
                }
            }
        }
        None => Err(Error::NotFound {
            msg: format!("recipe of id: {} not found", payload.recipe_id),
        }),
    }
}

// define update function to edit community recipes
#[ic_cdk::update]
fn edit_community_recipe(payload: EditCommunityRecipe) -> Result<Recipe, Error> {
    let recipe = RECIPE_STORAGE.with(|recipes| recipes.borrow().get(&payload.recipe_id));

    match recipe {
        Some(recipe) => {
            if !recipe.is_community {
                return Err(Error::Unauthorized { msg: format!("This is a private recipe, please contribute to a community based recipe of use the edit_owned_recipe method") });
            }

            let new_recipe = Recipe {
                id: recipe.id,
                title: recipe.title.clone(),
                description: payload.description,
                category: recipe.category,
                is_community: recipe.is_community,
                is_for_sale: recipe.is_for_sale,
                price: recipe.price,
                user_id: recipe.user_id,
                reviews: recipe.reviews,
            };

            match RECIPE_STORAGE.with(|s| s.borrow_mut().insert(recipe.id, new_recipe.clone())) {
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not edit recipe title: {}", recipe.title),
                }),
                Some(_) => Ok(new_recipe),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("recipe of id: {} not found", payload.recipe_id),
        }),
    }
}

// function to buy recipe NFT
#[ic_cdk::update]
fn buy_recipe_nft(payload: BuyNftPayload) -> Result<String, Error> {
    // get recipe
    let recipe = RECIPE_STORAGE.with(|recipes| recipes.borrow().get(&payload.recipe_id));
    // get user
    let user = USER_STORAGE.with(|users| users.borrow().get(&payload.user_id));
    match recipe {
        Some(recipe) => {
            // check if recipe is community
            if recipe.is_community {
                return Err(Error::InvalidPayload {
                    msg: format!("This is a community recipe, you can not buy it"),
                });
            }

            // check if recipe is up for sale
            if !recipe.is_for_sale {
                return Err(Error::InvalidPayload {
                    msg: format!("Sorry, This recipe is not currently for sale"),
                });
            }
            // check if user exists
            match user {
                Some(user) => {
                    // check if the password provided matches users
                    if user.password != payload.password {
                        return Err(Error::Unauthorized {
                            msg: format!("Unauthorized, password does not match, try again"),
                        });
                    }

                    // check if user has enough balance
                    if user.balance < recipe.price {
                        return Err(Error::InvalidPayload {
                            msg: format!("You do not have enough balance to buy this recipe"),
                        });
                    }
                    // check if user is not recipe owner
                    if user.id == recipe.user_id {
                        return Err(Error::InvalidPayload {
                            msg: format!("You can not buy your own recipe"),
                        });
                    }
                    // check if user has already bought recipe
                    if user.recipes.contains(&recipe.id) {
                        return Err(Error::InvalidPayload {
                            msg: format!("You have already bought this recipe"),
                        });
                    }
                    // get recipe owner
                    match transfer_recipe_to_user(payload.user_id, recipe.clone()) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }

                    match RECIPE_STORAGE.with(|r| {
                        r.borrow_mut().insert(
                            recipe.id,
                            Recipe {
                                user_id: user.id,
                                ..recipe
                            },
                        )
                    }) {
                        Some(_) => Ok(format!("Recipe bought successfully, Enjoy !!")),
                        None => Err(Error::NotFound {
                            msg: "could not update recipe user_id".to_string(),
                        }),
                    }
                }
                None => Err(Error::NotFound {
                    msg: format!("user not found"),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("recipe not found"),
        }),
    }
}

// add review to recipe
#[ic_cdk::update]
fn add_review(payload: ReviewPayload) -> Result<Recipe, Error> {
    // get recipe
    let recipe = RECIPE_STORAGE.with(|recipes| recipes.borrow().get(&payload.recipe_id));
    match recipe {
        Some(recipe) => {
            let mut new_reviews = recipe.reviews.clone();
            new_reviews.push(payload.review);
            let new_recipe = Recipe {
                reviews: new_reviews,
                ..recipe
            };

            match RECIPE_STORAGE.with(|s| s.borrow_mut().insert(recipe.id, new_recipe.clone())) {
                Some(_) => Ok(new_recipe),
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not add review to recipe id: {}", recipe.id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("recipe of id: {} not found", payload.recipe_id),
        }),
    }
}

// get recipe reviews by id
#[ic_cdk::query]
fn get_recipe_reviews(id: u64) -> Result<Vec<String>, Error> {
    // get recipe
    let recipe = RECIPE_STORAGE.with(|recipes| recipes.borrow().get(&id));
    match recipe {
        Some(recipe) => Ok(recipe.reviews),
        None => Err(Error::NotFound {
            msg: format!("recipe of id: {} not found", id),
        }),
    }
}

fn transfer_recipe_to_user(user_id: u64, recipe: Recipe) -> Result<(), Error> {
    let recipe_owner = USER_STORAGE.with(|users| users.borrow().get(&recipe.user_id));
    let recipe_buyer = USER_STORAGE.with(|users| users.borrow().get(&user_id));

    match recipe_owner {
        Some(recipe_owner) => {
            let mut new_user_recipes = recipe_owner.recipes.clone();
            new_user_recipes = new_user_recipes
                .into_iter()
                .filter(|r| *r != recipe.id)
                .collect();
            // update recipe owner balance and remove bought recipe
            let new_recipe_owner = User {
                recipes: new_user_recipes,
                balance: recipe_owner.balance + recipe.price,
                ..recipe_owner
            };
            // update recipe owner in storage
            match USER_STORAGE.with(|s| {
                s.borrow_mut()
                    .insert(recipe_owner.id, new_recipe_owner.clone())
            }) {
                Some(_) => {
                    // update user balance
                    match recipe_buyer {
                        Some(recipe_buyer) => {
                            let mut new_user_recipes = recipe_buyer.recipes.clone();
                            new_user_recipes.push(recipe.id);
                            let new_user = User {
                                recipes: new_user_recipes,
                                balance: recipe_buyer.balance - recipe.price.clone(),
                                ..recipe_buyer
                            };
                            // update user balance in storage
                            match USER_STORAGE
                                .with(|s| s.borrow_mut().insert(recipe_buyer.id, new_user.clone()))
                            {
                                Some(_) => Ok(()),
                                None => Err(Error::InvalidPayload {
                                    msg: format!("Could not update user balance"),
                                }),
                            }
                        }
                        None => Err(Error::NotFound {
                            msg: format!("Could not find recipe Buyer"),
                        }),
                    }
                }
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not update recipe owner balance"),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("Could not find recipe owner"),
        }),
    }
}

// Define query function to get a user by ID
#[ic_cdk::query]
fn get_user(id: u64) -> Result<ReturnUser, Error> {
    match USER_STORAGE.with(|users| users.borrow().get(&id)) {
        Some(user) => Ok(ReturnUser {
            id: user.id,
            name: user.name,
            email: user.email,
            recipes: user.recipes,
            balance: user.balance,
        }),
        None => Err(Error::NotFound {
            msg: format!("user id:{} does not exist", id),
        }),
    }
}

// Update function to add a user
#[ic_cdk::update]
fn add_user(payload: UserPayload) -> Result<ReturnUser, Error> {
    // validate payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let user = User {
        id,
        name: payload.name.clone(),
        email: payload.email,
        password: payload.password,
        recipes: vec![],
        balance: 0,
    };

    let return_user = ReturnUser {
        id,
        name: user.name.clone(),
        email: user.email.clone(),
        recipes: user.recipes.clone(),
        balance: user.balance.clone(),
    };

    match USER_STORAGE.with(|s| s.borrow_mut().insert(id, user.clone())) {
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("Could not add user name: {}", payload.name),
        }),
        None => Ok(return_user),
    }
}

// Define an Error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    AlreadyInit { msg: String },
    InvalidPayload { msg: String },
    Unauthorized { msg: String },
}

// Candid generator for exporting the Candid interface
ic_cdk::export_candid!();
