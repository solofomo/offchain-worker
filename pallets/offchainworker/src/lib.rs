#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::ValidateUnsigned,
    };
    use frame_system::pallet_prelude::*;
    use sp_core::crypto::KeyTypeId;
    use sp_runtime::{offchain::storage::StorageValueRef, traits::SaturatedConversion, TransactionValidity, RuntimeDebug};
    use sp_std::prelude::*;

    // Define crypto for signed payload
    pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");
    use sp_runtime::offchain::http;
    use sp_io::hashing::keccak_256;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn price_storage)]
    pub type PriceStorage<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PriceStored(u64),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            // Fetching the price (for example purposes, we'll just use a fixed value)
            let price: u64 = 50000; // Normally you'd fetch this from an HTTP API

            // Create a signed payload
            let payload = (price, block_number.saturated_into::<u64>());

            // Submit the unsigned transaction with the signed payload
            let call = Call::submit_price(payload);
            if let Err(e) = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()) {
                log::error!("Error in offchain_worker, submit_unsigned_transaction: {:?}", e);
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn submit_price(
            origin: OriginFor<T>,
            payload: (u64, u64),
        ) -> DispatchResultWithPostInfo {
            // Ensure that this is an unsigned transaction (not signed by anyone)
            ensure_none(origin)?;

            // Store the price and emit an event
            let (price, _block_number) = payload;
            PriceStorage::<T>::put(price);

            Self::deposit_event(Event::PriceStored(price));

            Ok(().into())
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            // Validate the call and check that the payload is signed
            // You should implement your logic for validating the signed payload here
            // For simplicity, this example does not actually validate the payload            // but in a real-world scenario, you would want to, for example, check the signature,
            // ensure the data has not been tampered with, etc.

            // Dummy implementation to accept the transaction:
            match call {
                Call::submit_price(payload) => {
                    // Validate the payload and check signature here
                    // ...

                    // For now, just accept it
                    ValidTransaction::with_tag_prefix("OffchainWorkerExample")
                        .priority(0)
                        .and_provides(payload)
                        .longevity(3)
                        .propagate(true)
                        .build()
                }
                _ => InvalidTransaction::Call.into(),
            }
        }
    }
}
