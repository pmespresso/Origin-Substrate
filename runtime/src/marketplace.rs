/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use codec::{Encode, Decode};
use rstd::prelude::Vec;
use sr_primitives::{
  RuntimeDebug,
  traits::{Zero}
};
use support::{decl_module, decl_storage, decl_event, dispatch::Result, ensure};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: balances::Trait + system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Clone, Copy, Decode, Encode, PartialEq, RuntimeDebug)]
pub enum OfferingStatus {
  Undefined,
  Created,
  Accepted,
  Disputed,
}

#[derive(Clone, Copy, Decode, Encode, PartialEq, RuntimeDebug)]
pub enum Ruling {
  Seller,
  Buyer,
  ComAndSeller,
  ComAndBuyer,
}

#[derive(Clone, Copy, Decode, Encode, PartialEq, RuntimeDebug)]
pub struct Listing<AccountId, Balance> {
  seller: AccountId,
  deposit: Balance,
  deposit_manager: AccountId
}

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
pub struct Offering<AccountId, Balance, BlockNumber> {
  value: Balance,         // Amount in Eth or ERC20 buyer is offering
  commission: Balance,    // Amount of commission earned if offer is finalized
  refund: Balance,        // Amount to refund buyer upon finalization
  buyer: AccountId,      // Buyer wallet / identity contract / other contract
  affiliate: AccountId,  // Address to send any commission
  arbitrator: AccountId, // Address that settles disputes
  // currency: Currency, // Currency of the listing
  finalizes: BlockNumber,     // Timestamp offer finalizes
  status: OfferingStatus,     // 0: Undefined, 1: Created, 2: Accepted, 3: Disputed  
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as MarketplaceModule {
		Listings get(listings): Vec<Listing<T::AccountId, T::Balance>>; // all listings vec
    ListingAtIndex get(listing_at_index): map u64 => Option<Listing<T::AccountId, T::Balance>>; // listing_id => Listing
    ListingsNonce get(listings_nonce): u64;
    Offerings get(offerings): map u64 => Vec<Offering<T::AccountId, T::Balance, T::BlockNumber>>; // listing_id => Vec<Offerings>
    OfferingsNonce get(offerings_nonce): u64;
    AllowedAffiliates get(allowed_affiliates) config(): map T::AccountId => bool; // whitelist of allowed affiliate accountIds, initialized at genesis in chain_spec.rs
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
    
    // FIXME: also transfer the actual balance to ... the listing itself?
		pub fn create_listing(origin, _deposit: T::Balance, _deposit_manager: T::AccountId, _ipfs_hash: T::Hash) -> Result {
      let sender = ensure_signed(origin)?;

      let listing = Listing {
        seller: sender.clone(),
        deposit: _deposit.clone(),
        deposit_manager: _deposit_manager.clone(), // key that controls how the deposit is distributed
      };

      // push the new listing
      <Listings<T>>::mutate(|l| l.push(listing.clone()));
      
      // increment nonce
      ListingsNonce::mutate(|nonce| *nonce + 1);

      let nonce = ListingsNonce::get();

      // map the listing to id
      <ListingAtIndex<T>>::insert(nonce, listing);

			Self::deposit_event(
        RawEvent::ListingCreated(sender, nonce, _ipfs_hash)
      );
			
      Ok(())
		}

    pub fn execute_ruling(
      origin,
      _listing_id: u64,
      _offer_id: u64,
      _ipfs_hash: T::Hash,
      _ruling: Ruling,
      _refund: T::Balance
    ) -> Result {
      let sender = ensure_signed(origin)?;

      if let Some(offer) = Self::offerings(_listing_id).get(_listing_id as usize) {
        ensure!(sender == offer.arbitrator, "sender must be arbitrator!");
        ensure!(_refund <= offer.value, "refund too high!");

        if offer.status == OfferingStatus::Accepted {
          return Err("offer has not yet been accepted");
        } else {
          let mut new_offer = offer.clone();

          new_offer.refund = _refund;

          <Offerings<T>>::mutate(&_listing_id, |offers| offers.push(new_offer));

          if _ruling == Ruling::Buyer {
              Self::refund_buyer(_listing_id, _offer_id);
          } else  {
              Self::pay_seller(_listing_id, _offer_id);
          }
          // if (_ruling & 2 == 2) {
              // payCommission(listingID, offerID);
          // } else  { // Refund commission to seller
              // listings[listingID].deposit += offer.commission;
          // }
          // emit OfferRuling(offer.arbitrator, listingID, offerID, _ipfsHash, _ruling);
          // delete offers[listingID][offerID];
        }
        // require(offer.status == 3, "status != disputed");
       
      }

      Ok(())
    }

    pub fn make_offer(
      origin,
      _listing_id: u64, // listing nonce
      _ipfs_hash: T::Hash, // ipfs hash container offer data
      _finalizes: T::BlockNumber, // block where accepted offer will finalize
      _affiliate: T::AccountId, // address to send any required commission to
      _commission: T::Balance, // amount of commission to send in Native Token if offer finalizes
      _value: T::Balance, // Offer amount in Native Token FIXME: would be cool to use offchain worker to get that value in USD
      _arbitrator: T::AccountId // Escrow arbitrator
    ) -> Result {
      let sender = ensure_signed(origin)?;

      let affiliate_whitelist_disabled = Self::allowed_affiliates(&sender);
      
      ensure!(!affiliate_whitelist_disabled, "Affiliate not allowed");

      let new_offering = Offering {
        status: OfferingStatus::Created,
        buyer: sender.clone(),
        finalizes: _finalizes,
        affiliate: _affiliate,
        commission: _commission,
        value: _value,
        arbitrator: _arbitrator,
        refund: Zero::zero()
      };

      <Offerings<T>>::mutate(&_listing_id, |curr_offerings| curr_offerings.push(new_offering));

      let num_of_offerings = Self::offerings_nonce();

      Self::deposit_event(
        RawEvent::OfferingCreated(sender, _listing_id, num_of_offerings, _ipfs_hash)
      );
      
      Ok(())
    }

    pub fn update_listing(origin, _listing_id: u64, _ipfs_hash: T::Hash, _additional_deposit: T::Balance) -> Result {
      let sender = ensure_signed(origin)?;
      
      if !_additional_deposit.is_zero() {
        //  FIXME: tokenAddr.transferFrom(_seller, this, _additionalDeposit);

        <ListingAtIndex<T>>::mutate(_listing_id, |listing_optional| {
          if let Some(listing) = listing_optional { listing.deposit += _additional_deposit; }
        });
      }
        
      Self::deposit_event(RawEvent::ListingUpdated(sender, _listing_id, _ipfs_hash));
        
      Ok(())
    }

    pub fn withdraw_listing(origin, _listing_id: u64, _target: T::AccountId, _ipfs_hash: T::Hash) -> Result {
      let sender = ensure_signed(origin)?;

      if let Some(listing) = Self::listing_at_index(_listing_id) {
        ensure!(sender == listing.deposit_manager, "only the deposit manager can withdraw this listing.");
      }

      // FIXME: also send the requested balance to _target

      Self::deposit_event(RawEvent::ListingWithdrawn(_target, _listing_id, _ipfs_hash));

      Ok(())
    }

    // emit events for associating ipfs hash of data regarding a listing and offering
    pub fn add_data(origin, _listing_id: u64, _offer_id: u64, _ipfs_hash: T::Hash) -> Result {
      let sender = ensure_signed(origin)?;

      Self::deposit_event(RawEvent::ListingData(_listing_id, _offer_id, _ipfs_hash));

      Self::deposit_event(RawEvent::OfferingData(_listing_id, _offer_id, _ipfs_hash));

      Self::deposit_event(RawEvent::MarketplaceData(sender, _ipfs_hash));

      Ok(())
    }

    pub fn add_affiliate(origin, _affiliate: T::AccountId, _ipfs_hash: T::Hash) -> Result {
      ensure_signed(origin)?;

      <AllowedAffiliates<T>>::insert(&_affiliate, true);

      Self::deposit_event(RawEvent::AffiliateAdded(_affiliate, _ipfs_hash));

      Ok(())
    }

    pub fn remove_affiliate(origin, _affiliate: T::AccountId, _ipfs_hash: T::Hash) -> Result {
      ensure_signed(origin)?;

      <AllowedAffiliates<T>>::insert(&_affiliate, false);

      Self::deposit_event(RawEvent::AffiliateRemoved(_affiliate, _ipfs_hash));

      Ok(())
    }
	}
}

impl<T: Trait> Module<T> {
  // @private called by module Refunds buyer in ETH or ERC20 - used by 1) executeRuling() and 2) to allow a seller to refund a purchase
  fn refund_buyer(_listing_id: u64, _offer_id: u64) -> Result {
    // let offer_optional = Self::offerings(_listing_id).get(_offer_id as usize);

    // if let Some(offer) = offer_optional {
      // ensure!(offer.currency == T::Balance::sa(0), "Refund failed");
      
      // FIXME actually send some balance in T::Currency

    Ok(())
  }

  // @dev Pay seller in T::Currency
  fn pay_seller(_listing_id: u64, _offer_id: u64) -> Result {
    // let listing_optional = Self::listings().get(_listing_id as usize);

    // Listing storage listing = listings[listingID];
    // Offer storage offer = offers[listingID][offerID];
    // uint value = offer.value - offer.refund;

    // if (address(offer.currency) == 0x0) {
    //     require(offer.buyer.send(offer.refund), "ETH refund failed");
    //     require(listing.seller.send(value), "ETH send failed");
    // } else {
    //     require(
    //         offer.currency.transfer(offer.buyer, offer.refund),
    //         "Refund failed"
    //     );
    //     require(
    //         offer.currency.transfer(listing.seller, value),
    //         "Transfer failed"
    //     );
    // }
    Ok(())
  }
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId,
  Hash = <T as system::Trait>::Hash {
    AffiliateAdded(AccountId, Hash), // _affilidate_id, ipfs hash
    AffiliateRemoved(AccountId, Hash), // _affilidate_id, ipfs hash
    ListingCreated (AccountId, u64, Hash), // party, listing_id , ipfs hash
    ListingData(u64, u64, Hash), // listing_id, offer_id, ipfs hash
    ListingUpdated (AccountId, u64, Hash), // party, listing_id, ipfs hash
    ListingWithdrawn(AccountId, u64, Hash), // _target, listing_id, _ipfs hash
    MarketplaceData(AccountId, Hash), // sender, ipfs hash
    OfferingCreated (AccountId, u64, u64, Hash), // party, listing_id, # of offerings, ipfs hash
    OfferingData(u64, u64, Hash), // listing_id, offer_id, ipfs hash
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, parameter_types};
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type WeightMultiplierUpdate = ();
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}

  impl balances::Trait for Test {
    type Balance = u64;
		type OnFreeBalanceZero = ();
		type OnNewAccount = ();
		type Event = ();
		type TransactionPayment = ();
		type TransferPayment = ();
		type DustRemoval = ();
		type WeightToFee = ();
    type ExistentialDeposit = ();
    type CreationFee = ();
    type TransferFee = ();
    type TransactionBaseFee = ();
    type TransactionByteFee = ();
  }

	impl super::Trait for Test {
		type Event = ();
	}

	type MarketplaceModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn create_listing() {
		with_externalities(&mut new_test_ext(), || {
			assert_ok!(MarketplaceModule::create_listing(Origin::signed(1), Zero::zero(), 1, Hash::default()));
		});
	}
}
