use sov_modules_api::mocks::MockContext;
use sov_modules_api::{CallResponse, Context, Error, Module, ModuleInfo};
use sov_modules_macros::{Genesis, ModuleInfo};
use sov_state::{JmtStorage, StateMap};

pub mod first_test_module {
    use super::*;

    #[derive(ModuleInfo)]
    pub(crate) struct FirstTestStruct<C: Context> {
        #[state]
        pub state_in_first_struct: StateMap<C::PublicKey, u32, C::Storage>,
    }

    impl<C: Context> Module for FirstTestStruct<C> {
        type Context = C;
        type CallMessage = ();
        type QueryMessage = ();

        fn genesis(&mut self) -> Result<(), Error> {
            Ok(())
        }

        fn call(
            &mut self,
            msg: Self::CallMessage,
            context: &Self::Context,
        ) -> Result<CallResponse, Error> {
            todo!()
        }

        #[cfg(feature = "native")]
        fn query(&self, msg: Self::QueryMessage) -> sov_modules_api::QueryResponse {
            todo!()
        }
    }
}

pub mod second_test_module {
    use super::*;

    #[derive(borsh::BorshDeserialize, borsh::BorshSerialize)]
    pub struct TestType {}

    #[derive(ModuleInfo)]
    pub(crate) struct SecondTestStruct<C: Context> {
        #[state]
        pub state_in_second_struct: StateMap<C::PublicKey, u32, C::Storage>,
    }

    impl<C: Context> Module for SecondTestStruct<C> {
        type Context = C;
        type CallMessage = TestType;
        type QueryMessage = TestType;

        fn genesis(&mut self) -> Result<(), Error> {
            Ok(())
        }

        fn call(
            &mut self,
            msg: Self::CallMessage,
            context: &Self::Context,
        ) -> Result<CallResponse, Error> {
            todo!()
        }

        #[cfg(feature = "native")]
        fn query(&self, msg: Self::QueryMessage) -> sov_modules_api::QueryResponse {
            todo!()
        }
    }
}

#[derive(Genesis)]
struct Runtime<C: Context> {
    first: first_test_module::FirstTestStruct<C>,
    second: second_test_module::SecondTestStruct<C>,
}

fn main() {
    //panic!()
}
