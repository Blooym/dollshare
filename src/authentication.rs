#[derive(Debug)]
pub struct Authentication {
    valid_tokens: Vec<String>,
}

#[derive(PartialEq, Eq)]
pub enum AuthenticationState {
    Valid,
    Invalid,
}

impl AuthenticationState {
    pub fn is_valid(&self) -> bool {
        *self == AuthenticationState::Valid
    }
}

impl Authentication {
    pub fn new(valid_tokens: Vec<String>) -> Self {
        Self { valid_tokens }
    }

    pub fn state_for_token(&self, token: &str) -> AuthenticationState {
        match self.valid_tokens.iter().any(|f| f == token) {
            true => AuthenticationState::Valid,
            false => AuthenticationState::Invalid,
        }
    }
}
