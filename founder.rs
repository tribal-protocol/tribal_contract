
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
    pub fn new (id: AccountId, required: bool, amount_promised: u128) -> Self {
        Founder {
             id: id,
             initial: false,
             required: required,
             vote_action: FOUNDER_PENDING,
             amount_promised: amount_promised,
             amount_funded: 0
         }
    }

    pub fn initial_founder(id: AccountId, amount_promised: u128) -> Self {
        let mut founder = Founder::new(id, true, amount_promised);
        founder.initial = true;
        return founder;
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
    use super::*;
    use ink_lang as ink;

    //use crate::inkrement::{FOUNDER_ACCEPTED, FOUNDER_REJECTED, FOUNDER_PENDING};
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
                let founder = Founder::new(id, required, picos);

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
        founder_new_0: (AccountId::try_from([0x0; 32]).unwrap(), false, 1234),
        founder_new_1: (AccountId::try_from([0x0; 32]).unwrap(), true, 1234),

        founder_new_2: (AccountId::try_from([0x1; 32]).unwrap(), false, 8899),
        founder_new_3: (AccountId::try_from([0x1; 32]).unwrap(), true, 8899),
    }

/*
    #[ink::test]
    fn founder_can_create () {
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        
        //ACT
        let founder = Founder::new(alice, true, 1234);

        //ASSERT
        assert_eq!(founder.id, alice);
        assert!(founder.initial == false);
        assert!(founder.required);
        assert_eq!(founder.amount_promised, 1234);
        assert_eq!(founder.amount_funded, 0);
    }
*/

    #[ink::test]
    fn initial_founder_can_create() {
        //ASSIGN
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        
        //ACT
        let founder = Founder::initial_founder(alice, 1234);

        //ASSERT
        assert_eq!(founder.id, alice);
        assert!(founder.initial);
        assert!(founder.required);
        assert_eq!(founder.amount_promised, 1234);
        assert_eq!(founder.amount_funded, 0);

    }
}
