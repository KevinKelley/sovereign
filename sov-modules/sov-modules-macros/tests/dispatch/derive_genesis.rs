mod modules;

use modules::{first_test_module, second_test_module};
use sov_modules_api::{mocks::MockContext, Context, Module};
use sov_modules_macros::{DispatchQuery, Genesis};
use sov_state::JmtStorage;

// Debugging hint: To expand the macro in tests run: `cargo expand --test tests`
#[derive(Genesis, DispatchQuery)]
struct Runtime<C>
where
    C: Context,
{
    first: first_test_module::FirstTestStruct<C>,
    second: second_test_module::SecondTestStruct<C>,
}

fn main() {
    use sov_modules_api::{DispatchQuery, Genesis};

    type C = MockContext;
    let storage = JmtStorage::temporary();
    Runtime::<C>::genesis(storage.clone()).unwrap();

    {
        let message = RuntimeQuery::<C>::first(());
        let response = message.dispatch_query(storage.clone());
        assert_eq!(response.response, vec![1]);
    }

    {
        let message = RuntimeQuery::<C>::second(second_test_module::TestType {});
        let response = message.dispatch_query(storage.clone());
        assert_eq!(response.response, vec![2]);
    }
}
