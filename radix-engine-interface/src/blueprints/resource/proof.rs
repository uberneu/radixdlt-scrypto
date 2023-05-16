use crate::blueprints::resource::*;
use crate::data::scrypto::model::*;
use crate::data::scrypto::ScryptoCustomTypeKind;
use crate::data::scrypto::ScryptoCustomValueKind;
use crate::*;
use radix_engine_common::data::scrypto::*;
use radix_engine_common::native_addresses::RESOURCE_PACKAGE;
use radix_engine_common::types::*;
use sbor::rust::prelude::*;
use sbor::*;

pub const PROOF_DROP_IDENT: &str = "Proof_drop";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ProofDropInput {
    pub proof: Proof,
}

pub type ProofDropOutput = ();

pub const PROOF_GET_AMOUNT_IDENT: &str = "Proof_get_amount";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ProofGetAmountInput {}

pub type ProofGetAmountOutput = Decimal;

pub const PROOF_GET_RESOURCE_ADDRESS_IDENT: &str = "Proof_get_resource_address";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ProofGetResourceAddressInput {}

pub type ProofGetResourceAddressOutput = ResourceAddress;

pub const PROOF_CLONE_IDENT: &str = "clone";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ProofCloneInput {}

pub type ProofCloneOutput = Proof;

//========
// Stub
//========

// TODO: update schema type

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Proof(pub Own);

#[derive(Debug, PartialEq, Eq, Hash, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
pub struct FungibleProof(pub Proof);

#[derive(Debug, PartialEq, Eq, Hash, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
pub struct NonFungibleProof(pub Proof);

impl From<FungibleProof> for Proof {
    fn from(value: FungibleProof) -> Self {
        value.0
    }
}

impl From<NonFungibleProof> for Proof {
    fn from(value: NonFungibleProof) -> Self {
        value.0
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Proof {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        Own::value_kind()
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Proof {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Proof {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|o| Self(o))
    }
}

impl Describe<ScryptoCustomTypeKind> for Proof {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::well_known(well_known_scrypto_custom_types::OWN_PROOF_ID);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        well_known_scrypto_custom_types::own_proof_type_data()
    }
}

impl Describe<ScryptoCustomTypeKind> for FungibleProof {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::Novel(const_sha1::sha1("FungibleProof".as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Own),
            metadata: TypeMetadata::no_child_names("FungibleProof"),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(
                    RESOURCE_PACKAGE,
                    FUNGIBLE_PROOF_BLUEPRINT.to_string(),
                ),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

impl Describe<ScryptoCustomTypeKind> for NonFungibleProof {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::Novel(const_sha1::sha1("NonFungibleProof".as_bytes()).as_bytes());

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Own),
            metadata: TypeMetadata::no_child_names("NonFungibleProof"),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(
                    RESOURCE_PACKAGE,
                    NON_FUNGIBLE_PROOF_BLUEPRINT.to_string(),
                ),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}
