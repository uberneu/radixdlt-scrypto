use scrypto::prelude::*;

blueprint! {
    struct AuthComponent {
        some_non_fungible: NonFungibleAddress,
    }

    impl AuthComponent {
        pub fn create_component(some_non_fungible: NonFungibleAddress) -> ComponentAddress {
            Self { some_non_fungible }
                .instantiate()
                .auth(
                    Authorization::new()
                        .method("get_secret", method_auth!(require("some_non_fungible")))
                        .default(method_auth!(allow_all))
                )
                .globalize()
        }

        pub fn get_secret(&self) -> String {
            "Secret".to_owned()
        }

        pub fn update_auth(&mut self, some_non_fungible: NonFungibleAddress) {
            self.some_non_fungible = some_non_fungible;
        }
    }
}
