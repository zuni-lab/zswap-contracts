use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::{ log, near_bindgen };

const DEFAULT_LOCK_MESSAGE: &str = "LOK";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LockContract {
    unlocked: bool,
}

impl Default for LockContract {
    fn default() -> Self {
        Self { unlocked: false }
    }
}

impl LockContract {
    pub fn lock(&mut self, f: impl FnOnce()) {
        assert!(self.unlocked, "{}", DEFAULT_LOCK_MESSAGE);

        self.unlocked = false;
        f();
        self.unlocked = true;
    }
}

#[near_bindgen]
impl LockContract {
    pub fn some_thing(&mut self) {
        self.lock(|| {
            log!("do something");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[should_panic(expected = "LOK")]
    fn test_lock_panic() {
        let mut lock_contract = LockContract::default();
        lock_contract.lock(|| {
            panic!("LOK");
        });
    }
}
