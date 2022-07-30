
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
    amount_funded: u128,
}

impl Founder {
    
    pub fn new (id: AccountId, required: bool, amount_promised: u128) -> Result<Self, String> {
        if amount_promised > 0 {  
            return Ok(Self {
                    id: id,
                    initial: false,
                    required: required,
                    vote_action: FOUNDER_PENDING,
                    amount_promised: amount_promised,
                    amount_funded: 0
                });
        }
        else {
            Err( String::from("amount promised in picos must be greater than 0"))
        }
    }

    pub fn initial_founder(id: AccountId, amount_promised: u128) -> Result<Self, String> {
        match Founder::new(id, true, amount_promised) {
            Ok(mut f) => {
                f.initial = true;
                Ok(f)
            },
            Err(err) => Err(err.into())
        }
    }

    pub fn fund(&mut self, amount: u128) -> u128 {
        if amount == 0 {
            panic!("founder must fund greater than zero amount")
        }
        if !self.is_accepted() {
            panic!("founder has not accepted tribe");
        }
        else if self.is_funded() {
            panic!("founder has completed funding");
        }
        else {
            self.amount_funded += amount;
        }
        self.amount_funded
    }

    pub fn has_funds(&self) -> bool {
        self.amount_funded > 0
    }

    pub fn has_pending_activity(&self) -> bool {

        if self.vote_action == FOUNDER_PENDING  {
            if self.required {
                return true;
            }
            else {
                return false;
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
       return self.vote_action == FOUNDER_ACCEPTED;
    }

    pub fn is_funded(&self) -> bool {
        self.amount_funded >= self.amount_promised
    }

    pub fn is_rejected(&self) -> bool {
        self.vote_action == FOUNDER_REJECTED
    }

    pub fn describe(&self) -> String {
        ink_prelude::format!(
            "Is Initial: {} Required: {} Is Rejected: {} Is Completed: {} Amount Promised: {} Total Amount Funded: {}",
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

    // let bob = AccountId::try_from([0x1; 32]).unwrap();
    // let charlie = AccountId::try_from([0x2; 32]).unwrap();  

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
        new_0: (AccountId::try_from([0x0; 32]).unwrap(), false, 1234),
        new_1: (AccountId::try_from([0x0; 32]).unwrap(), true, 1234),
        new_2: (AccountId::try_from([0x1; 32]).unwrap(), false, 8899),
        new_3: (AccountId::try_from([0x1; 32]).unwrap(), true, 8899),
    }

    #[ink::test]
    #[should_panic(expected = "amount promised in picos must be greater than 0")]
    fn new_fails_when_amount_promised_is_zero()  {
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        
        //ACT        
        match Founder::new(alice, false, 0) {
            Ok(_) =>  {},
            Err(e) => panic!("{}", e)
        };
    }

    #[ink::test]
    fn initial_founder_can_create() {
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        
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
    #[should_panic(expected = "amount promised in picos must be greater than 0")]
    fn initial_founder_fails_when_amount_promised_is_zero() {
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        
        //ACT
        match Founder::initial_founder(alice, 0) {
            Ok(_) =>  {},
            Err(e) => panic!("{}", e)
        };
    }

    #[ink::test]
    #[should_panic(expected = "founder has not accepted tribe")]
    fn fund_should_fail_when_tribe_is_not_accepted() {
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");

        //ACT
        founder.fund(5000);
    }

    #[ink::test]
    #[should_panic(expected = "founder must fund greater than zero amount")]
    fn fund_should_fail_with_zero_fund_amount() {
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");

        //ACT
        founder.fund(0);
    }

    #[ink::test]
    fn fund_should_pass_when_tribe_is_accepted() {
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_ACCEPTED;

        //ACT
        founder.fund(5000);
    }

    #[ink::test]
    fn fund_should_allow_mutliple_fundings_until_promise_amount() {
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_ACCEPTED;

        //ACT
        let round1= founder.fund(2000);
        let round2= founder.fund(2000);
        let round3= founder.fund(2000);

        //ASSERT
        assert_eq!(round1, 2000);
        assert_eq!(round2, 4000);
        assert_eq!(round3, 6000);
        assert!(founder.is_funded());
    }
    
    #[ink::test]
    #[should_panic(expected = "founder has completed funding")]
    fn fund_should_fail_when_founder_already_funded() {
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_ACCEPTED;

        //ACT
        assert!(founder.is_funded() == false);
        founder.fund(5000);
        assert!(founder.is_funded());
        founder.fund(5000);
    }

    #[ink::test]
    fn has_funds_should_return_true(){
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_ACCEPTED;
        
        //ACT
        founder.fund(100);

        //ASSERT
        assert!(founder.has_funds())
    }

    #[ink::test]
    fn has_funds_should_return_false(){
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let founder = Founder::new(alice, true, 5000).expect("expected founder");
        
        //ACT
        assert!(founder.has_funds() == false)
    }

    //required, vote_acount, promised, funded, expected
    macro_rules! founder_has_pending_activity {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[ink::test]
            fn $name() {
                //ASSIGN
                let alice = AccountId::try_from([0x0; 32]).unwrap();                
                let (required, vote_action, promised, funded, expected) = $value;
                let mut founder = Founder::new(alice, required, promised).expect("expected founder");
                if (funded > 0) {
                    founder.vote_action = FOUNDER_ACCEPTED;
                    founder.fund(funded);
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
                let alice = AccountId::try_from([0x0; 32]).unwrap();                
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
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let mut founder = Founder::new(alice, true, 5000).expect("expected founder");
        founder.vote_action = FOUNDER_ACCEPTED;

        //ACT
        assert!(founder.is_funded() == false);

        founder.fund(1000);
        assert!(founder.is_funded() == false);

        founder.fund(1000);
        assert!(founder.is_funded() == false);

        founder.fund(1000);
        assert!(founder.is_funded() == false);

        founder.fund(1000);
        assert!(founder.is_funded() == false);

        founder.fund(1000);
        assert!(founder.is_funded() == true);
    }

    macro_rules! founder_is_rejected {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[ink::test]
            fn $name() {
                //ASSIGN
                let alice = AccountId::try_from([0x0; 32]).unwrap();                
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
