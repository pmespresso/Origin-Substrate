/// runtime module implementing the ERC20 token interface
/// with added lock and unlock functions for staking in TCR runtime
/// implements a custom type `TokenBalance` for representing account balance
/// `TokenBalance` type is exactly the same as the `Balance` type in `balances` SRML module

use rstd::prelude::*;
use codec::Codec;
use support::{dispatch::Result, StorageMap, Parameter, StorageValue, decl_storage, decl_module, decl_event, ensure};
use system::{self, ensure_signed};
use sr_primitives::traits::{CheckedSub, CheckedAdd, Member, SimpleArithmetic};
use runtime_io::{with_storage, StorageOverlay, ChildrenStorageOverlay};

use constants::{Constants};

pub trait Trait: system::Trait + assets::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy;
    
}

/*
  In Ethereum smart contract world, I can have an ERC20 token that's native to my protocol, and then allow subtokens to be deployed at runtime.

  In Substrate runtime world, I have module acting as a ERC20 would, but to deploy a sub-token would involve deploying a new runtime...(is this true)?

  e.g.
   Origin Marketplace (Native token -> Origin)
    Digital Art => Digital Art Token
    Ad Space => Ad Space Token
*/

// storage for this runtime module
decl_storage! {
  trait Store for Module<T: Trait> as OriginToken {
    // bool flag to allow init to be called only once
    Init get(is_init): bool;
    // total supply of the token
    TotalSupply get(total_supply) config(): T::TokenBalance;
    // mapping of balances to accounts
    BalanceOf get(balance_of): map T::AccountId => T::TokenBalance;
    // mapping of allowances to accounts
    Allowance get(allowance): map (T::AccountId, T::AccountId) => T::TokenBalance;
  }

  add_extra_genesis {
    config(total_supply): Trait::TokenBalance;

    build(|storage: &mut StorageOverlay, _: &mut ChildrenStorageOverlay, config: &GenesisConfig<T>| {
      with_storage(storage, || {
        <TotalSupply<T>>::put(Constants::STARTING_SUPPLY);
      }
    }
  }
}

// public interface for this runtime module
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
      // initialize the default event for this module
      fn deposit_event() = default;

  }
}

// events
decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, TokenBalance = <T as self::Trait>::TokenBalance {
        Transfer(AccountId, AccountId, TokenBalance), // from, to, value
        Approval(AccountId, AccountId, TokenBalance), // owner, spender, value
    }
);