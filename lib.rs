#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod inkrement {

    use ink_storage::{traits::SpreadAllocate};
    use ink_prelude::{string::String, vec::Vec};
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Inkrement {
        /// Stores a single `bool` value on the storage.
        value: u64,
        enabled: bool,
        defunct: bool,
        name: String,
        initial_founder: AccountId,
        founders: Vec<AccountId>,
        founders_required: ink_storage::Mapping<AccountId, bool>,
        founders_rejected: ink_storage::Mapping<AccountId, bool>,
        founders_funded: ink_storage::Mapping<AccountId, bool>,
        founders_amount_needed: ink_storage::Mapping<AccountId, u128>,
        founders_amount_funded: ink_storage::Mapping<AccountId, u128>,
    }

    impl Inkrement {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor, payable)]
        pub fn new(init_name: String, initial_founder_required: bool, initial_founder_amount_needed: u128) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                let caller = Self::env().caller();
                contract.name = init_name;
                contract.enabled = false;
                contract.defunct = false;
                contract.initial_founder = caller;
                contract.founders.push(caller);
                contract.founders_required.insert(caller, &initial_founder_required);
                contract.founders_amount_needed.insert(caller, &initial_founder_amount_needed);
                contract.founders_amount_funded.insert(caller, &0);
            })
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::utils::initialize_contract(|_| {})
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn inc(&mut self) {
            self.value += 1;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get_all(&self) -> String {
            ink_prelude::format!(
                "Value: {}, Name: {}, Enabled: {}, Defunct: {}, InitialFounder: {:?} Founders: {:?} Req'd: {:?} Rej'D {:?} Funded {:?} AmountN: {:?} AmountF: {:?}",
                &self.value,
                &self.name,
                &self.enabled,
                &self.defunct,
                &self.initial_founder,
                &self.founders,
                &self.founders_required,
                &self.founders_rejected,
                &self.founders_funded,
                &self.founders_amount_needed,
                &self.founders_amount_funded,
            )
        }

        #[ink(message)]
        pub fn get(&self) -> u64 {
            self.value
        }

        
        #[ink(message, payable, selector = 0xCAFEBABE)]
        pub fn fund_tribe(&self) {
            let caller = Self::env().caller();
            ink_env::debug_println!(
                "received payment: {} from {:?}",
                self.env().transferred_value(),
                caller
            );

            // find founder in founders_amount_needed
            // if not found, fail
            // find founders_funded, if true, fail
            // if found, find founders_amount_needed > founders_amount_funded
            // OK, get amount units, and add to founders_amount_funded
            // if founders_amount_funded >= founders_amount_needed
            // set founders_funded to TRUE


            // if ALL founders_funded == true

            // calculate differences, send difference back to each founder
            // Dont implement this yet. 

            // ACTIVATE TRIBE BY SETTING enabled = true
        }

        #[ink(message)]
        pub fn reject_tribe(&self) {
            // let caller = Self::env().caller();

            // if defunct, fail
            
            // if founder does NOT exist in founders_required, fail
            
            // find founder in founders_rejected, founders_required

            // if founder is already in rejected, fail

            // set founders_rejected for AccountID

            // if any founder has already funded tribe, return funds to each founder
            
            // if founder is in required, set tribe to defunct
            
        }

        #[ink(message)]
        pub fn get_founder_status(&self, founder: AccountId) -> String {

            // get founder details -- is initial founder?
            let is_initial = founder == self.initial_founder;
            let is_founder = match self.founders_required.get(founder) {
                Some(_) => true,
                None => false
            };
            let is_required = match self.founders_required.get(founder) {
                Some(v) => v,
                None => false
            };

            let is_funded = match self.founders_funded.get(founder) {
                Some(v) => v,
                None => false
            };

            let amount_needed = match self.founders_amount_needed.get(founder) {
                Some(v) => v,
                None => 0
            };

            let amount_funded = match self.founders_amount_funded.get(founder) {
                Some(v) => v,
                None => 0
            };

            if is_founder == true {
                ink_prelude::format!(
                    "Is Initial: {} Required: {} Is Funded: {} Amount Needed: {} Amount Funded: {}",
                    is_initial,
                    is_required,
                    is_funded,
                    amount_needed,
                    amount_funded

                    
                )
            } else {
                ink_prelude::format!(
                    "Account is NOT a founder."
                )
            }
            
            
        }


        #[ink(message)]
        pub fn add_founder(&mut self, founder: AccountId, units: u128, required: bool) {
            let caller = Self::env().caller();
            
            // is the caller the initial_founder?
            assert!(&caller == &self.initial_founder, "You are not the initial founder.");
            
            // if yes, is the tribe active/defunct? 
            assert!(&self.defunct == &false, "Tribe is already defunct.");

            // get all accountIDs 

            for fnd in &self.founders {
                // is founder already in the founder list?
                assert!(&founder != fnd, "founder already exists.");

                // has any founder rejected? any amount funded?

                let founder_reject = match self.founders_rejected.get(&founder) {
                    Some(v) => v,
                    None => false
                };

                let founder_funded = match self.founders_funded.get(&founder) {
                    Some(v) => v,
                    None => false
                };

                let founder_amount_funded = match self.founders_amount_funded.get(&founder) {
                    Some(v) => v,
                    None => 0,
                };

                assert!(founder_reject == false, "a founder has already rejected tribe.");
                assert!(founder_funded == false, "a founder has already funded tribe.");
                assert!(founder_amount_funded == 0, "a founder already sent funds to tribe");
            }

            // we got this far, add the founder.

            self.founders.push(founder);
            self.founders_required.insert(founder, &required);
            self.founders_rejected.insert(founder, &false);
            self.founders_funded.insert(founder, &false);
            self.founders_amount_needed.insert(founder, &units);
            self.founders_amount_funded.insert(founder, &0);

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
            let inkrement = Inkrement::default();
            assert_eq!(inkrement.get(), 0);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn inkrement_val_can_be_incremented() {
            let mut inkrement = Inkrement::new("hey".to_string(), false, 0);
            assert_eq!(inkrement.get(), 0);
            inkrement.inc();
            assert_eq!(inkrement.get(), 1);
        }
    }
}
