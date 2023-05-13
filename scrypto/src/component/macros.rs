use crate::component::*;
use crate::engine::scrypto_env::ScryptoEnv;
use radix_engine_interface::api::*;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::scrypto_encode;
use sbor::rust::prelude::*;

#[macro_export]
macro_rules! borrow_package {
    ($address:expr) => {
        $crate::component::BorrowedPackage($address.clone())
    };
}

/// Instantiates a component.
pub fn create_component<T: ComponentState<C>, C: Component + LocalComponent>(
    blueprint_name: &str,
    state: T,
) -> ComponentHandle {
    let mut env = ScryptoEnv;
    let node_id = env
        .new_simple_object(blueprint_name, vec![scrypto_encode(&state).unwrap()])
        .unwrap();
    ComponentHandle::Own(Own(node_id))
}
