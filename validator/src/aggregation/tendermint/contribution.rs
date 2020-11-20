use std::collections::BTreeMap;

use beserial::{Deserialize, Serialize};
use nimiq_block_albatross::MultiSignature;
use nimiq_bls::{AggregateSignature, SecretKey};
use nimiq_collections::bitset::BitSet;
use nimiq_hash::{Blake2sHash, Blake2sHasher, Hasher, SerializeContent};

use nimiq_handel::contribution::{AggregatableContribution, ContributionError};

use super::utils::TendermintVote;

#[derive(Serialize, Deserialize, std::fmt::Debug, Clone)]
pub struct TendermintContribution {
    #[beserial(len_type(u16))]
    pub(crate) contributions: BTreeMap<Option<Blake2sHash>, MultiSignature>,
}

impl TendermintContribution {
    pub(crate) fn from_vote(
        vote: TendermintVote,
        secret_key: &SecretKey,
        validator_id: u16,
    ) -> Self {
        // create the to be signed hash
        let mut hasher = Blake2sHasher::new();
        vote.serialize_content(&mut hasher);
        let message_hash = hasher.finish();

        // sign the hash
        let signature = AggregateSignature::from_signatures(&[secret_key.sign_hash(message_hash)]);

        let mut signers = BitSet::new();
        signers.insert(validator_id as usize);

        // create a MultiSignature with the single contributor `validator_id`
        let multi_signature = MultiSignature::new(signature, signers);

        let mut contributions = BTreeMap::new();
        contributions.insert(vote.proposal_hash, multi_signature);
        Self { contributions }
    }
}

impl AggregatableContribution for TendermintContribution {
    /// Combines two TendermintContributions Every different proposal is represented as its own multisignature.
    /// When combining non existing keys must be inserted while the mutlisignatures of existing keys are combined.
    fn combine(&mut self, other_contribution: &Self) -> Result<(), ContributionError> {
        // TODO: If we don't need the overlapping IDs for the error, we can use `intersection_size`
        let overlap = &self.contributors() & &other_contribution.contributors();

        if overlap.is_empty() {
            // iterate over all different values the other_contributions HashMap contains.
            other_contribution.contributions.iter().for_each(|(hash, other_sig)| {
                // for every entry there
                self.contributions
                    // get the entry here
                    .entry(hash.clone())
                    // and update it
                    .and_modify(|sig|
                        // by combining both Multisigs
                        sig
                            .combine(other_sig)
                            .expect("Non Overlapping TendermintContribution encountered overlapping MultiSignatures"),
                    )
                    // if that entry does not exist, creatte it with the MultiSignature from other_contribution.
                    .or_insert(other_sig.clone());
            });
            Ok(())
        } else {
            Err(ContributionError::Overlapping(overlap))
        }
    }

    fn contributors(&self) -> BitSet {
        self.contributions
            .iter()
            .fold(BitSet::new(), |aggregated_set, multi_sig| {
                aggregated_set & multi_sig.1.contributors()
            })
    }
}
