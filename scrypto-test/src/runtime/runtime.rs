//! This module defines the test-runtime and its implementations, methods, and functions which is
//! the foundation of the invocation-based model of testing.

use crate::prelude::*;

/// A self-contained instance of the Radix Engine exposed through the [`ClientApi`] and
/// [`KernelApi`].
///
/// Each instance of [`TestRuntime`] has an [`InMemorySubstateDatabase`], [`Track`], and [`Kernel`]
/// which makes it a self-contained instance of the Radix Engine. It implements the [`ClientApi`]
/// and [`KernelApi`] making it a drop-in replacement for `ScryptoEnv` from Scrypto and the
/// [`SystemService`] from native.
pub struct TestRuntime(TestRuntimeInternal);

/// The internal implementation of the [`TestRuntime`].
///
/// This struct defines a self-contained instance of the Radix Engine that has all parts of the
/// engine stack from a substate store, all the way up to a kernel and VMs. The [`ouroboros`] crate
/// is used here to allow for the creation of a self-referencing struct. More specifically, this
/// crate allows for the use of the `'this` lifetime and allows members of the struct to hold
/// references to other members of the struct.
#[ouroboros::self_referencing(no_doc)]
struct TestRuntimeInternal {
    substate_db: InMemorySubstateDatabase,
    scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    native_vm: NativeVm<NoExtension>,
    id_allocator: IdAllocator,

    #[borrows(substate_db)]
    #[covariant]
    track: TestRuntimeTrack<'this>,

    #[borrows(scrypto_vm)]
    #[covariant]
    system_config: TestRuntimeSystemConfig<'this>,

    #[borrows(mut system_config, mut track, mut id_allocator)]
    #[not_covariant]
    kernel: TestRuntimeKernel<'this>,
}

/// Implements the [`ClientApi`] for the [`TestRuntime`] struct.
///
/// This macro exposes a high-level API for specifying the [`ClientApi`] traits to implement for the
/// [`TestRuntime`]. The trait methods are implements through a simple mechanism which creates a
/// [`SystemService`] object from the kernel and calls the trait method on the [`SystemService`]
/// object.
///
/// The syntax supported by this macro is as follows:
///
/// ```no_run
/// implement_client_api! {
///     trait_name: {
///         trait_method1: (args: ArgTypes) -> ReturnTypes,
///         trait_method2: (args: ArgTypes) -> ReturnTypes,
///     }
/// }
/// ```
///
/// This macro is only used internally in this crate for easy implementation of the [`ClientApi`]
/// and is not meant to be used outside or exported.
macro_rules! implement_client_api {
    (
        $(
            $trait: ident: {
                $(
                    $func_ident: ident: (
                        &mut self
                        $(, $input_ident: ident: $input_type: ty)* $(,)?
                    ) -> $outputs: ty
                ),* $(,)?
            }
        ),* $(,)*
    ) => {
        $(
            impl $trait<RuntimeError> for TestRuntime {
                $(
                    #[inline]
                    fn $func_ident(&mut self, $($input_ident: $input_type),*) -> $outputs {
                        self.0.with_kernel_mut(|kernel| {
                            SystemService {
                                api: kernel,
                                phantom: PhantomData,
                            }.$func_ident( $($input_ident),* )
                        })
                    }
                )*
            }
        )*
    };
}
implement_client_api! {
    ClientApi: {},
    ClientActorApi: {
        actor_get_blueprint_id: (&mut self) -> Result<BlueprintId, RuntimeError>,
        actor_open_field: (
            &mut self,
            object_handle: ObjectHandle,
            field: FieldIndex,
            flags: LockFlags,
        ) -> Result<FieldHandle, RuntimeError>,
        actor_is_feature_enabled: (
            &mut self,
            object_handle: ObjectHandle,
            feature: &str,
        ) -> Result<bool, RuntimeError>,
        actor_get_node_id: (&mut self) -> Result<NodeId, RuntimeError>,
        actor_get_outer_object: (&mut self) -> Result<GlobalAddress, RuntimeError>,
        actor_get_global_address: (&mut self) -> Result<GlobalAddress, RuntimeError>,
        actor_call_module: (
            &mut self,
            module_id: ObjectModuleId,
            method_name: &str,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
    },
    ClientActorIndexApi: {
        actor_index_insert: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            key: Vec<u8>,
            buffer: Vec<u8>,
        ) -> Result<(), RuntimeError>,
        actor_index_remove: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            key: Vec<u8>,
        ) -> Result<Option<Vec<u8>>, RuntimeError>,
        actor_index_scan_keys: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            limit: u32,
        ) -> Result<Vec<Vec<u8>>, RuntimeError>,
        actor_index_drain: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            limit: u32,
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, RuntimeError>,
    },
    ClientActorKeyValueEntryApi: {
        actor_open_key_value_entry: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            key: &Vec<u8>,
            flags: LockFlags,
        ) -> Result<KeyValueEntryHandle, RuntimeError>,
        actor_remove_key_value_entry: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            key: &Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
    },
    ClientActorSortedIndexApi: {
        actor_sorted_index_insert: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            sorted_key: SortedKey,
            buffer: Vec<u8>,
        ) -> Result<(), RuntimeError>,
        actor_sorted_index_remove: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            sorted_key: &SortedKey,
        ) -> Result<Option<Vec<u8>>, RuntimeError>,
        actor_sorted_index_scan: (
            &mut self,
            object_handle: ObjectHandle,
            collection_index: CollectionIndex,
            count: u32,
        ) -> Result<Vec<(SortedKey, Vec<u8>)>, RuntimeError>,
    },
    ClientBlueprintApi: {
        call_function: (
            &mut self,
            package_address: PackageAddress,
            blueprint_name: &str,
            function_name: &str,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>
    },
    ClientFieldApi: {
        field_read: (&mut self, handle: field_api::FieldHandle) -> Result<Vec<u8>, RuntimeError>,
        field_write: (&mut self, handle: FieldHandle, buffer: Vec<u8>) -> Result<(), RuntimeError>,
        field_lock: (&mut self, handle: FieldHandle) -> Result<(), RuntimeError>,
        field_close: (&mut self, handle: FieldHandle) -> Result<(), RuntimeError>
    },
    ClientKeyValueEntryApi: {
        key_value_entry_get: (&mut self, handle: KeyValueEntryHandle) -> Result<Vec<u8>, RuntimeError>,
        key_value_entry_set: (
            &mut self,
            handle: KeyValueEntryHandle,
            buffer: Vec<u8>,
        ) -> Result<(), RuntimeError>,
        key_value_entry_remove: (&mut self, handle: KeyValueEntryHandle) -> Result<Vec<u8>, RuntimeError>,
        key_value_entry_lock: (&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError>,
        key_value_entry_close: (&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError>,
    },
    ClientKeyValueStoreApi: {
        key_value_store_new: (&mut self, generic_args: KeyValueStoreGenericArgs) -> Result<NodeId, RuntimeError>,
        key_value_store_open_entry: (
            &mut self,
            node_id: &NodeId,
            key: &Vec<u8>,
            flags: LockFlags,
        ) -> Result<KeyValueEntryHandle, RuntimeError>,
        key_value_store_remove_entry: (
            &mut self,
            node_id: &NodeId,
            key: &Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
    },
    ClientObjectApi: {
        new_object: (
            &mut self,
            blueprint_ident: &str,
            features: Vec<&str>,
            generic_args: GenericArgs,
            fields: Vec<FieldValue>,
            kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
        ) -> Result<NodeId, RuntimeError>,
        drop_object: (&mut self, node_id: &NodeId) -> Result<Vec<Vec<u8>>, RuntimeError>,
        get_blueprint_id: (&mut self, node_id: &NodeId) -> Result<BlueprintId, RuntimeError>,
        get_outer_object: (&mut self, node_id: &NodeId) -> Result<GlobalAddress, RuntimeError>,
        allocate_global_address: (
            &mut self,
            blueprint_id: BlueprintId,
        ) -> Result<(GlobalAddressReservation, GlobalAddress), RuntimeError>,
        allocate_virtual_global_address: (
            &mut self,
            blueprint_id: BlueprintId,
            global_address: GlobalAddress,
        ) -> Result<GlobalAddressReservation, RuntimeError>,
        get_reservation_address: (&mut self, node_id: &NodeId) -> Result<GlobalAddress, RuntimeError>,
        globalize: (
            &mut self,
            modules: BTreeMap<ObjectModuleId, NodeId>,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Result<GlobalAddress, RuntimeError>,
        globalize_with_address_and_create_inner_object_and_emit_event: (
            &mut self,
            modules: BTreeMap<ObjectModuleId, NodeId>,
            address_reservation: GlobalAddressReservation,
            inner_object_blueprint: &str,
            inner_object_fields: Vec<FieldValue>,
            event_name: String,
            event_data: Vec<u8>,
        ) -> Result<(GlobalAddress, NodeId), RuntimeError>,
        call_method_advanced: (
            &mut self,
            receiver: &NodeId,
            module_id: ObjectModuleId,
            direct_access: bool,
            method_name: &str,
            args: Vec<u8>,
        ) -> Result<Vec<u8>, RuntimeError>,
    },
    ClientAuthApi: {
        get_auth_zone: (&mut self) -> Result<NodeId, RuntimeError>,
    },
    ClientExecutionTraceApi: {
        update_instruction_index: (&mut self, new_index: usize) -> Result<(), RuntimeError>,
    },
    ClientTransactionRuntimeApi: {
        get_transaction_hash: (&mut self) -> Result<Hash, RuntimeError>,
        generate_ruid: (&mut self) -> Result<[u8; 32], RuntimeError>,
        emit_log: (&mut self, level: Level, message: String) -> Result<(), RuntimeError>,
        emit_event: (&mut self, event_name: String, event_data: Vec<u8>) -> Result<(), RuntimeError>,
        panic: (&mut self, message: String) -> Result<(), RuntimeError>,
    },
    ClientCostingApi: {
        consume_cost_units: (&mut self, costing_entry: ClientCostingEntry) -> Result<(), RuntimeError>,
        credit_cost_units: (
            &mut self,
            vault_id: NodeId,
            locked_fee: LiquidFungibleResource,
            contingent: bool,
        ) -> Result<LiquidFungibleResource, RuntimeError>,
        execution_cost_unit_limit: (&mut self) -> Result<u32, RuntimeError>,
        execution_cost_unit_price: (&mut self) -> Result<Decimal, RuntimeError>,
        finalization_cost_unit_limit: (&mut self) -> Result<u32, RuntimeError>,
        finalization_cost_unit_price: (&mut self) -> Result<Decimal, RuntimeError>,
        usd_price: (&mut self) -> Result<Decimal, RuntimeError>,
        max_per_function_royalty_in_xrd: (&mut self) -> Result<Decimal, RuntimeError>,
        tip_percentage: (&mut self) -> Result<u32, RuntimeError>,
        fee_balance: (&mut self) -> Result<Decimal, RuntimeError>,
    }
}
