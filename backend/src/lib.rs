use candid::Principal;
use ic_cdk::candid::{CandidType, Deserialize};
use ic_cdk_macros::{candid_method, query, update, init};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::RwLock;

// A simple in-memory user storage (should be replaced with a more persistent solution)
static mut USERS: Option<HashMap<Principal, String>> = None;

#[ic_cdk::update]
fn register_user(user_id: Principal, username: String) -> String {
    unsafe {
        let users = USERS.get_or_insert_with(HashMap::new);
        if users.contains_key(&user_id) {
            return format!("User already registered: {}", user_id);
        }
        users.insert(user_id, username.clone());
        format!("User registered successfully: {}", username)
    }
}

#[ic_cdk::query]
fn get_user(user_id: Principal) -> Option<String> {
    unsafe {
        let users = USERS.get_or_insert_with(HashMap::new);
        users.get(&user_id).cloned()
    }
}

#[derive(CandidType, Deserialize, Clone)]
enum PropertyType {
    RealEstate,
    Car,
    Art,
    Other(String), // For any other property categories
}

#[derive(CandidType, Deserialize, Clone)]
struct Property {
    id: String,
    property_type: PropertyType,
    image_hash: String,
    // Additional fields that could apply to all property types
    description: String,
    owner: String,
}

#[derive(Default)]
struct DecentralizedPlatform {
    properties: HashMap<String, Property>,
}

// Global thread-safe state using thread-local storage and RwLock
thread_local! {
    static DECENTRALIZED_PLATFORM: RwLock<DecentralizedPlatform> = RwLock::new(DecentralizedPlatform::default());
}

// Initialize the canister
#[init]
fn init() {
    // No explicit initialization required here
}

// Function to upload an image, hash it, and store property data
#[update]
fn upload_property(
    property_id: String,
    property_type: PropertyType,
    image_data: Vec<u8>,
    description: String,
    owner: String,
) -> String {
    // Validate image data
    if image_data.is_empty() {
        ic_cdk::trap("Image data is empty.");
    }

    // Hash the image data using SHA-256
    let hash = hash_image(&image_data);

    // Create a new Property with the given ID, type, and other details
    let property = Property {
        id: property_id.clone(),
        property_type,
        image_hash: hash.clone(),
        description,
        owner,
    };

    // Safely store the property in the decentralized platform
    DECENTRALIZED_PLATFORM.with(|platform| {
        let mut platform = platform.write().unwrap();
        platform.properties.insert(property_id, property);
    });

    // Return the image hash
    hash
}

// Helper function to hash image data using SHA-256
fn hash_image(image_data: &Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(image_data);
    let result = hasher.finalize();
    hex::encode(result) // Convert the hash to a hexadecimal string
}

// Query function to get all property IDs and their associated image hashes
#[query]
fn get_properties() -> Vec<(String, String, String)> {
    DECENTRALIZED_PLATFORM.with(|platform| {
        let platform = platform.read().unwrap();
        platform
            .properties
            .iter()
            .map(|(id, property)| (id.clone(), format!("{:?}", property.property_type), property.image_hash.clone()))
            .collect()
    })
}

// Query function to get a specific property's details by ID
#[query]
fn get_property_by_id(property_id: String) -> Option<Property> {
    DECENTRALIZED_PLATFORM.with(|platform| {
        let platform = platform.read().unwrap();
        platform.properties.get(&property_id).cloned()
    })
}
