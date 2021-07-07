# Proof of Stake (PoS) system

## Epoch

An epoch is a range of blocks, whose length is set by the `epoch_duration` [parameter](#system-parameters). Epochs are identified by consecutive integers starting at 0. All the data relevant to PoS are [associated with epochs](#epoched-data).

An epoch starts at the beginning of block height `n` and end at the end of block height `n + epoch_duration - 1`. Then the following epoch starts at the beginning of block height `epoch_duration`, etc.

### Epoched data

Epoched data are data associated with a specific epoch that are set in advance. The data relevant to the PoS system in the ledger's state are epoched. Each data can be uniquely identified. These are:
- [Active validator set](#active-validator-set). A single value for each epoch.
- [Validators' consensus key, state and total bonded tokens](#validator). Identified by the validator's address.
- [Bonds](#bonds) are created by self-bonding and delegations. They are identified by the pair of delegator's address and the validator's address.

Changes to the epoched data do not take effect immediately. Instead, changes in epoch `n` are queued to take effect in the epoch `n + pipeline_length` for most cases and `n + unboding_length` for [unboding](#unbond) actions. Should the same validator's data or same bonds (i.e. with the same identity) be updated more than once in the same epoch, the later update overrides the previously queued-up update. For bonds, the token amounts are added up. Once the epoch `n` has ended, the queued-up updates for epoch `n + pipeline_length` become immutable.

## Entities

- [Validator](#validator): An account with a public consensus key, which may participate in producing blocks and governance activities. A validator may not also be a delegator.
- [Delegator](#delegator): An account that delegates some tokens to a validator. A delegator may not also be a validator.

Additionally, any account may submit evidence for [a slashable misbehaviour](#slashing).

### Validator

A validator must have a public consensus key. Additionally, it may also specify optional metadata fields (TBA).

A validator may be in one of the following states:
- *inactive*:
  A validator is not being considered for block creation and cannot receive any new delegations.
- *pending*:
  A validator has requested to become a *candidate*.
- *candidate*:
  A validator is considered for block creation and can receive delegations.

For each validator (in any state), the system also tracks total bonded tokens as a sum of the tokens in their self-bonds and delegated bonds, less any unbonded tokens. The total bonded tokens determine their voting voting power by division by the `votes_per_token` [parameter](#system-parameters). The voting power is used for validator selection for block creation and is used in governance related activities.

#### Validator actions

- *become validator*:
  Any account that is not a validator already and that doesn't have any delegations may request to become a validator. It is required to only provide a public consensus key. For the action applied in epoch `n`, the validator's state will be immediately set to *pending* and it will be set to *candidate* for epoch `n + pipeline_length`.
- *deactivate*:
  Only a *pending* or *candidate* validator account may *deactivate*. For this action applied in epoch `n`, the validator's account is set to become *inactive* in the epoch `n + pipeline_length`.
- *reactivate*:
  Only an *inactive* validator may *reactivate*. Similarly to *become validator* action, for this action applied in epoch `n`, the validator's state will be immediately set to *pending* and it will be set to *candidate* for epoch `n + pipeline_length`.
- *self-bond*:
  A validator may lock-up tokens into a [bond](#bonds) only for its own validator's address.
- *unbond*:
  Any self-bonded tokens may be partially or fully [unbonded](#unbond).
- *withdraw unbond*:
  Unbonded tokens may be withdrawn in or after the [unbond's epoch](#unbond).

#### Active validator set

From all the *candidate* validators, in each epoch the ones with the most voting power limited up to the `max_active_validators` [parameter](#system-parameters) are selected for the active validator set. The active validator set selected in epoch `n` is set for epoch `n + pipeline_length`.

### Delegator

A delegator may have any number number of delegations. Delegations are stored in [bonds](#bonds).

#### Delegator actions

- *delegate*:
  An account which is not a validator may delegate tokens to any number of validators. This will lock-up tokens into a [bond](#bonds).
- *undelegate*
  Any delegated tokens may be partially or fully [unbonded](#unbond).
- *withdraw unbond*:
  Unbonded tokens may be withdrawn in or after the [unbond's epoch](#unbond).

## Bonds

A bond locks-up tokens from validators' self-bonding and delegators' delegations. For self-bonding, the delegator's address is equal to the validator's address and the source of the bond is the validator's account. Only validators can self-bond. For a bond created from a delegation, the delegator's account is the bond's source.

For each epoch, bonds are uniquely identified by the pair of delegator's and validator's addresses. A bond created in epoch `n` is written into epoch `n + pipeline_length`. If there already is a bond in the epoch `n + pipeline_length` for this pair of delegator's and validator's addresses, its tokens are incremented by the newly bonded amount.

Any bonds created in epoch `n` increment the bond's validator's total bonded tokens by the bond's token amount for epoch `n + pipeline_length`.

The tokens put into a bond are immediately deducted from the source account.

### Unbond

An unbonding action (validator *unbond* or delegator *undelegate*) requested by the bond's source account in epoch `n` creates an "unbond" with epoch set to `n + unbounding_length`. We also store the epoch of the bond(s) from which the unbond is created in order to determine if the unbond should be slashed if a fault occurred within the range of bond epoch (inclusive) and unbond epoch (exclusive).

Any unbonds created in epoch `n` decrements the bond's validator's total bonded tokens by the bond's token amount for epoch `n + unbonding_length`.

An "unbond" with epoch set to `n` may be withdrawn by the bond's source address in or any time after the epoch `n`. Once withdrawn, the unbond is deleted and the tokens are credited to the source account.

### Staking rewards

TODO

### Slashing

Instead of absolute values, validators' total bonded token amounts and bonds' and unbonds' token amounts are stored as their deltas (i.e. the change of quantity from a previous epoch) to allow distinguishing changes for different epoch, which is essential for determining whether tokens be slashed. However, because slashes for a fault that occurred in epoch `n` may only be applied before the beginning of epoch `n + unbonding_length`, in epoch `m` we can sum all the deltas of total bonded token amounts and bonds and unbond with the same delegator and validator for epoch equal or less than `m - unboding_length` into a single total bonded token amount, single bond and single unbond record. This is to keep the total number of total bonded token amounts for a unique validator and bonds and unbonds for a unique pair of delegator and validator bound to a maximum number (equal to `unboding_length`).

To disincentivize validators misbehaviour in the PoS system a validator may be slashed for any fault that it has done. An evidence of misbehaviour may be submitted by any account for a fault that occurred in epoch `n` anytime before the beginning of epoch `n + unbonding_length`.

Actions that count as validator misbehaviour are:
- double signing two or more blocks at the same height
- going offline for a longer period (i.e. not producing signatures when available)

A valid evidence reduces the validator's total bonded token amount by the slash rate in and before the epoch in which the fault occurred. Additionally, a slash is stored with the misbehaving validator's address and the relevant epoch in which the fault occurred. When a bond is being unbonded or unbond withdrawn, we first look-up if any slash occurred within the range of epochs in which these were active and if so, reduce its token amount by the slash rate.

The invariant is that the sum of amounts that may be withdrawn from a misbehaving validator must always add up to the total bonded token amount.

## System parameters

- `epoch_duration`: Epoch duration in number of block, default `8640` (24 hours per epoch at the rate of 1 block every 10 secs)
- `max_active_validators`: Maximum active validators, default `128`
- `pipeline_length`: Pipeline length in number of epochs, default `2`
- `unboding_length`: Unbonding duration in number of epochs, default `6`
- `votes_per_token`: Used in validators' voting power calculation

## Storage

The [system parameters](#system-parameters) are written into the storage to allow for their changes.

The validators' data are keyed by the their addresses, conceptually:

```rust,ignore
type Validators = HashMap<Address, Validator>;
```

Epoched data are stored in the following structure:
```rust,ignore
struct Epoched<Data> {
  /// The epoch in which this data was last updated
  last_update: Epoch,
  /// Fixed-size array in which the head is the data for epoch in which the
  /// `last_update` was performed and every consecutive array element is the
  /// successor epoch of the predecessor array element.
  data: [Option<Data>; PIPELINE_LENGTH + 1],
}
```

Note that not all epochs will have data set, only the ones in which some changes occurred.

To try to look-up a value for `Epoched` data with independent values in each epoch (such as the active validator set) in the current epoch `n`:

1. let `index = min(n - last_update, pipeline_length)`
1. read the `data` field at `index`:
   1. if there's a value at `index` return it
   1. else if `index == 0`, return `None`
   1. else decrement `index` and repeat this sub-step from 1.

To look-up a value for `Epoched` data with delta values in the current epoch `n`:

1. let `end = min(n - last_update, pipeline_length) + 1`
1. sum all the values that are not `None` in the `0 .. end` range bounded inclusively below and exclusively above

To update a value in `Epoched` data with independent values in epoch `n` with value `new` for epoch `m`:

1. let `shift = min(n - last_update, pipeline_length)`
1. if `shift == 0`:
   1. `data[m - n] = new`
1. else:
   1. for `i in 0 .. shift` range bounded inclusively below and exclusively above, set `data[i] = None`
   1. rotate `data` left by `shift`
   1. set `data[m - n] = new`
   1. set `last_update` to the current epoch

To update a value in `Epoched` data with delta values in epoch `n` with value `delta` for epoch `m`:

1. let `shift = min(n - last_update, pipeline_length)`
1. if `shift == 0`:
   1. set `data[m - n] = data[m - n].map_or_else(delta, |last_delta| last_delta + delta)` (add the `delta` to the previous value, if any, otherwise use the `delta` as the value)
1. else:
   1. let `sum` to be equal to the sum of all delta values in the `i in 0 .. shift` range bounded inclusively below and exclusively above and set `data[i] = None`
   1. rotate `data` left by `shift`
   1. set `data[0] = data[0].map_or_else(sum, |last_delta| last_delta + sum)`
   1. set `data[m - n] = delta`
   1. set `last_update` to the current epoch

The invariants for updates in both cases are that `m - n >= 0` and `m - n <= pipeline_length`.

For the active validator set, we store all the active and inactive validators separately with their respective voting power:
```rust,ignore
type VotingPower = u64;

/// Validator's address with its voting power.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct WeighedValidator {
  /// The `voting_power` field must be on top, because lexicographic ordering is
  /// based on the top-to-bottom declaration order and in the `ValidatorSet`
  /// the `WeighedValidator`s these need to be sorted by the `voting_power`.
  voting_power: VotingPower,
  address: Address,
}

struct ValidatorSet {
  /// Active validator set with maximum size equal to `max_active_validators`
  active: BTreeSet<WeighedValidator>,
  /// All the other validators that are not active
  inactive: BTreeSet<WeighedValidator>,
}

type ValidatorSets = Epoched<ValidatorSet>;
```

When any validator's voting power changes, we attempt to perform the following update on the `ActiveValidatorSet`:

1. let `validator` be the validator's address, `power_before` and `power_after` be the voting power before and after the change, respectively
1. let `power_delta = power_after - power_before`
1. let `min_active = inactive.first()`
1. let `max_inactive = inactive.last()`
1. find whether the validator is active, let `is_active = power_before >= max_inactive.voting_power`
   1. if `is_active`:
      1. if `power_delta > 0 && power_after > max_inactive.voting_power`, update the validator in `active` set with `voting_power = power_after`
      1. else, remove the validator from `active`, insert it into `inactive` and remove `max_inactive.address` from `inactive` and insert it into `active`
   1. else (`!is_active`):
      1. if `power_delta < 0 && power_after < min_active.voting_power`, update the validator in `inactive` set with `voting_power = power_after`
      1. else, remove the validator from `inactive`, insert it into `active` and remove `min_active.address` from `active` and insert it into `inactive`

Within each validator's address space, we store public consensus key, state, total bonded token amount and voting power calculated from the total bonded token amount (even though the voting power is stored in the `ValidatorSet`, we also need to have the `voting_power` here because we cannot look it up in the `ValidatorSet` without iterating the whole set):

```rust,ignore
struct Validator {
  consensus_key: Epoched<PublicKey>,
  state: Epoched<ValidatorState>,
  total_deltas: Epoched<token::Amount>,
  voting_power: Epoched<VotingPower>,
}

enum ValidatorState {
  Inactive,
  Pending,
  Candidate,
}
```

The bonds and unbonds are keyed by their identifier:

```rust,ignore
type Bonds = HashMap<BondId, Epoched<Bond>>;
type Unbonds = HashMap<BondId, Epoched<Unbond>>;

struct BondId {
  validator: Address,
  delegator: Address,
}

struct Bond {
  delta: token::Amount,
}

struct Unbond {
  /// The epoch of the bond from which this unbond was created
  bond_epoch: Epoch,
  delta: token::Amount,
}
```

## Initialization

An initial validator set with self-bonded token amounts must be given on system initialization.

This set is used to pre-compute epochs in the genesis block from epoch `0` to epoch `pipeline_length - 1`.