use radix_engine::system::system_modules::costing::FeeTable;
use radix_engine::transaction::CostingParameters;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::types::*;
use radix_engine_interface::rule;
use scrypto_unit::*;
use transaction::prelude::*;
use transaction::validation::NotarizedTransactionValidator;
use transaction::validation::{TransactionValidator, ValidationConfig};

#[test]
fn test_transaction_preview_cost_estimate() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let network = NetworkDefinition::simulator();
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .drop_auth_zone_proofs()
        .build();
    let preview_flags = PreviewFlags {
        use_free_credit: true,
        assume_all_signature_proofs: false,
        skip_epoch_check: false,
    };
    let (notarized_transaction, preview_intent) = prepare_matching_test_tx_and_preview_intent(
        &mut test_runner,
        &network,
        manifest,
        &preview_flags,
    );
    let size_diff = manifest_encode(&notarized_transaction).unwrap().len()
        - manifest_encode(&preview_intent.intent).unwrap().len();

    // Act & Assert: Execute the preview, followed by a normal execution.
    // Ensure that both succeed and that the preview result provides an accurate cost estimate
    let preview_receipt = test_runner.preview(preview_intent, &network).unwrap();
    preview_receipt.expect_commit_success();
    let actual_receipt = test_runner.execute_transaction(
        validate(&network, &notarized_transaction).get_executable(),
        CostingParameters::default(),
        ExecutionConfig::for_notarized_transaction(network.clone())
            .with_kernel_trace(true)
            .with_cost_breakdown(true),
    );
    actual_receipt.expect_commit(true);
    assert_eq!(
        // TODO: better preview payload size estimate?
        preview_receipt
            .fee_summary
            .total_cost()
            .checked_add(
                Decimal::try_from(EXECUTION_COST_UNIT_PRICE_IN_XRD)
                    .unwrap()
                    .checked_mul(FeeTable::new().validate_tx_payload_cost(size_diff))
                    .unwrap()
            )
            .unwrap()
            .checked_add(
                Decimal::try_from(ARCHIVE_STORAGE_PRICE_IN_XRD)
                    .unwrap()
                    .checked_mul(size_diff)
                    .unwrap()
            )
            .unwrap()
            .checked_add(
                Decimal::try_from(EXECUTION_COST_UNIT_PRICE_IN_XRD)
                    .unwrap()
                    .checked_mul(FeeTable::new().verify_tx_signatures_cost(2))
                    .unwrap()
            )
            .unwrap(),
        actual_receipt.fee_summary.total_cost(),
    );
}

#[test]
fn test_transaction_preview_without_locking_fee() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let network = NetworkDefinition::simulator();
    let manifest = ManifestBuilder::new()
        // Explicitly don't lock fee from faucet
        .drop_auth_zone_proofs()
        .build();
    let preview_flags = PreviewFlags {
        use_free_credit: true,
        assume_all_signature_proofs: false,
        skip_epoch_check: false,
    };
    let (_, preview_intent) = prepare_matching_test_tx_and_preview_intent(
        &mut test_runner,
        &network,
        manifest,
        &preview_flags,
    );

    // Act
    let preview_receipt = test_runner.preview(preview_intent, &network).unwrap();
    let fee_summary = &preview_receipt.fee_summary;
    println!("{:?}", preview_receipt);
    assert!(fee_summary.total_execution_cost_in_xrd.is_positive());
    assert_eq!(fee_summary.total_tipping_cost_in_xrd, dec!("0"));
    assert!(fee_summary.total_storage_cost_in_xrd.is_positive()); // payload cost
    assert_eq!(fee_summary.total_royalty_cost_in_xrd, dec!("0"));
}

#[test]
fn test_assume_all_signature_proofs_flag_method_authorization() {
    // Arrange
    // Create an account component that requires a key auth for withdrawal
    let mut test_runner = TestRunnerBuilder::new().build();
    let network = NetworkDefinition::simulator();

    let public_key = Secp256k1PrivateKey::from_u64(99).unwrap().public_key();
    let withdraw_auth = rule!(require(NonFungibleGlobalId::from_public_key(&public_key)));
    let account = test_runner.new_account_advanced(OwnerRole::Fixed(withdraw_auth.clone()));
    let (_, _, other_account) = test_runner.new_allocated_account();

    let preview_flags = PreviewFlags {
        use_free_credit: true,
        assume_all_signature_proofs: true,
        skip_epoch_check: false,
    };

    // Check method authorization (withdrawal) without a proof in the auth zone
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500)
        .withdraw_from_account(account, XRD, 1)
        .try_deposit_entire_worktop_or_abort(other_account, None)
        .build();

    let (_, preview_intent) = prepare_matching_test_tx_and_preview_intent(
        &mut test_runner,
        &network,
        manifest,
        &preview_flags,
    );

    // Act
    let result = test_runner.preview(preview_intent, &network);

    // Assert
    result.unwrap().expect_commit_success();
}

fn prepare_matching_test_tx_and_preview_intent(
    test_runner: &mut DefaultTestRunner,
    network: &NetworkDefinition,
    manifest: TransactionManifestV1,
    flags: &PreviewFlags,
) -> (NotarizedTransactionV1, PreviewIntentV1) {
    let notary_priv_key = Secp256k1PrivateKey::from_u64(2).unwrap();
    let tx_signer_priv_key = Secp256k1PrivateKey::from_u64(3).unwrap();

    let notarized_transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(99),
            nonce: test_runner.next_transaction_nonce(),
            notary_public_key: notary_priv_key.public_key().into(),
            notary_is_signatory: false,
            tip_percentage: 0,
        })
        .manifest(manifest)
        .sign(&tx_signer_priv_key)
        .notarize(&notary_priv_key)
        .build();

    let preview_intent = PreviewIntentV1 {
        intent: notarized_transaction.signed_intent.intent.clone(),
        signer_public_keys: vec![tx_signer_priv_key.public_key().into()],
        flags: flags.clone(),
    };

    (notarized_transaction, preview_intent)
}

fn validate<'a>(
    network: &'a NetworkDefinition,
    transaction: &'a NotarizedTransactionV1,
) -> ValidatedNotarizedTransactionV1 {
    NotarizedTransactionValidator::new(ValidationConfig::default(network.id))
        .validate(transaction.prepare().unwrap())
        .unwrap()
}
