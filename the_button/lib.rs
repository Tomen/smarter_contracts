//! # The Button
//!
//! A simple ink! contract that allows users to press a button 
//! and claim a reward if 24 hours have passed since the last press.
//!
//! ## Overview
//! The idea of the contract is that it is a game where the last user to press the button wins a reward.
//! Users have to pay a certain amount to press the button.
//! If no other user presses the button within 24 hours, the last user to press the button wins the reward.
//! The reward is the balance of the contract.
//!
//! ## Details
//! The contract is initialized with the countdown duration in milliseconds.
//! Users can press the button by calling the `press()` function and paying `min_raise_balance`.
//! This resets the countdown.
//! If `countdown_duration` has passed since the last press, any user can claim the reward for the winner
//! by calling the `payout()` function.
//! The game ends when the reward is claimed and the contract will self-destruct.

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
        /// Minimum raised balance to press the button
        min_raise_balance: Balance,
    }

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// The countdown has not passed yet
        CountdownNotPassed,
        /// The caller has not paid enough balance
        InsertCoinToContinue,
    }
    /// Type alias for the contract's `Result` type.
    pub type Result<T> = core::result::Result<T, Error>;


    impl TheButton {
        /// The constructor initializes the contract countdown duration in milliseconds.
        /// The contract caller and timestamp are set to the caller and the block timestamp.
        #[ink(constructor)]
        pub fn new(countdown_duration: u64, min_raise_balance: Balance) -> Self {
            let last_press_caller = Self::env().caller();
            let last_press_timestamp = Self::env().block_timestamp();

            Self {
                last_press_caller,
                last_press_timestamp,
                countdown_duration,
                min_raise_balance,
            }
        }

        /// The default constructor initializes the contract with a countdown duration of 24 hours
        /// and a minimum raised balance of 1e10 units. (1 PAS, 1 DOT, 0.01 KSM)
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(86400 * 1000, 10_000_000_000)
        }
        
        /// The caller has to pay at least 1 unit of balance to press the button.
        /// The last caller and timestamp are updated. This resets the countdown.
        /// If the caller has not paid enough balance, the error `InsertCoinToContinue` is returned.
        #[ink(message, payable)]
        pub fn press(&mut self) -> Result<()> {
            // ensure that the caller has paid at least 1 unit of balance
            let _transferred = self.env().transferred_value();
            if _transferred < self.min_raise_balance {
                return Err(Error::InsertCoinToContinue);
            }

            self.last_press_caller = self.env().caller();
            self.last_press_timestamp = self.env().block_timestamp();

            Ok(())
        }

        /// Claims the reward if 24 hours have passed since the last press.
        /// The balance of the contract is transferred to the last user who pressed the button.
        /// If the countdown has not passed yet, the error `CountdownNotPassed` is returned.
        /// The contract is terminated after the reward is paid out. Any remaining balance is sent to the caller.
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

            self.env().terminate_contract(self.env().caller());
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

        /// Returns the contract balance. This is a convenience function to show the contract balance
        /// in contract explorers.
        #[ink(message)]
        pub fn get_balance(&self) -> Balance {
            self.env().balance()
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
            let button = TheButton::new(86400 * 1000, 1000);

            // Check that the contract was initialized correctly
            assert_eq!(button.get_last_press_timestamp(), block_timestamp);
            assert_eq!(button.get_last_press_caller(), caller);
        }

        #[ink::test]
        fn press_works() {
            // GIVEN

            // set up simulated environment
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let block_timestamp = 0;
            // Set the caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp);

            // Initialize the contract
            let mut button = TheButton::new(86400 * 1000, 1000);

            // WHEN
            // Set a new caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp + 1000);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1000);

            // Press the button
            let result = button.press();

            // THEN
            // Check that the button was pressed successfully
            assert_eq!(result, Ok(()));
            assert_eq!(button.get_last_press_caller(), accounts.bob);
            assert_eq!(button.get_last_press_timestamp(), block_timestamp + 1000);
        }
/*
        #[ink::test]
        fn payout_works() {
            // GIVEN

            // set up simulated environment
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let block_timestamp = 0;
            // Set the caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp);

            // Initialize the contract
            let mut button = TheButton::new(86400 * 1000, 1000);

            // WHEN
            // Set a new caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp + 1 * 1000);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1000);

            // Press the button
            let _result = button.press();

            // Set a new caller and block timestamp
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(block_timestamp + 86401 * 1000);

            // Payout the reward
            let result = button.payout();

            // THEN
            // Check that the reward was paid out successfully
        }
        */
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        const COUNTDOWN_DURATION: u64 = 86400 * 1000;
        const MIN_RAISE_BALANCE: Balance = 1000;
        
        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn contract_creation_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TheButtonRef::new(COUNTDOWN_DURATION);

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
            assert!(matches!(get_countdown_result.return_value(), COUNTDOWN_DURATION));

            let get_last_press_caller = call_builder.get_last_press_caller();
            let get_last_press_caller_result = client.call(&ink_e2e::alice(), &get_last_press_caller).await?;
            assert_eq!(get_last_press_caller_result.return_value(), ink_e2e::alice().account_id);

            let get_last_press_timestamp = call_builder.get_last_press_timestamp();
            let get_last_press_timestamp_result = client.call(&ink_e2e::alice(), &get_last_press_timestamp).await?;
            assert!(get_last_press_timestamp_result.return_value() > 0);

            Ok(())
        }

        #[ink_e2e::test]
        async fn press_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TheButtonRef::new(COUNTDOWN_DURATION, MIN_RAISE_BALANCE);

            let contract = client
                .instantiate("the_button", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<TheButton>();

            // When
            let press = call_builder.press().transferred_value(MIN_RAISE_BALANCE);
            let _result = client.call(&ink_e2e::bob(), &press).await?;

            // Then
            let get_last_press_caller = call_builder.get_last_press_caller();
            let get_last_press_caller_result = client.call(&ink_e2e::alice(), &get_last_press_caller).await?;
            assert_eq!(get_last_press_caller_result.return_value(), ink_e2e::bob().account_id);

            let get_countdown = call_builder.get_countdown();
            let get_countdown_result = client.call(&ink_e2e::alice(), &get_countdown).await?;
            assert_eq!(get_countdown_result.return_value(), COUNTDOWN_DURATION);
            
            Ok(())
        }

        /*
        #[ink_e2e::test]
        async fn payout_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given

            let mut constructor = TheButtonRef::new(COUNTDOWN_DURATION, MIN_RAISE_BALANCE);
            let contract = client
                .instantiate("the_button", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<TheButton>();

            // When
            let press = call_builder.press().transferred_value(MIN_RAISE_BALANCE);
            let _result = client.call(&ink_e2e::alice(), &press).await?;            
            client.advance_block().await?;

            let bobs_start_balance = client.get_balance(&ink_e2e::bob()).await?;

            _result = client.call(&ink_e2e::bob(), &press).await?;
            client.advance_block().await?;

            let bobs_balance_after_press = client.get_balance(&ink_e2e::bob()).await?;
            
            let payout = call_builder.payout();
            let result = client.call(&ink_e2e::bob(), &payout).await?;

            let bobs_balance_after_payout = client.get_balance(&ink_e2e::bob()).await?;

            // Then
            // the contract should have terminated
            contract.assert_terminated();
            assert_eq!(bobs_balance_after_press, bobs_start_balance - MIN_RAISE_BALANCE);
            assert_eq!(bobs_balance_after_payout, bobs_balance_after_press + contract_balance);
            
            Ok(())
        }

        #[ink_e2e::test]
        async fn press_fails_without_payment(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TheButtonRef::new(86400 * 1000);
            let contract = client
                .instantiate("the_button", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<TheButton>();

            // When
            let press = call_builder.press();
            let result = client.call(&ink_e2e::bob(), &press).await?;

            // Then
            assert_eq!(result.return_value(), Err(Error::InsertCoinToContinue));

            Ok(())
        }

        #[ink_e2e::test]
        async fn payout_fails_before_countdown(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TheButtonRef::new(86400 * 1000);
            let contract = client
                .instantiate("the_button", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<TheButton>();

            // When
            let press = call_builder.press().transferred_value(1);
            let _result = client.call(&ink_e2e::bob(), &press).await?;

            let payout = call_builder.payout();
            let result = client.call(&ink_e2e::bob(), &payout).await?;

            // Then
            assert_eq!(result.return_value(), Err(Error::CountdownNotPassed));

            Ok(())
        }
        */
    }

}
