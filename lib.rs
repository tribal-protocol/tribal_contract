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

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct TribeContract {
        /// Stores a single `bool` value on the storage.
        enabled: bool,
        defunct: bool,
        name: String,
        founders: ink_storage::Mapping<u32, Vec<Founder>>
    }

    impl TribeContract {
        /// Constructor that initializes the tribe with a given `init_name`, `initial_founder_picos_needed` must not be 0
        #[ink(constructor, payable)]
        pub fn new(init_name: String, initial_founder_picos_needed: u128) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                let caller = Self::env().caller();
                contract.name = init_name;
                contract.enabled = false;
                contract.defunct = false;                
                contract.founders.insert(0, &ink_prelude::vec![
                    Founder::initial_founder(caller, initial_founder_picos_needed).expect("expected founder")
                ]);
            })
        }

        fn activate_tribe(&mut self) {
            if self.enabled && !self.defunct {
                return;
            }

            let all_founders = self.get_founder_list();
            for founder in all_founders {
                if founder.has_pending_activity() {
                    return;
                }
            }

            self.enabled = true;
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

        #[ink(message)]
        pub fn accept_tribe(&mut self) {
            assert!(self.defunct == false, "tribe is defunct");
            assert!(self.enabled == false, "tribe is already enabled");

            let caller = self.env().caller();
            let mut founders = self.get_founder_list();
            let founder_index = self.get_founder_index(caller).expect("caller is not a founder");
            let founder = &founders[founder_index];
            assert!(!founder.is_rejected(), "founder has already rejected tribe");

            // we got this far, set action to ACCEPTED
            founders[founder_index].vote_action = FOUNDER_ACCEPTED;

            self.founders.insert(0, &founders);
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
                if !founder.has_pending_activity() || founder.has_funds() {
                    panic!("a founder has already performed an action against tribe.");
                }
            }

            // we got this far, add the founder.
            founders.push(Founder::new(potential_founder, required, picos).expect("expected founder"));

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

            founders[founder_index].fund(value);

            self.founders.insert(0, &founders);
            
            self.activate_tribe();
            

            // calculate differences, send difference back to each founder
            // Dont implement this yet. 
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
                self.activate_tribe();
            }

            self.founders.insert(0, &founders);
            // if any founder has already funded tribe, return funds to each founder
            // TODO

        }
    }

    #[cfg(test)]
    mod tests {        
        use super::*;        
        use ink_lang as ink;

        const NAME: &str = "a test tribe";

        #[ink::test]
        fn create_tribe_success() {
            //ACT
            let tribe = TribeContract::new(NAME.to_string(), 5000);

            //ASSERT
            assert_eq!(tribe.name, NAME.to_string());
            assert!(!tribe.enabled);
            assert!(!tribe.defunct);
        }

        #[ink::test]
        fn create_tribe_contains_only_initial_founder() {
            //ASSIGN
            let tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            assert!(tribe.founders.contains(0));
            let founder_vec = tribe.founders.get(0).expect("expected vector of founders");

            //ASSERT
            assert_eq!(tribe.name, NAME.to_string());
            assert_eq!(founder_vec.len(), 1);
            assert!(founder_vec[0].initial);  //assert only member of newly started tribe is the initial founder
        }
    
        #[ink::test]
        fn activate_tribe_with_no_activity_should_have_no_effect() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.activate_tribe();

            //ASSERT
            assert!(tribe.enabled == false)
        }
    
        #[ink::test]
        fn active_tribe_with_funded_activity_should_enable() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            /* Update Alice in founders list to accept and fully fund tribe */
            let alice_index = tribe.get_founder_index(alice).expect("alice should be initial founder");
            let mut founders = tribe.get_founder_list();
            founders[alice_index].vote_action = FOUNDER_ACCEPTED;
            founders[alice_index].fund(5000);
            tribe.founders.insert(0, &founders);

            //ACT
            let prev_enabled = tribe.enabled;
            tribe.activate_tribe();

            //ASSERT
            assert!(prev_enabled == false);
            assert!(tribe.enabled);
        }

        #[ink::test]
        fn get_founder_list_should_return_vec() {
            //ASSIGN
            let tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            let founder_list = tribe.get_founder_list();

            //ASSERT
            assert_eq!(founder_list.len(), 1);
        }

        #[ink::test]
        fn get_founder_index_should_return_initial_founder_index() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);

            //ACT
            let tribe = TribeContract::new(NAME.to_string(), 5000);
            match tribe.get_founder_index(alice) {
                Some(index) => {
                    let founders = tribe.get_founder_list();
                    let alice_founder = &founders[index];

                    //ASSERT
                    assert_eq!(alice_founder.id, alice);
                },
                None => panic!("founder index not found"),
            } 
        }

        #[ink::test]
        fn get_founder_index_should_not_find_index() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            let bob= AccountId::from([0x1; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);

            //ACT
            let tribe = TribeContract::new(NAME.to_string(), 5000);
            match tribe.get_founder_index(bob) {
                Some(_) => assert!(false), 
                None => assert!(true),
            } 
        }

        #[ink::test]
        #[should_panic(expected = "caller is not a founder")]   
        fn accept_tribe_should_fail_when_caller_is_not_founder() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            let bob = AccountId::from([0x1; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            tribe.accept_tribe();
        }

        #[ink::test]
        #[should_panic(expected = "founder has already rejected tribe")]   
        fn accept_tribe_should_fail_when_caller_already_rejected_tribe() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            match tribe.get_founder_index(alice) {
                Some(index) => {
                    // Mark founder as rejected
                    let mut founders = tribe.get_founder_list();
                    founders[index].vote_action = FOUNDER_REJECTED;
                    tribe.founders.insert(0, &founders);

                    tribe.accept_tribe();        
                },
                None => panic!("founder index not found"),
            } 
        }
    }
}
    