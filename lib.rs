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
        founders_completed: ink_storage::Mapping<AccountId, bool>,
        founders_amount_needed: ink_storage::Mapping<AccountId, u128>,
        founders_amount_funded: ink_storage::Mapping<AccountId, u128>,
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
                contract.initial_founder = caller;
                contract.founders.push(caller);
                contract.founders_required.insert(caller, &true);
                contract.founders_amount_needed.insert(caller, &initial_founder_picos_needed);
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
        pub fn get(&self) -> u64 {
            self.value
        }

        
        #[ink(message, payable, selector = 0xC4577B10)]
        pub fn fund_tribe(&mut self) {
            assert!(self.defunct == false, "tribe is defunct.");
            assert!(self.enabled == false, "tribe is already enabled.");
            let caller = self.env().caller();
            let value = self.env().transferred_value();
            ink_env::debug_println!(
                "received payment: {} from {:?}",
                self.env().transferred_value(),
                caller
            );
            // find founder in founders_amount_needed

            let mut is_found = false;
            for founder in &self.founders {
                if founder == &caller {
                    is_found = true;
                }
            }

            // if not found, fail
            assert!(is_found == true, "caller is not a founder.");

            // find founders_funded, if true, fail
            let is_completed = match self.founders_completed.get(caller) {
                Some(v) => v,
                None => false
            };

            assert!(is_completed == false, "founder is already funded or completed.");

            // if found, find founders_amount_needed > founders_amount_funded
            let amount_needed = match self.founders_amount_needed.get(caller) {
                Some(v) => v,
                None => 0,
            };

            let mut amount_funded = match self.founders_amount_funded.get(caller) {
                Some(v) => v,
                None => 0
            };

            assert!(amount_needed >= amount_funded, "no more units needed for user");

            // OK, get amount units, and add to founders_amount_funded
            amount_funded += value;
            self.founders_amount_funded.insert(caller, &amount_funded);
            
            // if founders_amount_funded >= founders_amount_needed
            if amount_funded >= amount_needed {
                // we're done! set to true
                self.founders_completed.insert(caller, &true);
            }

            let mut completed_founders_count = 0;

            let all_founders_count = self.founders.len() as i32;

            for founder in &self.founders {
                let f_is_completed = match self.founders_completed.get(founder) {
                    Some(v) => v,
                    None => false
                };

                if f_is_completed == true {
                    completed_founders_count += 1;
                }
            }

            // if ALL founders_funded == true
            if completed_founders_count == all_founders_count {
                // ACTIVATE TRIBE BY SETTING enabled = true
                self.enabled = true;
            }
            

            // calculate differences, send difference back to each founder
            // Dont implement this yet. 

            
        }

        #[ink(message)]
        pub fn reject_tribe(&mut self) {
            let caller = self.env().caller();

            // if defunct, fail
            assert!(self.defunct == false, "tribe is defunct.");
            assert!(self.enabled == false, "tribe is already enabled.");
            
            // if founder does NOT exist in founders_required, fail
            let mut is_found = false;
            for founder in &self.founders {
                if founder == &caller {
                    is_found = true;
                }
            }

            // if not found, fail
            assert!(is_found == true, "account is not a founder.");
            
            // find founder in founders_rejected, founders_required
            let founder_rejected = match self.founders_rejected.get(caller) {
                Some(v) => v,
                None => false
            };

            // if founder is already in rejected, fail
            assert!(founder_rejected == false, "founder already rejected.");

            let founder_required = match self.founders_required.get(caller) {
                Some(v) => v, 
                None => false
            };

            // set founders_rejected for AccountID
            self.founders_rejected.insert(caller, &true);

            if founder_required == true {
                // if req'd, defunct tribe.
                self.defunct = true;
            } else {
                self.founders_completed.insert(caller, &true);
                let mut completed_founders_count = 0;
                let all_founders_count = self.founders.len() as i32;
    
                for founder in &self.founders {
                    let f_is_completed = match self.founders_completed.get(founder) {
                        Some(v) => v,
                        None => false
                    };
    
                    if f_is_completed == true {
                        completed_founders_count += 1;
                    }
                }
    
                // if ALL founders_funded == true
                if completed_founders_count == all_founders_count {
                    // ACTIVATE TRIBE BY SETTING enabled = true
                    self.enabled = true;
                }
    
    
            }

            // if any founder has already funded tribe, return funds to each founder
            // TODO
            
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

            let is_completed = match self.founders_completed.get(founder) {
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

            let is_rejected = match self.founders_rejected.get(founder) {
                Some(v) => v,
                None => false
            };

            if is_founder == true {
                ink_prelude::format!(
                    "Is Initial: {} Required: {} Is Rejected: {} Is Completed: {} Amount Needed: {} Amount Funded: {}",
                    is_initial,
                    is_required,
                    is_rejected,
                    is_completed,
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
        pub fn add_founder(&mut self, founder: AccountId, picos: u128, required: bool) {
            let caller = Self::env().caller();
            
            // is the caller the initial_founder?
            assert!(&caller == &self.initial_founder, "You are not the initial founder.");
            
            // if yes, is the tribe active/defunct? 
            assert!(&self.defunct == &false, "Tribe is already defunct.");

            assert!(&self.enabled == &false, "Tribe is already enabled.");

            // get all accountIDs 

            for fnd in &self.founders {
                // is founder already in the founder list?
                assert!(&founder != fnd, "founder already exists.");

                // has any founder rejected? any amount funded?

                let founder_reject = match self.founders_rejected.get(&founder) {
                    Some(v) => v,
                    None => false
                };

                let founder_completed = match self.founders_completed.get(&founder) {
                    Some(v) => v,
                    None => false
                };

                let founder_amount_funded = match self.founders_amount_funded.get(&founder) {
                    Some(v) => v,
                    None => 0,
                };

                assert!(founder_reject == false, "a founder has already rejected tribe.");
                assert!(founder_completed == false, "a founder has already funded/completed tribe actions.");
                assert!(founder_amount_funded == 0, "a founder already sent funds to tribe");
            }

            // we got this far, add the founder.

            self.founders.push(founder);
            self.founders_required.insert(founder, &required);
            self.founders_rejected.insert(founder, &false);
            self.founders_completed.insert(founder, &false);
            self.founders_amount_needed.insert(founder, &picos);
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
            let mut inkrement = Inkrement::new("hey".to_string(), 1);
            assert_eq!(inkrement.get(), 0);
            inkrement.inc();
            assert_eq!(inkrement.get(), 1);
        }
    }
}
