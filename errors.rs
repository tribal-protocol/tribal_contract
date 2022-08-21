use std::fmt;
use std::fmt::{Formatter};

#[derive(PartialEq, Debug, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum TribeError {
    ActiveTribeCannotAcceptAction,
    AmountPromisedIsZero,
    CanNotInviteInitialFounder,
    FounderAlreadyInvited,
    FounderListNotFound,
    FounderRejectedInvitation,
    FounderVoteActionPending,
    FundingAlreadyCompleted,
    FundingAmountMustBeGreaterThanZero,
    NotAFounder,
    NotInitialFounder,
    TribeIsDefunct,
    TribeIsLocked
}

impl std::error::Error for TribeError { }

impl fmt::Display for TribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TribeError::ActiveTribeCannotAcceptAction => write!(f, "Active tribe cannot accept activity"),
            TribeError::AmountPromisedIsZero => write!(f, "Amount promised in pico must be greater than 0"),
            TribeError::CanNotInviteInitialFounder => write!(f, "The initial founder can not be invited to join their own tribe"),
            TribeError::FounderAlreadyInvited => write!(f, "AccountId already exists as a Founder"),
            TribeError::FounderListNotFound => write!(f, "Tribe list of founders  not found"),
            TribeError::FounderRejectedInvitation => write!(f, "Founder already rejected invitation to tribe"),
            TribeError::FounderVoteActionPending => write!(f, "Founder has not taken an action on pending invitation"),
            TribeError::FundingAlreadyCompleted => write!(f, "Founder has already completed funding"),
            TribeError::FundingAmountMustBeGreaterThanZero => write!(f, "Funding amount must be greater than zero amount"),
            TribeError::NotAFounder => write!(f, "AccountId is not a Founder"),
            TribeError::NotInitialFounder => write!(f, "AccountId is not the Initial Founder"),
            TribeError::TribeIsDefunct => write!(f, "Tribe is defunct and cannot accept any more activity"),
            TribeError::TribeIsLocked => write!(f, "Tribe is locked due to founder activity")
        }
    }
}

#[cfg(test)]
mod founder_tests {
    use super::*;
    use ink_lang as ink;

    #[ink::test]
    fn check_error_generates_correct_string()  {
        //ASSIGN
        let error = TribeError::ActiveTribeCannotAcceptAction;

        //ACT
        let description = ink_prelude::format!("{}", error);

        //ASSERT
        assert_eq!(description, "Active tribe cannot accept activity");
    }

    macro_rules! error_description_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[ink::test]
            fn $name() {
                //ASSIGN
                let (error, expected) = $value;

                //ACT
                let description = ink_prelude::format!("{}", error);

                //ASSERT
                assert_eq!(description, expected);
            }
        )*
        }
    }
    error_description_tests! {
        test_amount_promised_is_zero: (TribeError::AmountPromisedIsZero, "Amount promised in pico must be greater than 0"),
        test_can_not_invite_initial_founder: (TribeError::CanNotInviteInitialFounder, "The initial founder can not be invited to join their own tribe"),
        test_founder_already_invited: (TribeError::FounderAlreadyInvited, "AccountId already exists as a Founder"),
        test_founder_list_not_found: (TribeError::FounderListNotFound, "Tribe list of founders  not found"),
        test_founder_rejected_invitation: (TribeError::FounderRejectedInvitation, "Founder already rejected invitation to tribe"),
        test_founder_vote_action_pending: (TribeError::FounderVoteActionPending, "Founder has not taken an action on pending invitation"),
        test_funding_already_completed: (TribeError::FundingAlreadyCompleted, "Founder has already completed funding"),
        test_funding_amount_must_be_greater_than_zero: (TribeError::FundingAmountMustBeGreaterThanZero, "Funding amount must be greater than zero amount"),
        test_not_a_founder: (TribeError::NotAFounder, "AccountId is not a Founder"),
        test_not_initial_founder: (TribeError::NotInitialFounder, "AccountId is not the Initial Founder"),
        test_tribe_is_defunct: (TribeError::TribeIsDefunct, "Tribe is defunct and cannot accept any more activity"),
        test_tribe_is_locked: (TribeError::TribeIsLocked, "Tribe is locked due to founder activity"),
    }

}


