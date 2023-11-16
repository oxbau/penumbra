use anyhow::anyhow;
use decaf377_frost::round1;
use ed25519_consensus::{Signature, VerificationKey};
use penumbra_proto::{penumbra::custody::threshold::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_transaction::plan::TransactionPlan;

/// Represents the message sent by the coordinator at the start of the signing process.
///
/// This is nominally "round 1", even though it's the only message the coordinator ever sends.
#[derive(Debug, Clone)]
pub struct CoordinatorRound1 {
    plan: TransactionPlan,
}

impl CoordinatorRound1 {
    /// Construct a new round1 package given a transaction plan.
    pub fn new(plan: TransactionPlan) -> Self {
        Self { plan }
    }
}

impl From<CoordinatorRound1> for pb::CoordinatorRound1 {
    fn from(value: CoordinatorRound1) -> Self {
        Self {
            plan: Some(value.plan.into()),
        }
    }
}

impl TryFrom<pb::CoordinatorRound1> for CoordinatorRound1 {
    type Error = anyhow::Error;

    fn try_from(value: pb::CoordinatorRound1) -> Result<Self, Self::Error> {
        Ok(Self {
            plan: value.plan.ok_or(anyhow!("missing plan"))?.try_into()?,
        })
    }
}

impl TypeUrl for CoordinatorRound1 {
    const TYPE_URL: &'static str = "/penumbra.custody.threshold.v1alpha1.CoordinatorRound1";
}

impl DomainType for CoordinatorRound1 {
    type Proto = pb::CoordinatorRound1;
}

/// The message sent by the followers in round1 of signing.
#[derive(Debug, Clone)]
pub struct FollowerRound1 {
    /// A commitment for each spend we need to authorize.
    commitments: Vec<round1::SigningCommitments>,
    /// A verification key identifying who the sender is.
    pk: VerificationKey,
    /// The signature over the protobuf encoding of the commitments.
    sig: Signature,
}

impl From<FollowerRound1> for pb::FollowerRound1 {
    fn from(value: FollowerRound1) -> Self {
        Self {
            inner: Some(pb::follower_round1::Inner {
                commitments: value.commitments.into_iter().map(|x| x.into()).collect(),
            }),
            pk: Some(pb::VerificationKey {
                inner: value.pk.to_bytes().to_vec(),
            }),
            sig: Some(pb::Signature {
                inner: value.sig.to_bytes().to_vec(),
            }),
        }
    }
}

impl TryFrom<pb::FollowerRound1> for FollowerRound1 {
    type Error = anyhow::Error;

    fn try_from(value: pb::FollowerRound1) -> Result<Self, Self::Error> {
        Ok(Self {
            commitments: value
                .inner
                .ok_or(anyhow!("missing inner"))?
                .commitments
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            pk: value
                .pk
                .ok_or(anyhow!("missing pk"))?
                .inner
                .as_slice()
                .try_into()?,
            sig: value
                .sig
                .ok_or(anyhow!("missing sig"))?
                .inner
                .as_slice()
                .try_into()?,
        })
    }
}

impl TypeUrl for FollowerRound1 {
    const TYPE_URL: &'static str = "/penumbra.custody.threshold.v1alpha1.FollowerRound1";
}

impl DomainType for FollowerRound1 {
    type Proto = pb::FollowerRound1;
}
