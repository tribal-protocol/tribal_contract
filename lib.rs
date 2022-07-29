#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;

mod founder;

#[ink::contract]
mod inkrement {
    use ink_storage::traits::{SpreadAllocate};
    use ink_prelude::{string::String, vec::Vec};
    use crate::founder::*;

    pub const FOUNDER_REJECTED: i32 = -1;
    pub const FOUNDER_PENDING: i32 = 0;
    pub const FOUNDER_ACCEPTED: i32 = 1;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Inkrement {
        /// Stores a single `bool` value on the storage.
        enabled: bool,
        defunct: bool,
        name: String,
        founders: ink_storage::Mapping<u32, Vec<Founder>>
    }

    impl Inkrement {
        /// Constructor that initializes the tribe with a given `init_name`, `initial_founder_picos_needed` must not be 0
        #[ink(constructor, payable)]
        pub fn new(init_name: String, initial_founder_picos_needed: u128) -> Self {
            assert!(initial_founder_picos_needed != 0, "initial founder picos must not be 0");
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                let caller = Self::env().caller();
                contract.name = init_name;
                contract.enabled = false;
                contract.defunct = false;
                contract.founders.insert(0, &ink_prelude::vec![Founder {
                    // caller
                    id: caller,
                    initial: true,
                    required: true,
                    vote_action: FOUNDER_PENDING,
                    amount_promised: initial_founder_picos_needed,
                    amount_funded: 0
                }]);
            })
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::utils::initialize_contract(|_| {})
        }

        fn get_founder_list(&self) -> Vec<Founder> {
            self.founders.get(0).expect("could not get founder list!")
        }

        fn get_founder_index(&self, founder_id: AccountId) -> Option<usize> {
            let mut index: usize = 0;
            for founder in self.get_founder_list() {
                if founder_id == founder.id {
                    return Some(index);
                }
                index += 1;
            }
            return None;
        }

        fn check_and_activate_tribe(&mut self) {
            if self.enabled && !self.defunct {
                return;
            }

            let all_founders = self.get_founder_list();
            for founder in all_founders {
                if !founder.is_completed() {
                    return;
                }
            }

            self.enabled = true;
        }


        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get_tribe(&self) -> String {

            // get all founders
            // BASE58ENCODE ACCOUNTID! 

            ink_prelude::format!(
                "Name: {}, Enabled: {}, Defunct: {}",
                &self.name,
                &self.enabled,
                &self.defunct
            )
        }


        #[ink(message)]
        pub fn accept_tribe(&mut self) {
            assert!(self.defunct == false, "tribe is defunct.");
            assert!(self.enabled == false, "tribe is already enabled.");

            let caller = self.env().caller();
            let mut founders = self.get_founder_list();
            let founder_index = self.get_founder_index(caller).expect("caller is not a founder");
            let founder = &founders[founder_index];
            assert!(!founder.is_rejected(), "founder has already rejected tribe.");

            // we got this far, set action to ACCEPTED
            founders[founder_index].vote_action = FOUNDER_ACCEPTED;

            self.founders.insert(0, &founders);
        }

        #[ink(message, payable, selector = 0xC4577B10)]
        pub fn fund_tribe(&mut self) {
            assert!(self.defunct == false, "tribe is defunct.");
            assert!(self.enabled == false, "tribe is already enabled.");
            let caller = self.env().caller();
            let value = self.env().transferred_value();
            
            ink_env::debug_println!(
                "received payment: {} from {:?}",
                value,
                caller
            );

            let mut founders = self.get_founder_list();
            let founder_index = self.get_founder_index(caller).expect("caller is not a founder!");

            assert!(founders[founder_index].is_accepted(), "founder has not accepted tribe.");

            founders[founder_index].fund(value);

            self.founders.insert(0, &founders);
            
            self.check_and_activate_tribe();
            

            // calculate differences, send difference back to each founder
            // Dont implement this yet. 
        }

        #[ink(message)]
        pub fn reject_tribe(&mut self) {
            let caller = self.env().caller();

            // if defunct or enabled, fail
            assert!(self.defunct == false, "tribe is defunct.");
            assert!(self.enabled == false, "tribe is already enabled.");
            
            // if founder does NOT exist in founders_required, fail
            let mut founders = self.get_founder_list();
            let founder_index = self.get_founder_index(caller).expect("caller is not a founder.");

            founders[founder_index].vote_action = FOUNDER_REJECTED;

            if founders[founder_index].required {
                self.defunct = true;

                // TODO REFUND EVERYONE WHO SENT UNITS
            } else {
                self.check_and_activate_tribe();
            }

            self.founders.insert(0, &founders);
            // if any founder has already funded tribe, return funds to each founder
            // TODO

        }

        #[ink(message)]
        pub fn get_founder_status(&self, founder: AccountId) -> String {
            let founders = self.get_founder_list();
            let founder_index = match self.get_founder_index(founder) {
                Some(v) => v,
                None => {
                    return ink_prelude::format!("Account is not a founder!");
                }
            };

            founders[founder_index].describe()
        }

        #[ink(message)]
        pub fn add_founder(&mut self, potential_founder: AccountId, picos: u128, required: bool) {
            let caller = Self::env().caller();

            let mut founders = self.get_founder_list();
            let founder_index = self.get_founder_index(caller).expect("caller is not a founder.");
            let initial_founder = &founders[founder_index];

            // is the caller the initial_founder?
            assert!(&initial_founder.initial, "You are not the initial founder.");
            
            // if yes, is the tribe active/defunct? 
            assert!(&self.defunct == &false, "Tribe is already defunct.");
            assert!(&self.enabled == &false, "Tribe is already enabled.");

            // get all accountIDs 

            for founder in &founders {
                // is founder already in the founder list?
                assert!(founder.id != potential_founder, "founder already exists.");

                // has any founder rejected? any amount funded?
                if founder.is_rejected() || founder.is_completed() || founder.is_accepted() || founder.amount_funded > 0 {
                    panic!("a founder has already performed an action against tribe.");
                }
            }

            // we got this far, add the founder.
            founders.push(Founder {
                id: potential_founder,
                initial: false,
                required: required,
                vote_action: FOUNDER_PENDING,
                amount_funded: 0,
                amount_promised: picos
            });

            self.founders.insert(0, &founders);
        

        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn inkrement_val_instantiates_correctly() {
            //let inkrement = Inkrement::default();
            // assert_eq!(inkrement.get(), 0);
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
}
    