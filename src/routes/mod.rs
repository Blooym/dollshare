mod health;
mod index;
pub mod uploads;
pub use health::*;
pub use index::*;

fn authentication_valid(bearer_token: &str, configured_tokens: &Vec<String>) -> bool {
    configured_tokens.iter().any(|f| f == bearer_token)
}
