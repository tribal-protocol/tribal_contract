# tribal_contract

## To Build
`cargo +nightly contract build`


## To Test
`cargo +nightly contract test`

## methods
### `acceptTribe (): Result<Null, TribeContractErrorsTribeError>`
Mark the verified founder with a vote action of FOUNDER_ACCEPTED

### `fundTribe (): Result<u128, TribeContractErrorsTribeError>`
Can record multiple funding actions for the verified founder. Only available to founders who have already `accept_tribe`

### `inviteFounder (potentialFounder: AccountId, amountInPico: u128, required: bool): Result<Null, TribeContractErrorsTribeError>`
Attempt to include the `potential_founder` AccountId in the tribe contract’s founders collection. The initial founder must also provide the `amount_in_pico` to tribe and a flag to determine if this is a `required` founder

### `rejectTribe (): Result<Null, TribeContractErrorsTribeError>`
Attempts to mark the verified founder with a vote action of FOUNDER_REJECTED

### `getFounderStatus (founder: AccountId): Result<Text, TribeContractErrorsTribeError>`
Returns current state of the founder as json
