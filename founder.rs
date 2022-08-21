use ink_env::AccountId;
use ink_storage::traits::{SpreadLayout, PackedLayout};
use ink_prelude::{string::String};
use crate::
{
    errors::TribeError,
    tribe::{FOUNDER_ACCEPTED, FOUNDER_REJECTED, FOUNDER_PENDING}
};

#[derive(PackedLayout, SpreadLayout, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub struct Founder {
    pub id: AccountId,
    pub initial: bool,
    pub required: bool,
    pub vote_action: i32,
    pub amount_promised: u128,
    amount_funded: u128,
}

impl Founder {
    
    pub fn new (id: AccountId, required: bool, amount_promised: u128) -> Result<Self, TribeError> {
        return if amount_promised > 0 {
            Ok(Self {
                id,
                initial: false,
                required,
                vote_action: FOUNDER_PENDING,
                amount_promised,
                amount_funded: 0
            })
        } else {
            Err(TribeError::AmountPromisedIsZero)
        }
    }

    pub fn initial_founder(id: AccountId, amount_promised: u128) -> Result<Self, TribeError> {
        let mut founder = Founder::new(id, true, amount_promised)?;
        founder.initial = true;
        Ok(founder)
    }

    pub fn fund(&mut self, amount: u128) -> Result<u128, TribeError> {
        if amount == 0 {
            return Err(TribeError::FundingAmountMustBeGreaterThanZero);
        }
        if self.is_rejected() {
            return Err(TribeError::FounderRejectedInvitation);
        }
        if !self.is_accepted() {
            return Err(TribeError::FounderVoteActionPending)
        }
        if self.is_funded() {
            return Err(TribeError::FundingAlreadyCompleted);
        }
        else {
            self.amount_funded += amount;
        }
        Ok(self.amount_funded)
    }

    pub fn has_funds(&self) -> bool {
        self.amount_funded > 0
    }

    pub fn has_pending_activity(&self) -> bool {

        if self.vote_action == FOUNDER_PENDING  {
            return if self.required {
                true
            } else {
                false
            }
        } 
        else if self.is_rejected() {
            return false;
        } 
        else if self.is_funded() {
            return false;
        }
        return true;
    }

    pub fn is_accepted(&self) -> bool {
       self.vote_action == FOUNDER_ACCEPTED
    }

    pub fn is_funded(&self) -> bool {
        self.amount_funded >= self.amount_promised
    }

    pub fn is_rejected(&self) -> bool {
        self.vote_action == FOUNDER_REJECTED
    }

    pub fn describe(&self) -> String { 

        ink_prelude::format!(r#"{{
    "initial": {},
    "required": {},
    "rejected": {},
    "completed": {},
    "amount_promised": {},
    "amount_funded": {}
}}"#, 
          self.initial,
          self.required,
          self.is_rejected(),
          !self.has_pending_activity(),
          self.amount_promised,
          self.amount_funded
      )
    }
}

/// 
/// Founder Unit Tests
/// 
#[cfg(test)]
mod founder_tests {
    use super::*;
    use ink_lang as ink;

    macro_rules! founder_new_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[ink::test]
            fn $name() {
                //ASSIGN
                let (id, required, picos) = $value;

                //ACT
                let founder = Founder::new(id, required, picos).expect("expected founder");

                //ASSERT
                assert_eq!(founder.id, id);
                assert!(founder.initial == false);
                assert_eq!(founder.required, required);
                assert_eq!(founder.amount_promised, picos);
                assert_eq!(founder.amount_funded, 0);
            }
        )*
        }
    }
    founder_new_tests! {
        new_0: (AccountId::from([0x0; 32]), false, 1234),
        new_1: (AccountId::from([0x0; 32]), true, 1234),
        new_2: (AccountId::from([0x0; 32]), false, 8899),
        new_3: (AccountId::from([0x0; 32]), true, 8899),
    }

    #[ink::test]
    fn new_fails_when_amount_promised_is_zero()  {
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        
        //ACT        
        match Founder::new(alice, false, 0) {
            Ok(_) => assert!(false, "Should NOT have passed"),
            Err(e) => assert_eq!(e, TribeError::AmountPromisedIsZero) 
        };
    }

    #[ink::test]
    fn initial_founder_can_create() {
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        
        //ACT
        let founder = Founder::initial_founder(alice, 1234).expect("expected founder");

        //ASSERT
        assert_eq!(founder.id, alice);
        assert!(founder.initial);
        assert!(founder.required);
        assert_eq!(founder.amount_promised, 1234);
        assert_eq!(founder.amount_funded, 0);
    }

    #[ink::test]    
    fn initial_founder_fails_when_amount_promised_is_zero() {
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        
        //ACT
        match Founder::initial_founder(alice, 0) {            
            Ok(_) => assert!(false, "Should NOT have passed."),
            Err(e) => assert_eq!(e, TribeError::AmountPromisedIsZero) 
        };
    }

//***************************** fund() ***************************
    #[ink::test]
    fn fund_should_fail_when_tribe_is_not_accepted() {
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");

        //ACT
        match founder.fund(5000) {
            Ok(_) => assert!(false, "Should not have passed"),
            Err(err) => assert_eq!(err, TribeError::FounderVoteActionPending)
        }
    }

    #[ink::test]
    fn fund_should_fail_when_founder_rejected_tribe() {
        //ASSIGN
        let alice = AccountId::from([0x0; 32]);
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_REJECTED;

        //ACT
        match founder.fund(2000) {
            Ok(_) => assert!(false, "Should not have passed"),
            Err(err) => assert_eq!(err, TribeError::FounderRejectedInvitation)
        }
    }

    #[ink::test]
    fn fund_should_fail_with_zero_fund_amount() {
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");

        //ACT
        match founder.fund(0) {
            Ok(_) => assert!(false, "Should not have passed"),
            Err(err) => assert_eq!(err, TribeError::FundingAmountMustBeGreaterThanZero)
        }
    }

    #[ink::test]
    fn fund_should_pass_when_tribe_is_accepted() {
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_ACCEPTED;

        //ACT
        let amount = founder.fund(5000).expect("funding ok");

        //ASSERT
        assert_eq!(amount, 5000);
    }

    #[ink::test]
    fn fund_should_allow_multiple_funding_events_until_promise_amount() {
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_ACCEPTED;

        //ACT
        let round1= founder.fund(2000).expect("funding ok");
        let round2= founder.fund(2000).expect("funding ok");
        let round3= founder.fund(2000).expect("funding ok");

        //ASSERT
        assert_eq!(round1, 2000);
        assert_eq!(round2, 4000);
        assert_eq!(round3, 6000);
        assert!(founder.is_funded());
    }
    
    #[ink::test]
    fn fund_should_fail_when_founder_already_funded() {
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_ACCEPTED;

        //ACT
        assert_eq!(founder.is_funded(), false);
        founder.fund(5000).expect("funding ok");
        assert!(founder.is_funded());
        match founder.fund(5000) {
            Ok(_) => assert!(false, "Should not have passed"),
            Err(err) => assert_eq!(err, TribeError::FundingAlreadyCompleted)
        }
    }

    #[ink::test]
    fn has_funds_should_return_true(){
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_ACCEPTED;
        
        //ACT
        founder.fund(100).expect("funding should be ok");

        //ASSERT
        assert!(founder.has_funds())
    }

    #[ink::test]
    fn has_funds_should_return_false(){
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        let founder = Founder::new(alice, true, 5000).expect("expected founder");
        
        //ACT
        assert_eq!(founder.has_funds(), false)
    }

    //required, vote_account, promised, funded, expected
    macro_rules! founder_has_pending_activity {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[ink::test]
            fn $name() {
                //ASSIGN
                let alice = AccountId::from([0x0; 32]); 
                let (required, vote_action, promised, funded, expected) = $value;
                let mut founder = Founder::new(alice, required, promised).expect("expected founder");
                if (funded > 0) {
                    founder.vote_action = FOUNDER_ACCEPTED;
                    founder.fund(funded).expect("ok");
                }
                founder.vote_action = vote_action;

                //ACT
                let result = founder.has_pending_activity();

                //ASSERT                
                assert_eq!(expected, result);
            }
        )*
        }
    }
    founder_has_pending_activity! {
        required_pending_5000_0: (true, FOUNDER_PENDING, 5000, 0, true),
        required_accepted_5000_0: (true, FOUNDER_ACCEPTED, 5000, 0, true),
        required_accepted_5000_5000: (true, FOUNDER_ACCEPTED, 5000, 5000, false),
        required_rejected_5000_0: (true, FOUNDER_REJECTED, 5000, 0, false),
        required_rejected_5000_5000: (true, FOUNDER_REJECTED, 5000, 5000, false),

        optional_pending_5000_0: (false, FOUNDER_PENDING, 5000, 0, false),
        optional_accepted_5000_0: (false, FOUNDER_ACCEPTED, 5000, 0, true),
        optional_accepted_5000_5000: (false, FOUNDER_ACCEPTED, 5000, 5000, false),
        optional_rejected_5000_0: (false, FOUNDER_REJECTED, 5000, 0, false),
        optional_rejected_5000_5000: (false, FOUNDER_REJECTED, 5000, 5000, false),
    }
    
    macro_rules! founder_is_accepted {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[ink::test]
            fn $name() {
                //ASSIGN
                let alice = AccountId::from([0x0; 32]); 
                let (required, vote_action, expected) = $value;
                let mut founder = Founder::new(alice, required, 5555).expect("expected founder");

                //ACT
                founder.vote_action = vote_action;
                let result = founder.is_accepted();

                //ASSERT                
                assert!(founder.initial == false);
                assert_eq!(founder.required, required);
                assert_eq!(expected, result);
            }
        )*
        }
    }
    founder_is_accepted! {
        is_accpted_true_with_required_founder_accepted: (true, FOUNDER_ACCEPTED, true),
        is_accpted_false_with_required_founder_pending: (true, FOUNDER_PENDING, false),
        is_accpted_false_with_required_founder_rejected: (true, FOUNDER_REJECTED, false),

        is_accpted_true_with_founder_accepted: (false, FOUNDER_ACCEPTED, true),
        is_accpted_false_with_founder_pending: (false, FOUNDER_PENDING, false),
        is_accpted_false_with_founder_rejected: (false, FOUNDER_REJECTED, false),
    }      

    #[ink::test]
    fn is_funded_should_return_expected() {
        //ASSIGN
        let alice = AccountId::from([0x0; 32]); 
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_ACCEPTED;

        //ACT
        assert_eq!(founder.is_funded(), false);

        assert_eq!(founder.fund(1000).expect("funding should pass"), 1000);
        assert_eq!(founder.is_funded(), false);

        assert_eq!(founder.fund(1000).expect("funding should pass"), 2000);
        assert_eq!(founder.is_funded(), false);

        assert_eq!(founder.fund(1000).expect("funding should pass"), 3000);
        assert_eq!(founder.is_funded(), false);

        assert_eq!(founder.fund(1000).expect("funding should pass"), 4000);
        assert_eq!(founder.is_funded(), false);

        assert_eq!(founder.fund(1000).expect("funding should pass"), 5000);
        assert!(founder.is_funded());
    }

    macro_rules! founder_is_rejected {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[ink::test]
            fn $name() {
                //ASSIGN
                let alice = AccountId::from([0x0; 32]); 
                let (vote_action, expected) = $value;
                let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
                founder.vote_action = vote_action;

                //ACT
                let result = founder.is_rejected();

                //ASSERT                
                assert_eq!(expected, result);
            }
        )*
        }
    }
    founder_is_rejected! {
        founder_is_rejected_pending: (FOUNDER_PENDING, false),
        founder_is_rejected_accepted: (FOUNDER_ACCEPTED, false),
        founder_is_rejected_rejected: (FOUNDER_REJECTED, true),    
    }
}
