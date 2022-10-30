use core::fmt::Debug;

use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::{GlobalAddressSubstate, PackageSubstate};
use crate::types::*;
use crate::wasm::*;

pub struct Package;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    InvalidAbi(DecodeError),
    InvalidWasm(PrepareError),
    BlueprintNotFound,
    MethodNotFound(String),
}

impl Package {
    fn new(
        code: Vec<u8>,
        abi: HashMap<String, BlueprintAbi>,
    ) -> Result<PackageSubstate, PrepareError> {
        WasmValidator::default().validate(&code, &abi)?;

        Ok(PackageSubstate {
            code: code,
            blueprint_abis: abi,
        })
    }
}

impl NativeFunctionInvocation for PackagePublishInput {
    type NativeOutput = PackageAddress;

    fn native_function() -> NativeFunction {
        NativeFunction::Package(PackageFunction::Publish)
    }

    fn call_frame_update(&self) -> CallFrameUpdate {
        CallFrameUpdate::empty()
    }

    fn execute<'s, 'a, Y, R>(
        invocation: Self,
        system_api: &mut Y,
    ) -> Result<(PackageAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNativeFunction<'a>
            + Invokable<NativeMethodInvocation>,
        R: FeeReserve,
    {
        let code = system_api.read_blob(&invocation.code.0)?.to_vec();
        let blob = system_api.read_blob(&invocation.abi.0)?;
        let abi = scrypto_decode::<HashMap<String, BlueprintAbi>>(blob).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidAbi(e),
            ))
        })?;
        let package = Package::new(code, abi).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(e),
            ))
        })?;

        let node_id = system_api.create_node(RENode::Package(package))?;
        let package_id: PackageId = node_id.into();

        let global_node_id =
            system_api.create_node(RENode::Global(GlobalAddressSubstate::Package(package_id)))?;

        let package_address: PackageAddress = global_node_id.into();
        Ok((package_address, CallFrameUpdate::empty()))
    }
}
