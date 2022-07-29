
use ink_env::AccountId;
use ink_storage::traits::{SpreadLayout, PackedLayout};
use ink_prelude::{string::String};
use crate::inkrement::{FOUNDER_ACCEPTED, FOUNDER_REJECTED, FOUNDER_PENDING};

#[derive(PackedLayout, SpreadLayout, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub struct Founder {
    pub id: AccountId,
    pub initial: bool,
    pub required: bool,
    pub vote_action: i32,
    pub amount_promised: u128,
    pub amount_funded: u128,
}

impl Founder {
    pub fn initial_founder(id: AccountId, amount_promised: u128) -> Self {
       Founder {
            id: id,
            initial: true,
            required: true,
            vote_action: FOUNDER_PENDING,
            amount_promised: amount_promised,
            amount_funded: 0
        }
    }

    pub fn is_accepted(&self) -> bool {
       return self.vote_action == FOUNDER_ACCEPTED;
    }

    pub fn fund(&mut self, amount: u128) {
        if self.amount_funded >= self.amount_promised {
            panic!("user is alredy funded");
        }
        self.amount_funded += amount;
    }

    pub fn is_funded(&self) -> bool {
        self.amount_funded >= self.amount_promised
    }

    pub fn is_rejected(&self) -> bool {
        self.vote_action == FOUNDER_REJECTED
    }

    pub fn is_completed(&self) -> bool {

        if !self.required && self.is_rejected() {
            return true;
        } else if self.is_funded() {
            return true;
        }
        return false;
    }

    pub fn describe(&self) -> String {
        ink_prelude::format!(
            "Is Initial: {} Required: {} Is Rejected: {} Is Completed: {} Amount Promised: {} Total Amount Funded: {}",
            self.initial,
            self.required,
            self.is_rejected(),
            self.is_completed(),
            self.amount_promised,
            self.amount_funded
        )
    }
}


/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
/// module and test functions are marked with a `#[test]` attribute.
/// The below code is technically just normal Rust code.
#[cfg(test)]
mod founder_tests {
    use crate::inkrement::{FOUNDER_ACCEPTED, FOUNDER_REJECTED, FOUNDER_PENDING};

    /// Imports all the definitions from the outer scope so we can use them here.
    use super::*;

    /// Imports `ink_lang` so we can use `#[ink::test]`.
    use ink_lang as ink;

    /// We test if the default constructor does its job.
    #[ink::test]
    fn founder_can_create () {
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        // let bob = AccountId::try_from([0x1; 32]).unwrap();
        // let charlie = AccountId::try_from([0x2; 32]).unwrap();  
       let founder = Founder {
           id: alice,
           initial: true,
           required: true,
           vote_action: FOUNDER_PENDING,
           amount_promised: 1234,
           amount_funded: 0
       };
       assert_eq!(founder.id, alice);
    }

    /// We test a simple use case of our contract.
    #[ink::test]
    fn inkrement_val_can_be_incremented() {
        //let mut inkrement = Inkrement::new("hey".to_string(), 1);
        // assert_eq!(inkrement.get(), 0);
        // inkrement.inc();
        // assert_eq!(inkrement.get(), 1);
    }
}
