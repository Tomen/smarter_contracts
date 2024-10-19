//! # The Button
//!
//! A simple ink! contract that allows users to press a button 
//! and claim a reward if 24 hours have passed since the last press.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(unexpected_cfgs)]

#[ink::contract]
mod the_button {

    #[ink(storage)]
    pub struct TheButton {
        /// The account of the last caller
        last_press_caller: AccountId,
        /// The timestamp of the last call
        last_press_timestamp: u64,
        /// How long the countdown is
        countdown_duration: u64,
    }

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// The countdown has not passed yet
        CountdownNotPassed,
        /// The caller has not paid enough balance
        NotEnoughBalance,
    }
    /// Type alias for the contract's `Result` type.
    pub type Result<T> = core::result::Result<T, Error>;


    impl TheButton {
        /// The constructor initializes the contract with the last caller's account id,
        /// the timestamp of the last call, and the countdown duration.
        /// `last_press_caller` and `last_press_timestamp` are optional parameters. If they are not provided,
        /// the contract is initialized with the caller's account id and the current block timestamp.
        #[ink(constructor)]
        pub fn new(last_press_caller: Option<AccountId>, last_press_timestamp: Option<u64>, countdown_duration: u64) -> Self {
            let last_press_caller = last_press_caller.unwrap_or(Self::env().caller());
            let last_press_timestamp = last_press_timestamp.unwrap_or(Self::env().block_timestamp());

            Self {
                last_press_caller,
                last_press_timestamp,
                countdown_duration,
            }
        }
        
        /// let's the caller press the button
        /// and stores the caller's account id
        /// the caller has to pay 1 unit of balance
        #[ink(message, payable)]
        pub fn press(&mut self) -> Result<()> {
            // ensure that the caller has paid at least 1 unit of balance
            let _transferred = self.env().transferred_value();
            if _transferred < 1 {
                return Err(Error::NotEnoughBalance);
            }

            self.last_press_caller = self.env().caller();
            self.last_press_timestamp = self.env().block_timestamp();

            Ok(())
        }

        /// If 24 hours have passed since the last call
        /// the caller can claim the reward
        #[ink(message)]
        pub fn payout(&mut self) -> Result<()> {
            let now = self.env().block_timestamp();
            let last_call = self.last_press_timestamp;
            let time_passed = now.checked_sub(last_call).unwrap();

            if time_passed < self.countdown_duration {
                return Err(Error::CountdownNotPassed);
            }

            // transfer the balance to the caller
            let balance = self.env().balance();
            let _result = self.env().transfer(self.last_press_caller, balance);

            Ok(())
        }

        /// Return the countdown until the next payout
        #[ink(message)]
        pub fn get_countdown(&self) -> u64 {
            let now = self.env().block_timestamp();
            let last_call = self.last_press_timestamp;
            let time_passed = now.checked_sub(last_call).unwrap();

            if time_passed >= self.countdown_duration {
                return 0;
            }

            self.countdown_duration.checked_sub(time_passed).unwrap()
        }

        /// Return the account id of the last caller
        #[ink(message)]
        pub fn get_last_press_caller(&self) -> AccountId {
            self.last_press_caller
        }

        /// Return the timestamp of the last call
        #[ink(message)]
        pub fn get_last_press_timestamp(&self) -> u64 {
            self.last_press_timestamp
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            // set up simulated environment
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let caller = accounts.alice;
            let block_timestamp = 12345;

            // Set the caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(caller);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp);

            // Initialize the contract
            let button = TheButton::new(None, None, 86400);

            // Check that the contract was initialized correctly
            assert_eq!(button.get_last_press_timestamp(), block_timestamp);
            assert_eq!(button.get_last_press_caller(), caller);
        }

        #[ink::test]
        fn press_works() {
            // GIVEN

            // set up simulated environment
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let block_timestamp = 12345;
            // Set the caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp);

            // Initialize the contract
            let mut button = TheButton::new(None, None, 86400);

            // WHEN
            // Set a new caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp + 1000);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1);

            // Press the button
            let result = button.press();

            // THEN
            // Check that the button was pressed successfully
            assert_eq!(result, Ok(()));
            assert_eq!(button.get_last_press_caller(), accounts.bob);
            assert_eq!(button.get_last_press_timestamp(), block_timestamp + 1000);
        }

        #[ink::test]
        fn payout_works() {
            // GIVEN

            // set up simulated environment
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let block_timestamp = 12345;
            // Set the caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp);

            // Initialize the contract
            let mut button = TheButton::new(None, None, 86400);

            // WHEN
            // Set a new caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp + 1);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1);

            // Press the button
            let _result = button.press();

            // Set a new caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp + 86401);

            // Payout the reward
            let result = button.payout();

            // THEN
            // Check that the reward was paid out successfully
            assert_eq!(result, Ok(()));
        }
    }


    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TheButtonRef::new(None, None, 86400);

            // When
            let contract = client
                .instantiate("the_button", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<TheButton>();

            // Then
            let get_countdown = call_builder.get_countdown();
            let get_countdown_result = client.call(&ink_e2e::alice(), &get_countdown).dry_run().await?;
            assert!(matches!(get_countdown_result.return_value(), 86400));

            Ok(())
        }
    }
}
