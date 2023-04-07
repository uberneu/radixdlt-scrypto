use radix_engine::blueprints::resource::FungibleResourceManagerSubstate;
use radix_engine::kernel::interpreters::*;
use radix_engine::system::bootstrap::{
    create_genesis, GenesisData, GenesisResource, GenesisValidator,
};
use radix_engine::transaction::{
    execute_transaction, BalanceChange, ExecutionConfig, FeeReserveConfig,
};
use radix_engine::types::*;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use radix_engine_stores::interface::{CommittableSubstateDatabase, SubstateDatabase};
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

#[test]
fn test_bootstrap_receipt_should_match_constants() {
    let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
    let substate_store = InMemorySubstateDatabase::standard();
    let validator_key = EcdsaSecp256k1PublicKey([0; 33]);
    let validator_address = ComponentAddress::virtual_account_from_public_key(&validator_key);
    let staker_address = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let mut stakes = BTreeMap::new();
    stakes.insert(0, vec![(0, Decimal::one())]);
    let genesis_data = GenesisData {
        validators: vec![GenesisValidator {
            key: validator_key,
            component_address: validator_address,
        }],
        resources: vec![],
        accounts: vec![staker_address],
        resource_balances: BTreeMap::new(),
        xrd_balances: BTreeMap::new(),
        stakes,
    };
    let genesis_transaction = create_genesis(genesis_data, 1u64, 1u64, 1u64);

    let transaction_receipt = execute_transaction(
        &substate_store,
        &scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::genesis(),
        &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
    );

    transaction_receipt
        .expect_commit(true)
        .next_epoch()
        .expect("There should be a new epoch.");

    assert!(transaction_receipt
        .expect_commit(true)
        .new_package_addresses()
        .contains(&PACKAGE_PACKAGE));
}

#[test]
fn test_genesis_xrd_allocation_to_accounts() {
    let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
    let mut substate_store = InMemorySubstateDatabase::standard();
    let account_public_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let account_component_address = ComponentAddress::virtual_account_from_public_key(
        &PublicKey::EcdsaSecp256k1(account_public_key.clone()),
    );
    let allocation_amount = dec!("100");
    let mut xrd_balances = BTreeMap::new();
    xrd_balances.insert(0, allocation_amount);
    let genesis_data = GenesisData {
        validators: vec![],
        resources: vec![],
        accounts: vec![account_component_address],
        resource_balances: BTreeMap::new(),
        xrd_balances,
        stakes: BTreeMap::new(),
    };
    let genesis_transaction = create_genesis(genesis_data, 1u64, 1u64, 1u64);

    let transaction_receipt = execute_transaction(
        &substate_store,
        &scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::genesis(),
        &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
    );

    let commit_result = transaction_receipt.expect_commit(true);
    substate_store.commit(&commit_result.state_updates).unwrap();

    assert!(transaction_receipt
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|rc| rc.amount == allocation_amount
            && rc.node_id.eq(account_component_address.as_node_id())
            && rc.resource_address == RADIX_TOKEN));
}

#[test]
fn test_genesis_resource_with_initial_allocation() {
    let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
    let mut substate_store = InMemorySubstateDatabase::standard();
    let tokenholder = ComponentAddress::virtual_account_from_public_key(
        &PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key()),
    );
    let allocation_amount = dec!("105");
    let address_bytes = hash(vec![1, 2, 3]).lower_26_bytes();
    let resource_address = NodeId::new(EntityType::GlobalFungibleResource as u8, &address_bytes);

    let owner = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(2).unwrap().public_key(),
    );

    let genesis_resource = GenesisResource {
        symbol: "TST".to_string(),
        name: "Test".to_string(),
        description: "A test resource".to_string(),
        url: "test".to_string(),
        icon_url: "test".to_string(),
        address_bytes: resource_address.into(),
        owner_with_mint_and_burn_rights: Some(1),
    };
    let mut resource_balances = BTreeMap::new();
    resource_balances.insert(0, vec![(0, allocation_amount)]);

    let genesis_data = GenesisData {
        resources: vec![genesis_resource],
        validators: vec![],
        accounts: vec![tokenholder.clone(), owner],
        resource_balances,
        xrd_balances: BTreeMap::new(),
        stakes: BTreeMap::new(),
    };

    let genesis_transaction = create_genesis(genesis_data, 1u64, 1u64, 1u64);

    let transaction_receipt = execute_transaction(
        &substate_store,
        &scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::genesis(),
        &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
    );

    let commit_result = transaction_receipt.expect_commit(true);
    substate_store.commit(&commit_result.state_updates).unwrap();

    let persisted_resource_manager_substate = substate_store
        .get_substate(
            &resource_address,
            SysModuleId::ObjectState.into(),
            &ResourceManagerOffset::ResourceManager.into(),
        )
        .unwrap()
        .unwrap();

    let resource_manager_substate: FungibleResourceManagerSubstate =
        scrypto_decode(&persisted_resource_manager_substate).unwrap();
    assert_eq!(resource_manager_substate.total_supply, dec!("105"));

    let persisted_symbol_metadata_entry = substate_store
        .get_substate(
            &resource_address,
            SysModuleId::Metadata.into(),
            &SubstateKey::from_vec(scrypto_encode("symbol").unwrap()).unwrap(),
        )
        .unwrap()
        .unwrap();

    let entry: Option<MetadataEntry> = scrypto_decode(&persisted_symbol_metadata_entry).unwrap();
    if let Some(MetadataEntry::Value(MetadataValue::String(symbol))) = entry {
        assert_eq!(symbol, "TST");
    } else {
        panic!("Resource symbol was not a string");
    }

    assert!(transaction_receipt
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|rc| rc.amount == allocation_amount
            && rc.node_id.eq(tokenholder.as_node_id())
            && rc.resource_address.as_node_id().eq(&resource_address)));

    assert!(transaction_receipt
        .execution_trace
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|rc|
            // Not an ideal condition, but assuming this is the owner badge
            rc.amount == dec!("1")
                && rc.node_id.eq(owner.as_node_id())));
}

#[test]
fn test_genesis_stake_allocation() {
    let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
    let mut substate_store = InMemorySubstateDatabase::standard();

    // There are two genesis validators
    // - one with two stakers (0 and 1)
    // - one with one staker (just 1)
    let validator_0: GenesisValidator = EcdsaSecp256k1PrivateKey::from_u64(10)
        .unwrap()
        .public_key()
        .into();
    let validator_1: GenesisValidator = EcdsaSecp256k1PrivateKey::from_u64(11)
        .unwrap()
        .public_key()
        .into();

    let staker_0 = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(4).unwrap().public_key(),
    );

    let staker_1 = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(5).unwrap().public_key(),
    );

    let mut stakes = BTreeMap::new();
    stakes.insert(0, vec![(0, dec!("10")), (1, dec!("50000"))]);
    stakes.insert(1, vec![(1, dec!("1"))]);

    let genesis_data = GenesisData {
        resources: vec![],
        validators: vec![validator_0, validator_1],
        accounts: vec![staker_0.clone(), staker_1.clone()],
        resource_balances: BTreeMap::new(),
        xrd_balances: BTreeMap::new(),
        stakes,
    };

    let genesis_transaction = create_genesis(genesis_data, 1u64, 1u64, 1u64);

    let transaction_receipt = execute_transaction(
        &substate_store,
        &scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::genesis(),
        &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
    );

    let commit_result = transaction_receipt.expect_commit(true);
    substate_store.commit(&commit_result.state_updates).unwrap();

    // Staker 0 should have one liquidity balance entry
    {
        let address: GlobalAddress = staker_0.into();
        let balances = commit_result
            .state_update_summary
            .balance_changes
            .get(&address)
            .unwrap();
        assert!(balances.len() == 1);
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("10"))));
    }

    // Staker 1 should have two liquidity balance entries
    {
        let address: GlobalAddress = staker_1.into();
        let balances = commit_result
            .state_update_summary
            .balance_changes
            .get(&address)
            .unwrap();
        assert!(balances.len() == 2);
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("1"))));
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("50000"))));
    }
}
