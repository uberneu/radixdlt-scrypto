use crate::errors::InvokeError;
use crate::errors::RuntimeError;
use crate::types::*;
use crate::vm::wasm::*;
use radix_engine_interface::api::actor_api::EventFlags;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::key_value_store_api::KeyValueStoreDataSchema;
use radix_engine_interface::api::{ActorRefHandle, AttachedModuleId, ClientApi, FieldValue};
use radix_engine_interface::types::ClientCostingEntry;
use radix_engine_interface::types::Level;
use sbor::rust::vec::Vec;

/// A shim between ClientApi and WASM, with buffer capability.
pub struct ScryptoRuntime<'y, Y>
where
    Y: ClientApi<RuntimeError>,
{
    api: &'y mut Y,
    buffers: IndexMap<BufferId, Vec<u8>>,
    next_buffer_id: BufferId,
    package_address: PackageAddress,
    export_name: String,
    wasm_execution_units_buffer: u32,
    max_number_of_buffers: usize,
}

impl<'y, Y> ScryptoRuntime<'y, Y>
where
    Y: ClientApi<RuntimeError>,
{
    pub fn new(api: &'y mut Y, package_address: PackageAddress, export_name: String) -> Self {
        ScryptoRuntime {
            api,
            buffers: index_map_new(),
            next_buffer_id: 0,
            package_address,
            export_name,
            wasm_execution_units_buffer: 0,
            max_number_of_buffers: MAX_NUMBER_OF_BUFFERS,
        }
    }

    pub fn parse_blueprint_id(
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
    ) -> Result<(PackageAddress, String), InvokeError<WasmRuntimeError>> {
        let package_address = PackageAddress::try_from(package_address.as_slice())
            .map_err(|_| WasmRuntimeError::InvalidPackageAddress)?;
        let blueprint_name =
            String::from_utf8(blueprint_name).map_err(|_| WasmRuntimeError::InvalidString)?;
        Ok((package_address, blueprint_name))
    }
}

impl<'y, Y> WasmRuntime for ScryptoRuntime<'y, Y>
where
    Y: ClientApi<RuntimeError>,
{
    fn allocate_buffer(
        &mut self,
        buffer: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        assert!(buffer.len() <= 0xffffffff);

        if self.buffers.len() >= self.max_number_of_buffers {
            return Err(InvokeError::SelfError(WasmRuntimeError::TooManyBuffers));
        }

        let id = self.next_buffer_id;
        let len = buffer.len();

        self.buffers.insert(id, buffer);
        self.next_buffer_id += 1;

        Ok(Buffer::new(id, len as u32))
    }

    fn buffer_consume(
        &mut self,
        buffer_id: BufferId,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        self.buffers
            .remove(&buffer_id)
            .ok_or(InvokeError::SelfError(WasmRuntimeError::BufferNotFound(
                buffer_id,
            )))
    }

    fn object_call(
        &mut self,
        receiver: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let receiver = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(receiver.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let ident = String::from_utf8(ident).map_err(|_| WasmRuntimeError::InvalidString)?;
        let return_data = self.api.call_method(&receiver, ident.as_str(), args)?;

        self.allocate_buffer(return_data)
    }

    fn object_call_module(
        &mut self,
        receiver: Vec<u8>,
        module_id: u32,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let receiver = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(receiver.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let ident = String::from_utf8(ident).map_err(|_| WasmRuntimeError::InvalidString)?;
        let module_id = u8::try_from(module_id)
            .ok()
            .and_then(|x| AttachedModuleId::from_repr(x))
            .ok_or(WasmRuntimeError::InvalidAttachedModuleId(module_id))?;

        let return_data =
            self.api
                .call_module_method(&receiver, module_id, ident.as_str(), args)?;

        self.allocate_buffer(return_data)
    }

    fn object_call_direct(
        &mut self,
        receiver: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let receiver = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(receiver.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let ident = String::from_utf8(ident).map_err(|_| WasmRuntimeError::InvalidString)?;
        let return_data = self
            .api
            .call_direct_access_method(&receiver, ident.as_str(), args)?;

        self.allocate_buffer(return_data)
    }

    fn blueprint_call(
        &mut self,
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
        function_ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let (package_address, blueprint_name) =
            Self::parse_blueprint_id(package_address, blueprint_name)?;
        let function_ident =
            String::from_utf8(function_ident).map_err(|_| WasmRuntimeError::InvalidString)?;

        let return_data = self.api.call_function(
            package_address,
            blueprint_name.as_str(),
            &function_ident,
            args,
        )?;

        self.allocate_buffer(return_data)
    }

    fn object_new(
        &mut self,
        blueprint_name: Vec<u8>,
        object_states: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let blueprint_name =
            String::from_utf8(blueprint_name).map_err(|_| WasmRuntimeError::InvalidString)?;
        let object_states = scrypto_decode::<IndexMap<u8, FieldValue>>(&object_states)
            .map_err(WasmRuntimeError::InvalidObjectStates)?;

        let component_id = self
            .api
            .new_simple_object(blueprint_name.as_ref(), object_states)?;

        self.allocate_buffer(component_id.to_vec())
    }

    fn address_allocate(
        &mut self,
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let (package_address, blueprint_name) =
            Self::parse_blueprint_id(package_address, blueprint_name)?;

        let address_reservation_and_address = self.api.allocate_global_address(BlueprintId {
            package_address,
            blueprint_name,
        })?;
        let encoded = scrypto_encode(&address_reservation_and_address)
            .expect("Failed to encode object address");

        self.allocate_buffer(encoded)
    }

    fn address_get_reservation_address(
        &mut self,
        node_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );

        let address = self.api.get_reservation_address(&node_id)?;
        let address_encoded = scrypto_encode(&address).expect("Failed to encode address");

        self.allocate_buffer(address_encoded)
    }

    fn globalize_object(
        &mut self,
        node_id: Vec<u8>,
        modules: Vec<u8>,
        address_reservation: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let modules = scrypto_decode::<IndexMap<AttachedModuleId, NodeId>>(&modules)
            .map_err(WasmRuntimeError::InvalidModules)?;
        let address_reservation =
            scrypto_decode::<Option<GlobalAddressReservation>>(&address_reservation)
                .map_err(|_| WasmRuntimeError::InvalidGlobalAddressReservation)?;

        let address = self.api.globalize(node_id, modules, address_reservation)?;

        self.allocate_buffer(address.to_vec())
    }

    fn key_value_store_new(
        &mut self,
        schema: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let schema = scrypto_decode::<KeyValueStoreDataSchema>(&schema)
            .map_err(WasmRuntimeError::InvalidKeyValueStoreSchema)?;

        let key_value_store_id = self.api.key_value_store_new(schema)?;

        self.allocate_buffer(key_value_store_id.to_vec())
    }

    fn key_value_store_open_entry(
        &mut self,
        node_id: Vec<u8>,
        key: Vec<u8>,
        flags: u32,
    ) -> Result<SubstateHandle, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );

        let flags = LockFlags::from_bits(flags).ok_or(WasmRuntimeError::InvalidLockFlags)?;
        let handle = self.api.key_value_store_open_entry(&node_id, &key, flags)?;

        Ok(handle)
    }

    fn key_value_entry_get(
        &mut self,
        handle: u32,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let value = self.api.key_value_entry_get(handle)?;
        self.allocate_buffer(value)
    }

    fn key_value_entry_set(
        &mut self,
        handle: u32,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.key_value_entry_set(handle, data)?;
        Ok(())
    }

    fn key_value_entry_remove(
        &mut self,
        handle: u32,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let value = self.api.key_value_entry_remove(handle)?;
        self.allocate_buffer(value)
    }

    fn key_value_entry_close(&mut self, handle: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.key_value_entry_close(handle)?;
        Ok(())
    }

    fn key_value_store_remove_entry(
        &mut self,
        node_id: Vec<u8>,
        key: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let rtn = self.api.key_value_store_remove_entry(&node_id, &key)?;
        self.allocate_buffer(rtn)
    }

    fn actor_open_field(
        &mut self,
        object_handle: u32,
        field: u8,
        flags: u32,
    ) -> Result<SubstateHandle, InvokeError<WasmRuntimeError>> {
        let flags = LockFlags::from_bits(flags).ok_or(WasmRuntimeError::InvalidLockFlags)?;
        let handle = self.api.actor_open_field(object_handle, field, flags)?;

        Ok(handle)
    }

    fn field_entry_read(
        &mut self,
        handle: SubstateHandle,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let substate = self.api.field_read(handle)?;

        self.allocate_buffer(substate)
    }

    fn field_entry_write(
        &mut self,
        handle: SubstateHandle,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.field_write(handle, data)?;

        Ok(())
    }

    fn field_entry_close(
        &mut self,
        handle: SubstateHandle,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.field_close(handle)?;

        Ok(())
    }

    fn actor_get_node_id(
        &mut self,
        actor_ref_handle: ActorRefHandle,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = self.api.actor_get_node_id(actor_ref_handle)?;

        self.allocate_buffer(node_id.0.to_vec())
    }

    fn actor_get_package_address(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let blueprint_id = self.api.actor_get_blueprint_id()?;

        self.allocate_buffer(blueprint_id.package_address.to_vec())
    }

    fn actor_get_blueprint_name(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let blueprint_id = self.api.actor_get_blueprint_id()?;

        self.allocate_buffer(blueprint_id.blueprint_name.into_bytes())
    }

    fn consume_wasm_execution_units(
        &mut self,
        mut n: u32,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        // Use buffer
        let min = u32::min(self.wasm_execution_units_buffer, n);
        self.wasm_execution_units_buffer -= min;
        n -= min;

        // If not covered, request from the system
        if n > 0 {
            let amount_to_request = n
                .checked_add(WASM_EXECUTION_COST_UNITS_BUFFER)
                .unwrap_or(u32::MAX);
            self.api
                .consume_cost_units(ClientCostingEntry::RunWasmCode {
                    package_address: &self.package_address,
                    export_name: &self.export_name,
                    wasm_execution_units: amount_to_request,
                })
                .map_err(InvokeError::downstream)?;
            self.wasm_execution_units_buffer += amount_to_request - n;
        }

        Ok(())
    }

    fn instance_of(
        &mut self,
        object_id: Vec<u8>,
        package_address: Vec<u8>,
        blueprint_name: Vec<u8>,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let object_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(object_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let (package_address, blueprint_name) =
            Self::parse_blueprint_id(package_address, blueprint_name)?;
        let blueprint_id = self.api.get_blueprint_id(&object_id)?;

        if blueprint_id.package_address.eq(&package_address)
            && blueprint_id.blueprint_name.eq(&blueprint_name)
        {
            Ok(1)
        } else {
            Ok(0)
        }
    }

    fn blueprint_id(
        &mut self,
        object_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let object_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(object_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let blueprint_id = self.api.get_blueprint_id(&object_id)?;

        let mut buf = Vec::new();
        buf.extend(blueprint_id.package_address.as_ref().to_vec());
        buf.extend(blueprint_id.blueprint_name.as_bytes().to_vec());

        self.allocate_buffer(buf)
    }

    fn get_outer_object(
        &mut self,
        node_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let address = self.api.get_outer_object(&node_id)?;

        self.allocate_buffer(address.to_vec())
    }

    fn actor_emit_event(
        &mut self,
        event_name: Vec<u8>,
        event_payload: Vec<u8>,
        event_flags: EventFlags,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.actor_emit_event(
            String::from_utf8(event_name).map_err(|_| WasmRuntimeError::InvalidString)?,
            event_payload,
            event_flags,
        )?;
        Ok(())
    }

    fn sys_log(
        &mut self,
        level: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.emit_log(
            scrypto_decode::<Level>(&level).map_err(WasmRuntimeError::InvalidLogLevel)?,
            String::from_utf8(message).map_err(|_| WasmRuntimeError::InvalidString)?,
        )?;
        Ok(())
    }

    fn sys_bech32_encode_address(
        &mut self,
        address: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let address =
            scrypto_decode::<GlobalAddress>(&address).map_err(WasmRuntimeError::InvalidAddress)?;
        let encoded = self.api.bech32_encode_address(address)?;
        self.allocate_buffer(encoded.into_bytes())
    }

    fn sys_panic(&mut self, message: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api
            .panic(String::from_utf8(message).map_err(|_| WasmRuntimeError::InvalidString)?)?;
        Ok(())
    }

    fn sys_get_transaction_hash(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let hash = self.api.get_transaction_hash()?;

        self.allocate_buffer(hash.to_vec())
    }

    fn sys_generate_ruid(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let ruid = self.api.generate_ruid()?;

        self.allocate_buffer(ruid.to_vec())
    }

    fn costing_get_execution_cost_unit_limit(
        &mut self,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let execution_cost_unit_limit = self.api.execution_cost_unit_limit()?;

        Ok(execution_cost_unit_limit)
    }

    fn costing_get_execution_cost_unit_price(
        &mut self,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let execution_cost_unit_price = self.api.execution_cost_unit_price()?;

        self.allocate_buffer(
            scrypto_encode(&execution_cost_unit_price)
                .expect("Failed to encode execution_cost_unit_price"),
        )
    }

    fn costing_get_finalization_cost_unit_limit(
        &mut self,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let finalization_cost_unit_limit = self.api.finalization_cost_unit_limit()?;

        Ok(finalization_cost_unit_limit)
    }

    fn costing_get_finalization_cost_unit_price(
        &mut self,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let finalization_cost_unit_price = self.api.finalization_cost_unit_price()?;

        self.allocate_buffer(
            scrypto_encode(&finalization_cost_unit_price)
                .expect("Failed to encode finalization_cost_unit_price"),
        )
    }

    fn costing_get_usd_price(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let usd_price = self.api.usd_price()?;
        self.allocate_buffer(
            scrypto_encode(&usd_price).expect("Failed to encode finalization_cost_unit_price"),
        )
    }

    fn costing_get_tip_percentage(&mut self) -> Result<u32, InvokeError<WasmRuntimeError>> {
        let tip_percentage = self.api.tip_percentage()?;

        Ok(tip_percentage.into())
    }

    fn costing_get_fee_balance(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let fee_balance = self.api.fee_balance()?;

        self.allocate_buffer(scrypto_encode(&fee_balance).expect("Failed to encode fee_balance"))
    }
}
