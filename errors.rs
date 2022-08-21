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

