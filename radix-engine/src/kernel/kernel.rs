use crate::blueprints::account::AccountSubstate;
use crate::blueprints::identity::Identity;
use crate::errors::RuntimeError;
use crate::errors::*;
use crate::kernel::kernel_api::{KernelSubstateApi, LockFlags};
use crate::kernel::KernelModule;
use crate::kernel::*;
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::SysBucket;
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, GlobalAddress, GlobalOffset, LockHandle, ProofOffset, RENodeId,
    SubstateId, SubstateOffset, WorktopOffset,
};
use radix_engine_interface::blueprints::resource::{
    require, AccessRule, AccessRuleKey, AccessRules, Bucket,
};
use radix_engine_interface::rule;
use sbor::rust::mem;

pub struct Kernel<
    'g, // Lifetime of values outliving all frames
    's, // Substate store lifetime
    W,  // WASM engine type
> where
    W: WasmEngine,
{
    /// Current execution mode, specifies permissions into state/invocations
    pub(super) execution_mode: ExecutionMode,
    /// Stack
    pub(super) current_frame: CallFrame,
    // This stack could potentially be removed and just use the native stack
    // but keeping this call_frames stack may potentially prove useful if implementing
    // execution pause and/or for better debuggability
    pub(super) prev_frame_stack: Vec<CallFrame>,
    /// Heap
    pub(super) heap: Heap,
    /// Store
    pub(super) track: &'g mut Track<'s>,

    /// ID allocator
    pub(super) id_allocator: &'g mut IdAllocator,
    /// Interpreter capable of running scrypto programs
    pub(super) scrypto_interpreter: &'g ScryptoInterpreter<W>,
    /// Kernel module mixer
    pub(super) module: KernelModuleMixer,
}

impl<'g, 's, W> Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    pub fn new(
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s>,
        scrypto_interpreter: &'g ScryptoInterpreter<W>,
        module: KernelModuleMixer,
    ) -> Self {
        Self {
            execution_mode: ExecutionMode::Kernel,
            heap: Heap::new(),
            track,
            scrypto_interpreter,
            id_allocator,
            current_frame: CallFrame::new_root(),
            prev_frame_stack: vec![],
            module,
        }
    }

    pub fn initialize(&mut self) -> Result<(), RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::KernelModule, |api| {
            KernelModuleMixer::on_init(api)
        })
    }

    pub fn teardown(mut self) -> (KernelModuleMixer, Option<RuntimeError>) {
        // Rewind call stack
        loop {
            if let Some(f) = self.prev_frame_stack.pop() {
                self.current_frame = f;
            } else {
                break;
            }
        }

        // Tear down kernel modules
        let result = self
            .execute_in_mode::<_, _, RuntimeError>(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::on_teardown(api)
            });

        match result {
            Ok(_) => (self.module, None),
            Err(e) => (self.module, Some(e)),
        }
    }

    fn create_virtual_account(
        &mut self,
        node_id: RENodeId,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Result<(), RuntimeError> {
        // TODO: Replace with trusted IndexedScryptoValue
        let access_rule = rule!(require(non_fungible_global_id));
        let component_id = {
            let kv_store_id = {
                let node_id = self.allocate_node_id(RENodeType::KeyValueStore)?;
                let node = RENodeInit::KeyValueStore;
                self.create_node(node_id, node, BTreeMap::new())?;
                node_id
            };

            let access_rules = {
                let mut access_rules = AccessRules::new();
                access_rules.set_access_rule_and_mutability(
                    AccessRuleKey::Native(NativeFn::Account(AccountFn::Deposit)),
                    AccessRule::AllowAll,
                    AccessRule::DenyAll,
                );
                access_rules.set_access_rule_and_mutability(
                    AccessRuleKey::Native(NativeFn::Account(AccountFn::DepositBatch)),
                    AccessRule::AllowAll,
                    AccessRule::DenyAll,
                );
                access_rules.default(access_rule.clone(), access_rule)
            };

            let node_id = {
                let mut node_modules = BTreeMap::new();
                node_modules.insert(
                    NodeModuleId::Metadata,
                    RENodeModuleInit::Metadata(MetadataSubstate {
                        metadata: BTreeMap::new(),
                    }),
                );
                let access_rules_substate = AccessRulesChainSubstate {
                    access_rules_chain: vec![access_rules],
                };
                node_modules.insert(
                    NodeModuleId::AccessRules,
                    RENodeModuleInit::AccessRulesChain(access_rules_substate),
                );
                let account_substate = AccountSubstate {
                    vaults: Own::KeyValueStore(kv_store_id.into()),
                };

                let node_id = self.allocate_node_id(RENodeType::Account)?;
                let node = RENodeInit::Account(account_substate);
                self.create_node(node_id, node, node_modules)?;
                node_id
            };
            node_id
        };

        // TODO: Use api to globalize component when create_node is refactored
        // TODO: to allow for address selection
        let global_substate = GlobalAddressSubstate::Account(component_id.into());

        self.current_frame.create_node(
            node_id,
            RENodeInit::Global(global_substate),
            BTreeMap::new(),
            &mut self.heap,
            &mut self.track,
            true,
        )?;

        Ok(())
    }

    fn create_virtual_identity(
        &mut self,
        node_id: RENodeId,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Result<(), RuntimeError> {
        let access_rule = rule!(require(non_fungible_global_id));
        let underlying_node_id = Identity::create(access_rule, self)?;

        // TODO: Use api to globalize component when create_node is refactored
        // TODO: to allow for address selection
        let global_substate = GlobalAddressSubstate::Identity(underlying_node_id.into());
        self.current_frame.create_node(
            node_id,
            RENodeInit::Global(global_substate),
            BTreeMap::new(),
            &mut self.heap,
            &mut self.track,
            true,
        )?;

        Ok(())
    }

    pub(super) fn try_virtualize(
        &mut self,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> Result<bool, RuntimeError> {
        match (node_id, offset) {
            (
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Global(GlobalOffset::Global),
            ) => {
                // Lazy create component if missing
                match component_address {
                    ComponentAddress::EcdsaSecp256k1VirtualAccount(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleLocalId::Bytes(address.into()),
                        );
                        self.create_virtual_account(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EddsaEd25519VirtualAccount(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleLocalId::Bytes(address.into()),
                        );
                        self.create_virtual_account(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EcdsaSecp256k1VirtualIdentity(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleLocalId::Bytes(address.into()),
                        );
                        self.create_virtual_identity(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EddsaEd25519VirtualIdentity(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleLocalId::Bytes(address.into()),
                        );
                        self.create_virtual_identity(node_id, non_fungible_global_id)?;
                    }
                    _ => return Ok(false),
                };

                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(super) fn drop_node_internal(
        &mut self,
        node_id: RENodeId,
    ) -> Result<HeapRENode, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::DropNode, |api| match node_id {
            RENodeId::Logger => Ok(()),
            RENodeId::TransactionRuntime => Ok(()),
            RENodeId::AuthZoneStack => {
                let handle = api.lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
                    LockFlags::MUTABLE,
                )?;
                let mut substate_ref_mut = api.get_ref_mut(handle)?;
                let auth_zone_stack = substate_ref_mut.auth_zone_stack();
                auth_zone_stack.clear_all();
                api.drop_lock(handle)?;
                Ok(())
            }
            RENodeId::Proof(..) => {
                let handle = api.lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Proof(ProofOffset::Proof),
                    LockFlags::MUTABLE,
                )?;
                let mut substate_ref_mut = api.get_ref_mut(handle)?;
                let proof = substate_ref_mut.proof();
                proof.drop();
                api.drop_lock(handle)?;
                Ok(())
            }
            RENodeId::Worktop => {
                let handle = api.lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Worktop(WorktopOffset::Worktop),
                    LockFlags::MUTABLE,
                )?;

                let buckets = {
                    let mut substate_ref_mut = api.get_ref_mut(handle)?;
                    let worktop = substate_ref_mut.worktop();
                    mem::replace(&mut worktop.resources, BTreeMap::new())
                };
                for (_, bucket) in buckets {
                    let bucket = Bucket(bucket.bucket_id());
                    if !bucket.sys_is_empty(api)? {
                        return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                            RENodeId::Worktop,
                        )));
                    }
                }

                api.drop_lock(handle)?;
                Ok(())
            }
            RENodeId::Bucket(..) => Ok(()),
            _ => Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                node_id,
            ))),
        })?;

        let node = self.current_frame.remove_node(&mut self.heap, node_id)?;
        for (_, substate) in &node.substates {
            let (_, child_nodes) = substate.to_ref().references_and_owned_nodes();
            for child_node in child_nodes {
                // Need to go through api so that visibility issues can be caught
                self.drop_node(child_node)?;
            }
        }
        // TODO: REmove
        Ok(node)
    }

    fn drop_nodes_in_frame(&mut self) -> Result<(), RuntimeError> {
        let mut worktops = Vec::new();
        let owned_nodes = self.current_frame.owned_nodes();

        // Need to go through api so that visibility issues can be caught
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Client, |api| {
            for node_id in owned_nodes {
                if let RENodeId::Worktop = node_id {
                    worktops.push(node_id);
                } else {
                    api.drop_node(node_id)?;
                }
            }
            for worktop_id in worktops {
                api.drop_node(worktop_id)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    fn run<X: Executor>(
        &mut self,
        executor: X,
        actor: ResolvedActor,
        mut call_frame_update: CallFrameUpdate,
    ) -> Result<X::Output, RuntimeError> {
        let derefed_lock = if let Some(ResolvedReceiver {
            derefed_from: Some((_, derefed_lock)),
            ..
        }) = &actor.receiver
        {
            Some(*derefed_lock)
        } else {
            None
        };
        let caller = self.current_frame.actor.clone();

        // Before push call frame
        {
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::before_push_frame(api, &actor, &mut call_frame_update)
            })?;
        }

        // Push call frame
        {
            self.id_allocator.push();

            let frame = CallFrame::new_child_from_parent(
                &mut self.current_frame,
                actor.clone(),
                call_frame_update,
            )?;
            let parent = mem::replace(&mut self.current_frame, frame);
            self.prev_frame_stack.push(parent);
        }

        // Execute
        let (output, update) = {
            // Handle execution start
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::on_execution_start(api, &caller)
            })?;

            // Run
            let (output, mut update) =
                self.execute_in_mode(ExecutionMode::Client, |api| executor.execute(api))?;

            // Handle execution finish
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::on_execution_finish(api, &caller, &mut update)
            })?;

            // Auto drop locks
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;

            // Auto-drop locks again in case module forgot to drop
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;

            (output, update)
        };

        // Pop call frame
        {
            let mut parent = self.prev_frame_stack.pop().unwrap();

            // Move resource
            CallFrame::update_upstream(&mut self.current_frame, &mut parent, update)?;

            // drop proofs and check resource leak
            self.drop_nodes_in_frame()?;

            // Restore previous frame
            self.current_frame = parent;

            self.id_allocator.pop()?;
        }

        // After pop call frame
        {
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::after_pop_frame(api)
            })?;
        }

        if let Some(derefed_lock) = derefed_lock {
            self.current_frame
                .drop_lock(&mut self.heap, &mut self.track, derefed_lock)?;
        }

        Ok(output)
    }

    pub fn node_method_deref(
        &mut self,
        node_id: RENodeId,
    ) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        if let RENodeId::Global(..) = node_id {
            let derefed =
                self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Deref, |api| {
                    let offset = SubstateOffset::Global(GlobalOffset::Global);
                    let handle =
                        api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::empty())?;
                    let substate_ref = api.get_ref(handle)?;
                    Ok((substate_ref.global_address().node_deref(), handle))
                })?;

            Ok(Some(derefed))
        } else {
            Ok(None)
        }
    }

    pub fn node_offset_deref(
        &mut self,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        if let RENodeId::Global(..) = node_id {
            if !matches!(offset, SubstateOffset::Global(GlobalOffset::Global)) {
                let derefed =
                    self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Deref, |api| {
                        let handle = api.lock_substate(
                            node_id,
                            NodeModuleId::SELF,
                            SubstateOffset::Global(GlobalOffset::Global),
                            LockFlags::empty(),
                        )?;
                        let substate_ref = api.get_ref(handle)?;
                        Ok((substate_ref.global_address().node_deref(), handle))
                    })?;

                Ok(Some(derefed))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn verify_valid_mode_transition(
        cur: &ExecutionMode,
        next: &ExecutionMode,
    ) -> Result<(), RuntimeError> {
        match (cur, next) {
            (ExecutionMode::Kernel, ..) => Ok(()),
            (ExecutionMode::Resolver, ExecutionMode::Deref) => Ok(()),
            _ => Err(RuntimeError::KernelError(
                KernelError::InvalidModeTransition(*cur, *next),
            )),
        }
    }

    pub(super) fn invoke_internal<X: Executor>(
        &mut self,
        executor: X,
        actor: ResolvedActor,
        call_frame_update: CallFrameUpdate,
    ) -> Result<X::Output, RuntimeError> {
        let depth = self.current_frame.depth;
        // TODO: Move to higher layer
        if depth == 0 {
            for node_id in &call_frame_update.node_refs_to_copy {
                match node_id {
                    RENodeId::Global(global_address) => {
                        if self.current_frame.get_node_location(*node_id).is_err() {
                            if matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                                )
                            ) || matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EddsaEd25519VirtualAccount(..)
                                )
                            ) || matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                                )
                            ) || matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EddsaEd25519VirtualIdentity(..)
                                )
                            ) {
                                self.current_frame
                                    .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                continue;
                            }

                            // TODO: Cleanup
                            {
                                if matches!(
                                    global_address,
                                    GlobalAddress::Package(RESOURCE_MANAGER_PACKAGE)
                                ) {
                                    self.current_frame
                                        .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                    continue;
                                }

                                if matches!(
                                    global_address,
                                    GlobalAddress::Package(IDENTITY_PACKAGE)
                                ) {
                                    self.current_frame
                                        .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                    continue;
                                }

                                if matches!(
                                    global_address,
                                    GlobalAddress::Package(EPOCH_MANAGER_PACKAGE)
                                ) {
                                    self.current_frame
                                        .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                    continue;
                                }

                                if matches!(global_address, GlobalAddress::Package(CLOCK_PACKAGE)) {
                                    self.current_frame
                                        .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                    continue;
                                }

                                if matches!(global_address, GlobalAddress::Package(ACCOUNT_PACKAGE))
                                {
                                    self.current_frame
                                        .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                    continue;
                                }

                                if matches!(
                                    global_address,
                                    GlobalAddress::Package(ACCESS_CONTROLLER_PACKAGE)
                                ) {
                                    self.current_frame
                                        .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                    continue;
                                }
                            }

                            let offset = SubstateOffset::Global(GlobalOffset::Global);

                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, NodeModuleId::SELF, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.track
                                .release_lock(
                                    SubstateId(*node_id, NodeModuleId::SELF, offset),
                                    false,
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.current_frame
                                .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                        }
                    }
                    RENodeId::Vault(..) => {
                        if self.current_frame.get_node_location(*node_id).is_err() {
                            let offset = SubstateOffset::Vault(VaultOffset::Vault);
                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, NodeModuleId::SELF, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.track
                                .release_lock(
                                    SubstateId(*node_id, NodeModuleId::SELF, offset),
                                    false,
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;

                            self.current_frame
                                .add_stored_ref(*node_id, RENodeVisibilityOrigin::DirectAccess);
                        }
                    }
                    _ => {}
                }
            }
        }

        let output = self.run(executor, actor, call_frame_update)?;

        Ok(output)
    }

    pub(super) fn execute_in_mode<X, RTN, E>(
        &mut self,
        execution_mode: ExecutionMode,
        execute: X,
    ) -> Result<RTN, RuntimeError>
    where
        RuntimeError: From<E>,
        X: FnOnce(&mut Self) -> Result<RTN, E>,
    {
        Self::verify_valid_mode_transition(&self.execution_mode, &execution_mode)?;

        // Save and replace kernel actor
        let saved = self.execution_mode;
        self.execution_mode = execution_mode;

        let rtn = execute(self)?;

        // Restore old kernel actor
        self.execution_mode = saved;

        Ok(rtn)
    }
}
