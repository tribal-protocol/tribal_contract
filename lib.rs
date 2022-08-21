#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;

mod errors;
mod founder;

#[ink::contract]
mod tribe {
    //use ink_env::{AccountId, return_value};
    use ink_storage::traits::{SpreadAllocate};
    use ink_prelude::{string::String, vec::Vec};
    use crate::errors::TribeError;
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
        /// Constructor that initializes the tribe with a given `init_name`, `initial_founder_amount_in_pico_needed` must not be 0
        #[ink(constructor, payable)]
        pub fn new(init_name: String, initial_founder_amount_in_pico_needed: u128) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                let caller = Self::env().caller();
                contract.name = init_name;
                contract.enabled = false;
                contract.defunct = false;

                contract.founders.insert(0, &ink_prelude::vec![
                    Founder::initial_founder(caller, initial_founder_amount_in_pico_needed).expect("need initial founder")
                ]);
            })
        }

        fn activate_tribe(&mut self) -> Result<(), TribeError> {
            if self.enabled && !self.defunct {
                return Ok(());
            }

            let all_founders = self.get_founder_list()?;
            for founder in all_founders {
                if founder.has_pending_activity() {
                    return Ok(());
                }
            }

            self.enabled = true;

            //TODO event goes here

            return Ok(());
        }

        fn get_founder_list(&self) -> Result<Vec<Founder>,TribeError> {
            return match self.founders.get(0) {
                Some(list) => Ok(list),
                None => Err(TribeError::FounderListNotFound)
            };
        }

        fn general_tribe_check(&self) -> Result<(), TribeError> {
            if self.defunct {
                return Err(TribeError::TribeIsDefunct);
            }
            if self.enabled {
                return Err(TribeError::ActiveTribeCannotAcceptAction);
            }
            Ok(())
        }

        fn get_founder_index(&self, founder_id: AccountId) -> Result<usize, TribeError> {
            let mut index: usize = 0;
            for founder in self.get_founder_list()? {
                if founder_id == founder.id {
                    return Ok(index);
                }
                index += 1;
            }
            Err(TribeError::NotAFounder)
        }

        /// Invoked by any founder to Accept Tribe, Before Funding
        #[ink(message)]
        pub fn accept_tribe(&mut self) -> Result<(), TribeError> {
            self.general_tribe_check()?;

            let caller = self.env().caller();
            let mut founders = self.get_founder_list()?;

            let founder_index: usize = match self.get_founder_index(caller) {
                Ok(index) => index,
                Err(err) => return Err(err)
            };

            let founder = &founders[founder_index];
            if founder.is_rejected() {
                return Err(TribeError::FounderRejectedInvitation)
            }

            // we got this far, set action to ACCEPTED
            founders[founder_index].vote_action = FOUNDER_ACCEPTED;
            self.founders.insert(0, &founders);

            Ok(())
        }

        /// Invoked by the initial founder to invite an AccountId as a founder. This is done Before the Tribe is marked either as `enabled` or `defunct`
        #[ink(message)]
        pub fn invite_founder(&mut self, potential_founder: AccountId, amount_in_pico: u128, required: bool) -> Result<(), TribeError> {
            self.general_tribe_check()?;

            let caller = Self::env().caller();
            if caller == potential_founder {
                return Err(TribeError::CanNotInviteInitialFounder);
            }

            let mut founders = self.get_founder_list()?;

            let founder_index = self.get_founder_index(caller)?;
            let initial_founder = &founders[founder_index];

            // is the caller the initial_founder?
            if initial_founder.initial == false {
                return Err(TribeError::NotInitialFounder);
            }

            // iterate all AccountIds
            for founder in &founders {
                // is founder already in the founder list?
                if founder.id == potential_founder {
                    return Err(TribeError::FounderAlreadyInvited);
                }

                // has any founder rejected? any amount funded?
                if !founder.has_pending_activity() || founder.has_funds() {
                    return Err(TribeError::TribeIsLocked);
                }
            }

            // we got this far, add the founder.
            let new_founder = Founder::new(potential_founder, required, amount_in_pico)?;
            founders.push(new_founder);
            self.founders.insert(0, &founders);

            Ok(())
        }

        /// Invoked by any founder After accept_tribe success
        #[ink(message, payable, selector = 0xC4577B10)]
        pub fn fund_tribe(&mut self) -> Result<u128, TribeError> {
            self.general_tribe_check()?;

            let caller = self.env().caller();
            let value = self.env().transferred_value();
            
            ink_env::debug_println!(
                "received payment: {} from {:?}",
                value,
                caller
            );

            let mut founders = self.get_founder_list()?;
            let founder_index = self.get_founder_index(caller)?;
            let total_funded_amount = founders[founder_index].fund(value)?;

            self.founders.insert(0, &founders);
            
            self.activate_tribe()?;

            // calculate differences, send difference back to each founder
            // Dont implement this yet.

            Ok(total_funded_amount)
        }

        /// Returns a string representation of the current founder status for the given AccountId
        #[ink(message)]
        pub fn get_founder_status(&self, founder: AccountId) -> String {
            match self.get_founder_index(founder) {
                Ok(v) => match self.get_founder_list() {
                    Ok(founders) =>
                        founders[v].describe(),
                    Err(err) => {
                        ink_prelude::format!("Problem with founder list; {}", err)
                    }
                },
                Err(err) => {
                    ink_prelude::format!("{}", err)
                }
            }
        }

        /// Returns a string representation of the current tribe
        #[ink(message)]
        pub fn get_tribe(&self) -> String {

            ink_prelude::format!(r#"{{
    "name": {},
    "enabled": {},
    "defunct": {}
}}"#, 
                &self.name,
                &self.enabled,
                &self.defunct
            )
        }

        /// Invoked by any founder to Reject the Tribe. Can be done After Accept, and can not be undone.
        #[ink(message)]
        pub fn reject_tribe(&mut self) -> Result<(), TribeError> {
            self.general_tribe_check()?;

            let caller = self.env().caller();

            // if founder does NOT exist in founders_required, fail
            let mut founders = self.get_founder_list()?;
            let founder_index = self.get_founder_index(caller)?;
            founders[founder_index].vote_action = FOUNDER_REJECTED;

            if founders[founder_index].required {
                self.defunct = true;

                // TODO REFUND EVERYONE WHO SENT UNITS
            } else {
                self.activate_tribe()?;
            }

            self.founders.insert(0, &founders);
            // if any founder has already funded tribe, return funds to each founder
            // TODO

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        const NAME: &str = "a test tribe";

//******************************** create_tribe  ********************************
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

//******************************** activate_tribe  ********************************
        #[ink::test]
        fn activate_tribe_with_no_activity_should_have_no_effect() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.activate_tribe().expect("should pass");

            //ASSERT
            assert_eq!(tribe.enabled, false)
        }

        #[ink::test]
        fn active_tribe_with_funded_activity_should_enable() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            /* Update Alice in founders list to accept and fully fund tribe */
            let alice_index = tribe.get_founder_index(alice).expect("alice should be initial founder");
            let mut founders = tribe.get_founder_list().expect("should get list");
            founders[alice_index].vote_action = FOUNDER_ACCEPTED;
            founders[alice_index].fund(5000).expect("ok");
            tribe.founders.insert(0, &founders);

            //ACT
            let prev_enabled = tribe.enabled;
            tribe.activate_tribe().expect("should pass");

            //ASSERT
            assert!(!prev_enabled);
            assert!(tribe.enabled);
        }

//******************************** get_founder_list  ********************************
        #[ink::test]
        fn get_founder_list_should_return_vec() {
            //ASSIGN
            let tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            let founder_list = tribe.get_founder_list().expect("should pass");

            //ASSERT
            assert_eq!(founder_list.len(), 1);
        }

//******************************** get_founder_index  ********************************
        #[ink::test]
        fn get_founder_index_should_return_initial_founder_index() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);

            //ACT
            let tribe = TribeContract::new(NAME.to_string(), 5000);
            match tribe.get_founder_index(alice) {
                Ok(index) => {
                    let founders = tribe.get_founder_list().expect("should get founder list");
                    let alice_founder = &founders[index];

                    //ASSERT
                    assert_eq!(alice_founder.id, alice);
                },
                Err(err) => panic!("founder index not found; {}", err),
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
                Ok(_) => assert!(false),
                //ASSERT
                Err(err) => assert_eq!(TribeError::NotAFounder, err),
            }
        }

//******************************** accept_tribe  ********************************
        #[ink::test]
        fn accept_tribe_should_fail_when_tribe_is_defunct(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.defunct = true;
            match tribe.accept_tribe() {
                Ok(_) => assert!(false, "Should not accept tribe"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::TribeIsDefunct, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn accept_tribe_should_fail_when_tribe_is_enabled(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.enabled = true;
            match tribe.accept_tribe() {
                Ok(_) => assert!(false, "Should not accept tribe"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::ActiveTribeCannotAcceptAction, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn accept_tribe_should_fail_when_caller_is_not_founder() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            let bob = AccountId::from([0x1; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            match tribe.accept_tribe() {
                Ok(_) => assert!(false, "Should not accept tribe"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::NotAFounder, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn accept_tribe_should_fail_when_caller_already_rejected_tribe() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            match tribe.get_founder_index(alice) {
                Ok(index) => {
                    // Mark founder as rejected
                    let mut founders = tribe.get_founder_list().expect("should pass");
                    founders[index].vote_action = FOUNDER_REJECTED;
                    tribe.founders.insert(0, &founders);

                    match tribe.accept_tribe() {
                        Ok(_) => assert!(false, "Should not accept tribe"),
                        //ASSERT
                        Err(err) => assert_eq!(TribeError::FounderRejectedInvitation, err, "actual error received {}", err)
                    }

                },
                Err(err) => panic!("founder index not found; Error={}", err)
            }
        }

        #[ink::test]
        fn accept_tribe_should_mark_founder_as_accepted() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            let founders = tribe.get_founder_list().expect("should get founder list");

            //ACT
            tribe.accept_tribe().expect("Should have passed");

            //ASSERT
            match tribe.get_founder_index(alice) {
                Ok(index) => {
                    let founder = &founders[index];
                    assert_eq!(founder.is_accepted(), false);
                },
                Err(err) => panic!("founder index not found; Error={}", err)
            }
        }

//******************************** invite_founder  ********************************
        #[ink::test]
        fn invite_founder_should_fail_when_tribe_is_defunct(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            let bob = AccountId::from([0x1; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.defunct = true;
            match tribe.invite_founder(bob, 4000, false) {
                Ok(_) => assert!(false, "Invite founder should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::TribeIsDefunct, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn invite_founder_should_fail_when_tribe_is_enabled(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            let bob = AccountId::from([0x1; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.enabled = true;
            match tribe.invite_founder(bob, 4000, false) {
                Ok(_) => assert!(false, "Invite founder should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::ActiveTribeCannotAcceptAction, err, "actual error received {}", err)
            }

        }

        #[ink::test]
        fn invite_founder_should_fail_when_caller_is_not_founder() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            let bob = AccountId::from([0x1; 32]);
            let charlie = AccountId::from([0x2; 32]);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            match tribe.invite_founder(charlie, 4000, false) {
                Ok(_) => assert!(false, "Invite founder should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::NotAFounder, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn invite_founder_should_fail_when_caller_is_not_the_initial_founder() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            let bob = AccountId::from([0x1; 32]);
            let charlie = AccountId::from([0x2; 32]);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            tribe.invite_founder(bob, 4000, false).expect("should pass");

            //ACT
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            match tribe.invite_founder(charlie, 4000, false) {
                Ok(_) => assert!(false, "Invite founder should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::NotInitialFounder, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn invite_founder_should_fail_to_invite_the_initial_founder() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            match tribe.invite_founder(alice, 4000, false) {
                Ok(_) => assert!(false, "Invite founder should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::CanNotInviteInitialFounder, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn invite_founder_should_fail_to_invite_same_account_more_than_once() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            let bob = AccountId::from([0x1; 32]);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            tribe.invite_founder(bob, 4000, false).expect("should pass");

            //ACT
            match tribe.invite_founder(bob, 4000, false) {
                Ok(_) => assert!(false, "Invite founder should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::FounderAlreadyInvited, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn invite_founder_should_fail_when_any_founder_has_activity() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            let bob = AccountId::from([0x1; 32]);
            let charlie = AccountId::from([0x2; 32]);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            tribe.invite_founder(bob, 4000, false).expect("should pass");

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            tribe.reject_tribe().expect("bob should be able to reject tribe");

            //ACT
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            match tribe.invite_founder(charlie, 4000, false) {
                Ok(_) => assert!(false, "Invite founder should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::TribeIsLocked, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn invite_founder_should_succeed() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            let bob = AccountId::from([0x1; 32]);

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.invite_founder(bob, 4000, false).expect("should pass");

            //ASSERT
            let founders = tribe.get_founder_list().expect("should get founder list");
            assert_eq!(founders.len(), 2);
        }

//******************************** fund_tribe  ********************************
        #[ink::test]
        fn fund_tribe_should_fail_when_tribe_is_defunct() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.defunct = true;
            match tribe.fund_tribe() {
                Ok(_) => assert!(false, "fund tribe should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::TribeIsDefunct, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn fund_tribe_should_fail_when_tribe_is_enabled(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.enabled = true;
            match tribe.fund_tribe() {
                Ok(_) => assert!(false, "fund tribe should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::ActiveTribeCannotAcceptAction, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn fund_tribe_should_fail_when_caller_is_not_founder(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            let bob = AccountId::from([0x1; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            match tribe.fund_tribe() {
                Ok(_) => assert!(false, "fund tribe should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::NotAFounder, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn fund_tribe_without_value_should_fail(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            match tribe.fund_tribe() {
                Ok(_) => assert!(false, "fund tribe should not pass"),
                //ASSERT
                Err(err) => assert_eq!(err, TribeError::FundingAmountMustBeGreaterThanZero)
            }
        }

        #[ink::test]
        fn fund_tribe_should_accept_funds(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            tribe.accept_tribe().expect("should pass");

            //ACT
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(3000);
            let funding = tribe.fund_tribe().expect("should pass");

            //ASSERT
            assert_eq!(tribe.enabled, false);
            assert_eq!(funding, 3000);
        }

        #[ink::test]
        fn fund_tribe_should_accept_full_funding(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            tribe.accept_tribe().expect("should pass");

            //ACT
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(3000);
            let funding1 = tribe.fund_tribe().expect("should pass");

            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(2000);
            let funding2 = tribe.fund_tribe().expect("should pass");

            //ASSERT
            assert!(tribe.enabled);
            assert_eq!(funding1, 3000);
            assert_eq!(funding2, 5000);
        }

        //******************************** get_founder_status  ********************************
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
            assert_eq!(status, "AccountId is not a Founder");
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

//******************************** get_tribe  ********************************
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
            get_tribe_not_enabled_not_defunct: ("alice's massive tribe", false, false, "{\n    \"name\": alice's massive tribe,\n    \"enabled\": false,\n    \"defunct\": false\n}"),
            get_tribe_enabled_not_defunct: ("yet another tribe", true, false, "{\n    \"name\": yet another tribe,\n    \"enabled\": true,\n    \"defunct\": false\n}"),
            get_tribe_not_enabled_defunct: ("a defunct tribe", false, true, "{\n    \"name\": a defunct tribe,\n    \"enabled\": false,\n    \"defunct\": true\n}"),
        }

//******************************** reject_tribe  ********************************

        #[ink::test]
        fn reject_tribe_should_fail_when_tribe_is_defunct(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.defunct = true;
            match tribe.reject_tribe() {
                Ok(_) => assert!(false, "reject tribe should not pass"),
                //ASSERT
                Err(err) => assert_eq!(err, TribeError::TribeIsDefunct)
            }
        }

        #[ink::test]
        fn reject_tribe_should_fail_when_tribe_is_enabled(){
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.enabled = true;
            match tribe.reject_tribe() {
                Ok(_) => assert!(false, "reject tribe should not pass"),
                //ASSERT
                Err(err) => assert_eq!(err, TribeError::ActiveTribeCannotAcceptAction)
            }
        }

        #[ink::test]
        fn reject_tribe_should_fail_when_caller_is_not_founder() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            let bob = AccountId::from([0x1; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(bob);
            match tribe.reject_tribe() {
                Ok(_) => assert!(false, "reject tribe should not pass"),
                //ASSERT
                Err(err) => assert_eq!(TribeError::NotAFounder, err, "actual error received {}", err)
            }
        }

        #[ink::test]
        fn reject_tribe_should_succeed_and_mark_tribe_as_defunct() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);
            let prev_defunct = tribe.defunct;

            //ACT
            tribe.reject_tribe().expect("should pass");

            //ASSERT
            assert_eq!(prev_defunct, false);

            assert!(tribe.defunct);
        }

        #[ink::test]
        fn reject_tribe_should_succeed_and_mark_founder_vote() {
            //ASSIGN
            let alice = AccountId::from([0x0; 32]);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            let mut tribe = TribeContract::new(NAME.to_string(), 5000);

            //ACT
            tribe.reject_tribe().expect("should pass");

            //ASSERT
            let founders = tribe.get_founder_list().expect("should get list");
            match tribe.get_founder_index(alice) {
                Ok(index) => {
                    let founder = &founders[index];
                    assert!(founder.is_rejected());
                },
                Err(err) => panic!("founder index not found; Error={}", err),
            }
        }

    }
}
    