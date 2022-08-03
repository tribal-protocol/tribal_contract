#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;

mod founder;

#[ink::contract]
mod tribe {
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
            // is the tribe active/defunct? 
            assert!(self.defunct == false, "tribe is defunct");
            assert!(self.enabled == false, "tribe is already active");

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
            // is the tribe active/defunct? 
            assert!(self.defunct == false, "tribe is defunct");
            assert!(self.enabled == false, "tribe is already active");

            let caller = Self::env().caller();

            let mut founders = self.get_founder_list();
            let founder_index = self.get_founder_index(caller).expect("caller is not a founder");
            let initial_founder = &founders[founder_index];

            // is the caller the initial_founder?
            assert!(&initial_founder.initial, "caller is not the initial founder");
            
            // itterate all AccountIds 
            for founder in &founders {
                // is founder already in the founder list?
                assert!(founder.id != potential_founder, "founder already exists");

                // has any founder rejected? any amount funded?
                if !founder.has_pending_activity() || founder.has_funds() {
                    panic!("tribe is locked due to founder activity");
                }
            }

            // we got this far, add the founder.
            founders.push(Founder::new(potential_founder, required, picos).expect("expected founder"));

            self.founders.insert(0, &founders);        

        }

        #[ink(message, payable, selector = 0xC4577B10)]
        pub fn fund_tribe(&mut self) {
            // is the tribe active/defunct? 
            assert!(self.defunct == false, "tribe is defunct");
            assert!(self.enabled == false, "tribe is already active");

            let caller = self.env().caller();
            let value = self.env().transferred_value();
            
            ink_env::debug_println!(
                "received payment: {} from {:?}",
                value,
                caller
            );

            let mut founders = self.get_founder_list();
            let founder_index = self.get_founder_index(caller).expect("caller is not a founder");

            founders[founder_index].fund(value);

            self.founders.insert(0, &founders);
            
            self.activate_tribe();

            // calculate differences, send difference back to each founder
            // Dont implement this yet. 
        }

        #[ink(message)]
        pub fn get_founder_status(&self, founder: AccountId) -> String {
            let founders = self.get_founder_list();
            match self.get_founder_index(founder) {
                Some(v) => {
                    return founders[v].describe();
                },
                None => {
                    return ink_prelude::format!("Account is not a founder!");
                }
            };
        }

        #[ink(message)]
        pub fn get_tribe(&self) -> String {

            ink_prelude::format!(
                "Name: {}, Enabled: {}, Defunct: {}",
                &self.name,
                &self.enabled,
                &self.defunct
            )
        }

        #[ink(message)]
        pub fn reject_tribe(&mut self) {
            // if defunct or enabled, fail
            assert!(self.defunct == false, "tribe is defunct");
            assert!(self.enabled == false, "tribe is already active");

            let caller = self.env().caller();

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
    
/******************************** activate_tribe  ********************************/                
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

/******************************** get_founder_list  ********************************/                
        #[ink::test]
        fn get_founder_list_should_return_vec() {
            //ASSIGN
            let tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            let founder_list = tribe.get_founder_list();

            //ASSERT
            assert_eq!(founder_list.len(), 1);
        }

/******************************** get_founder_index  ********************************/        
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

/******************************** accept_tribe  ********************************/
        #[ink::test]
        #[should_panic(expected = "tribe is defunct")]   
        fn accept_tribe_should_fail_when_tribe_is_defunct(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.defunct = true;
            tribe.accept_tribe();
        }

        #[ink::test]
        #[should_panic(expected = "tribe is already active")]   
        fn accept_tribe_should_fail_when_tribe_is_enabled(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.enabled = true;
            tribe.accept_tribe();
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

        #[ink::test]
        fn accept_tribe_should_mark_founder_as_accepted() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            let founders = tribe.get_founder_list();

            //ACT
            tribe.accept_tribe();

            //ASSERT
            match tribe.get_founder_index(alice) {
                Some(index) => {
                    let founder = &founders[index];
                    assert!(founder.is_accepted()==false);
                },
                None => panic!("founder index not found"),
            } 
        }

/******************************** add_founder  ********************************/
        #[ink::test]
        #[should_panic(expected = "tribe is defunct")]   
        fn add_founder_should_fail_when_tribe_is_defunct(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            let bob = AccountId::from([0x1; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.defunct = true;
            tribe.add_founder(bob, 4000, false);
        }

        #[ink::test]
        #[should_panic(expected = "tribe is already active")]   
        fn add_founder_should_fail_when_tribe_is_enabled(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            let bob = AccountId::from([0x1; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.enabled = true;
            tribe.add_founder(bob, 4000, false);
        }

        #[ink::test]
        #[should_panic(expected = "caller is not a founder")]   
        fn add_founder_should_fail_when_caller_is_not_founder() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            let bob = AccountId::from([0x1; 32]); 
            let charlie = AccountId::from([0x2; 32]); 

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            tribe.add_founder(charlie, 4000, false);
        }

        #[ink::test]
        #[should_panic(expected = "caller is not the initial founder")]   
        fn add_founder_should_fail_when_caller_is_the_initial_founder() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            let bob = AccountId::from([0x1; 32]); 
            let charlie = AccountId::from([0x2; 32]); 

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            tribe.add_founder(bob, 4000, false);

            //ACT
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            tribe.add_founder(charlie, 4000, false);       
        }

        #[ink::test]
        #[should_panic(expected = "founder already exists")]   
        fn add_founder_should_fail_when_caller_is_initial_founder() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.add_founder(alice, 4000, false);       
        }

        #[ink::test]
        #[should_panic(expected = "founder already exists")]   
        fn add_founder_should_fail_when_caller_already_exists() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            let bob = AccountId::from([0x1; 32]); 

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            tribe.add_founder(bob, 4000, false);

            //ACT
            tribe.add_founder(bob, 4000, false);       
        }

        #[ink::test]
        #[should_panic(expected = "tribe is locked due to founder activity")]   
        fn add_founder_should_fail_when_any_founder_has_activity() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            let bob = AccountId::from([0x1; 32]); 
            let charlie = AccountId::from([0x2; 32]);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            tribe.add_founder(bob, 4000, false);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            tribe.reject_tribe();

            //ACT
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            tribe.add_founder(charlie, 4000, false);       
        }

        #[ink::test]
        fn add_founder_should_succeed() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            let bob = AccountId::from([0x1; 32]); 

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.add_founder(bob, 4000, false);

            //ASSERT
            let founders = tribe.get_founder_list();
            assert_eq!(founders.len(), 2);
        }

/******************************** fund_tribe  ********************************/
        #[ink::test]
        #[should_panic(expected = "tribe is defunct")]   
        fn fund_tribe_should_fail_when_tribe_is_defunct(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.defunct = true;
            tribe.fund_tribe();
        }

        #[ink::test]
        #[should_panic(expected = "tribe is already active")]   
        fn fund_tribe_should_fail_when_tribe_is_enabled(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.enabled = true;
            tribe.fund_tribe();
        }

        #[ink::test]
        #[should_panic(expected = "caller is not a founder")] 
        fn fund_tribe_should_fail_when_caller_is_not_founder(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            let bob = AccountId::from([0x1; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            tribe.fund_tribe();
        }

        #[ink::test]
        #[should_panic(expected = "founder must fund greater than zero amount")] 
        fn fund_tribe_without_value_should_fail(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.fund_tribe();
        }
        
        #[ink::test]
        fn fund_tribe_should_accept_funds(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            tribe.accept_tribe();

            //ACT
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(3000);
            tribe.fund_tribe();

            //ASSERT
            assert!(tribe.enabled == false);
        }
        
        #[ink::test]
        fn fund_tribe_should_accept_full_funding(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            tribe.accept_tribe();

            //ACT
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(3000);
            tribe.fund_tribe();

            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(2000);
            tribe.fund_tribe();

            //ASSERT
            assert!(tribe.enabled);
        }
    
/******************************** get_founder_status  ********************************/
        #[ink::test]
        fn get_founder_status_should_return_not_found_message() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);            
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let tribe = TribeContract::new(NAME.to_string(), 5000);
            
            //ACT
            let bob = AccountId::from([0x1; 32]);
            let status = tribe.get_founder_status(bob);
            
            ink_env::debug_println!("received status: {} ", status);

            //ASSERT
            assert_eq!(status, "Account is not a founder!");
        }

        #[ink::test]
        fn get_founder_status_should_return_founder_description() {
            
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);            
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let tribe = TribeContract::new(NAME.to_string(), 5000);
            
            //ACT
            let status = tribe.get_founder_status(alice);
            
            ink_env::debug_println!("received status: {} ", status);

            //ASSERT
            assert_eq!(status, r#"{
    "initial": true,
    "required": true,
    "rejected": false,
    "completed": false,
    "amount_promised": 5000,
    "amount_funded": 0
}"#);
        }
        
/******************************** get_tribe  ********************************/        
        macro_rules! get_tribe_should_return_expected {
            ($($name:ident: $value:expr,)*) => {
            $(
                #[ink::test]
                fn $name() {
                    //ASSIGN
                    let (name, enabled, defunct, expected) = $value;
                    let mut tribe = TribeContract::new(name.to_string(), 5000);
                    tribe.enabled = enabled;
                    tribe.defunct = defunct;
    
                    //ACT
                    let result = tribe.get_tribe();
    
                    //ASSERT                
                    assert_eq!(expected, result);
                }
            )*
            }
        }
        get_tribe_should_return_expected! {
            get_tribe_not_enabled_not_defunct: ("alice's massive tribe", false, false, "Name: alice's massive tribe, Enabled: false, Defunct: false"),
            get_tribe_enabled_not_defunct: ("yet another tribe", true, false, "Name: yet another tribe, Enabled: true, Defunct: false"),
            get_tribe_not_enabled_defunct: ("a defunct tribe", false, true, "Name: a defunct tribe, Enabled: false, Defunct: true"),
        }
    
/******************************** reject_tribe  ********************************/                

        #[ink::test]
        #[should_panic(expected = "tribe is defunct")]   
        fn reject_tribe_should_fail_when_tribe_is_defunct(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.defunct = true;
            tribe.reject_tribe();
        }

        #[ink::test]
        #[should_panic(expected = "tribe is already active")]   
        fn reject_tribe_should_fail_when_tribe_is_enabled(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.enabled = true;
            tribe.reject_tribe();
        }

        #[ink::test]
        #[should_panic(expected = "caller is not a founder")]   
        fn reject_tribe_should_fail_when_caller_is_not_founder() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            let bob = AccountId::from([0x1; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            tribe.reject_tribe();
        }

        #[ink::test]
        fn reject_tribe_should_succeed_and_mark_tribe_as_defunct() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            let prev_defunct = tribe.defunct;

            //ACT            
            tribe.reject_tribe();

            //ASSERT
            assert!(prev_defunct == false);
            
            assert!(tribe.defunct);
        }
    
        #[ink::test]
        fn reject_tribe_should_succeed_and_mark_founder_vote() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]); 
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT            
            tribe.reject_tribe();

            //ASSERT
            let founders = tribe.get_founder_list();
            match tribe.get_founder_index(alice) {
                Some(index) => {
                    let founder = &founders[index];
                    assert!(founder.is_rejected());
                },
                None => panic!("founder index not found"),
           } 
        }

    }
}
    