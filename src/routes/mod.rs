mod health;
mod index;
pub mod uploads;
pub use health::*;
pub use index::*;

fn authentication_valid(bearer_token: &str, configured_token: &str) -> bool {
    bearer_token == configured_token
}
