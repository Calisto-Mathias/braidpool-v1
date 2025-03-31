#![allow(unused)]
use crate::beads::Bead;
use crate::braid::{self, DagBraid};
use crate::utils::bitcoin::{MerklePathProof, MiningBlockHeader};
use crate::{
    beads::TransactionWithMerklePath,
    utils::{BeadHash, Bytes},
};
use bitcoin::absolute::LockTime;
use bitcoin::absolute::Time;
use bitcoin::hashes::Sha256d;
use bitcoin::hex::FromHex;
use bitcoin::secp256k1::hashes::{sha256d, Hash};
use bitcoin::Block;
use bitcoin::{hashes::HmacSha256, Amount, Txid};
use bitcoin::{
    BlockHash, BlockHeader, BlockTime, BlockVersion, CompactTarget, OutPoint, Script, ScriptBuf,
    Sequence, Transaction, TransactionVersion, TxIn, TxMerkleNode, TxOut, Witness,
};

use crate::braid::Cohort;
use std::dbg;

use crate::beads::DagBead;
use std::collections::HashSet;
#[cfg(test)]
pub mod tests {

    use super::*;
    fn create_block_header(
        version: i32,
        prev_blockhash_bytes: [u8; 32],
        merkle_root_bytes: [u8; 32],
        time: u32,
        bits: u32,
        nonce: u32,
    ) -> BlockHeader {
        BlockHeader {
            version: BlockVersion::from_consensus(version),
            prev_blockhash: BlockHash::from_byte_array(prev_blockhash_bytes),
            merkle_root: TxMerkleNode::from_byte_array(merkle_root_bytes),
            time: BlockTime::from_u32(time),
            bits: CompactTarget::from_consensus(bits),
            nonce,
        }
    }

    fn create_dummy_transaction(bytes: [u8; 32]) -> Transaction {
        let witness_item_1 =
            Vec::from_hex("03d2e15674941bad4a996372cb87e1856d3652606d98562fe39c5e9e7e413f2105")
                .unwrap();
        let witness_item_2 = Vec::from_hex("000000").unwrap();
        let witness_entries = [witness_item_1, witness_item_2];
        Transaction {
            version: TransactionVersion::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_byte_array(bytes),
                    vout: 2,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence(0xFFFFFFFF),
                witness: Witness::from_slice(&witness_entries),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(10_000_000).unwrap(),
                script_pubkey: ScriptBuf::new(),
            }],
        }
    }
    fn create_dummy_merkle_path_proof(bytes: [u8; 32]) -> MerklePathProof {
        MerklePathProof {
            transaction_hash: Txid::from_byte_array(bytes),
            merkle_path: vec![],
        }
    }
    fn create_parents_set(parents_vec: Vec<(BeadHash, Time)>) -> HashSet<(BeadHash, Time)> {
        parents_vec.into_iter().collect()
    }
    fn create_test_bead(
        block_version: i32,
        prev_blockhash_bytes: [u8; 32],
        merkle_root_bytes: [u8; 32],
        timestamp: u32,
        difficulty_bits: u32,
        nonce: u32,
        bead_hash_bytes: [u8; 32],
        coinbase_tx_bytes: [u8; 32],
        payout_tx_bytes: [u8; 32],
        lesser_difficulty_target: u32,
        parent_hashes: Vec<(BeadHash, Time)>,
        transactions_bytes: Vec<[u8; 32]>,
        bits: u32,
        bead_bytes: [u8; 32],
    ) -> Bead {
        let target: CompactTarget = CompactTarget::from_consensus(lesser_difficulty_target);
        let bytes: [u8; 32] = bead_bytes;
        let beadhash: BeadHash = BlockHash::from_byte_array(bytes);
        let blockheader: BlockHeader = create_block_header(
            block_version,
            prev_blockhash_bytes,
            merkle_root_bytes,
            timestamp,
            bits,
            nonce,
        );
        let coinbase_transaction: TransactionWithMerklePath = (
            create_dummy_transaction(coinbase_tx_bytes),
            create_dummy_merkle_path_proof(coinbase_tx_bytes),
        );

        let parents: HashSet<(BeadHash, Time)> = create_parents_set(parent_hashes);

        let mut transactions_of_bead: Vec<Transaction> = Vec::new();

        for transaction in transactions_bytes.iter() {
            transactions_of_bead.push(create_dummy_transaction(*transaction));
        }
        let payout_transaction: TransactionWithMerklePath = (
            create_dummy_transaction(payout_tx_bytes),
            create_dummy_merkle_path_proof(payout_tx_bytes),
        );
        return Bead {
            block_header: blockheader,
            bead_hash: beadhash,
            coinbase_transaction: coinbase_transaction,
            payout_update_transaction: payout_transaction,
            lesser_difficulty_target: target,
            parents: parents,
            transactions: transactions_of_bead,
        };
    }
    fn create_test_dag_bead(
        version: i32,
        prev_hash: [u8; 32],
        merkle_root: [u8; 32],
        timestamp: u32,
        bits: u32,
        nonce: u32,
        extra_nonce: [u8; 32],
        commitment: [u8; 32],
        signature: [u8; 32],
        weight: u32,
        parents: Vec<(BeadHash, Time)>,
        proof: Vec<[u8; 32]>,
        difficulty: u32,
        reserved: [u8; 32],
        observed_time: u32,
    ) -> DagBead {
        let bead: Bead = create_test_bead(
            version,
            prev_hash,
            merkle_root,
            timestamp,
            bits,
            nonce,
            extra_nonce,
            commitment,
            signature,
            weight,
            parents,
            proof,
            difficulty,
            reserved,
        );

        DagBead {
            bead_data: bead,
            observed_time_at_node: Time::from_consensus(observed_time).unwrap(),
        }
    }

    #[test]
    fn test_valid_bead() {
        let test_dag_bead: DagBead = create_test_dag_bead(
            2,
            [0x00; 32],
            [
                0xf3, 0xb8, 0x76, 0x2e, 0x7c, 0x1b, 0xd6, 0x47, 0xf1, 0xf6, 0x9d, 0x2a, 0x7f, 0x9c,
                0x85, 0xf0, 0xb2, 0x5e, 0x64, 0x69, 0xf1, 0x07, 0xd2, 0x31, 0xdf, 0xf4, 0x5c, 0x47,
                0x1f, 0x88, 0x94, 0x58,
            ],
            1653195600,
            486604799,
            0,
            [0x00; 32],
            [0xbb; 32],
            [0xbb; 32],
            4040404,
            vec![
                (
                    BeadHash::from_byte_array([0x01; 32]),
                    Time::from_consensus(1690000000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x02; 32]),
                    Time::from_consensus(1690001000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x03; 32]),
                    Time::from_consensus(1690002000).expect("invalid time value"),
                ),
            ], // parents
            vec![[0x11; 32], [0x22; 32], [0x33; 32], [0x44; 32]],
            436864982,
            [0x00; 32],
            1653195600,
        );
        let x = &test_dag_bead.bead_data;
        assert_eq!(test_dag_bead.is_valid_bead(), true);
    }
    #[test]
    fn test_bead_contain() {
        let bytes: [u8; 32] = [0; 32];
        let mut genesis_beads: HashSet<BeadHash> = HashSet::new();
        genesis_beads.insert(BlockHash::from_byte_array(bytes));
        let mut test_braid = DagBraid::new(genesis_beads);
        let dummy_dag_bead = create_test_dag_bead(
            2,
            [0x00; 32],
            [
                0xf3, 0xb8, 0x76, 0x2e, 0x7c, 0x1b, 0xd6, 0x47, 0xf1, 0xf6, 0x9d, 0x2a, 0x7f, 0x9c,
                0x85, 0xf0, 0xb2, 0x5e, 0x64, 0x69, 0xf1, 0x07, 0xd2, 0x31, 0xdf, 0xf4, 0x5c, 0x47,
                0x1f, 0x88, 0x94, 0x58,
            ],
            1653195600,
            486604799,
            0,
            [0x00; 32],
            [0xbb; 32],
            [0xbb; 32],
            4040404,
            vec![
                (
                    BeadHash::from_byte_array([0x01; 32]),
                    Time::from_consensus(1690000000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x02; 32]),
                    Time::from_consensus(1690001000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x03; 32]),
                    Time::from_consensus(1690002000).expect("invalid time value"),
                ),
            ], // parents
            vec![[0x11; 32], [0x22; 32], [0x33; 32], [0x44; 32]],
            436864982,
            [0x00; 32],
            1653195600,
        );
        let reference_dummy_bead = dummy_dag_bead.clone();
        test_braid.add_bead(dummy_dag_bead);
        assert_eq!(
            test_braid.contains_bead(reference_dummy_bead.bead_data.bead_hash),
            true
        );
    }
    #[test]
    fn test_remove_parents() {
        let test_dag_bead = create_test_dag_bead(
            2,
            [0x00; 32],
            [
                0xf3, 0xb8, 0x76, 0x2e, 0x7c, 0x1b, 0xd6, 0x47, 0xf1, 0xf6, 0x9d, 0x2a, 0x7f, 0x9c,
                0x85, 0xf0, 0xb2, 0x5e, 0x64, 0x69, 0xf1, 0x07, 0xd2, 0x31, 0xdf, 0xf4, 0x5c, 0x47,
                0x1f, 0x88, 0x94, 0x58,
            ],
            1653195600,
            486604799,
            0,
            [0x00; 32],
            [0xbb; 32],
            [0xbb; 32],
            4040404,
            vec![
                (
                    BeadHash::from_byte_array([0x01; 32]),
                    Time::from_consensus(1690000000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x02; 32]),
                    Time::from_consensus(1690001000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x03; 32]),
                    Time::from_consensus(1690002000).expect("invalid time value"),
                ),
            ], // parents
            vec![[0x11; 32], [0x22; 32], [0x33; 32], [0x44; 32]],
            436864982,
            [0x00; 32],
            1653195600,
        );
        let bytes: [u8; 32] = [0; 32];
        let mut genesis_beads: HashSet<BeadHash> = HashSet::new();
        genesis_beads.insert(BlockHash::from_byte_array(bytes));
        let mut test_braid = DagBraid::new(genesis_beads);

        let referenced_val = test_dag_bead.clone();
        test_braid.add_bead(test_dag_bead);
        test_braid.remove_parent_beads_from_tips(&referenced_val);

        let parents = referenced_val.bead_data.parents;

        for parent in parents {
            assert_eq!(test_braid.tips.contains(&parent.0), false);
        }
    }
    #[test]
    fn test_orphan_bead() {
        let test_dag_bead = create_test_dag_bead(
            2,
            [0x00; 32],
            [
                0xf3, 0xb8, 0x76, 0x2e, 0x7c, 0x1b, 0xd6, 0x47, 0xf1, 0xf6, 0x9d, 0x2a, 0x7f, 0x9c,
                0x85, 0xf0, 0xb2, 0x5e, 0x64, 0x69, 0xf1, 0x07, 0xd2, 0x31, 0xdf, 0xf4, 0x5c, 0x47,
                0x1f, 0x88, 0x94, 0x58,
            ],
            1653195600,
            486604799,
            0,
            [0x00; 32],
            [0xbb; 32],
            [0xbb; 32],
            4040404,
            vec![
                (
                    BeadHash::from_byte_array([0x01; 32]),
                    Time::from_consensus(1690000000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x02; 32]),
                    Time::from_consensus(1690001000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x03; 32]),
                    Time::from_consensus(1690002000).expect("invalid time value"),
                ),
            ], // parents
            vec![[0x11; 32], [0x22; 32], [0x33; 32], [0x44; 32]],
            436864982,
            [0x00; 32],
            1653195600,
        );
        let bytes: [u8; 32] = [0; 32];
        let mut genesis_beads: HashSet<BeadHash> = HashSet::new();
        genesis_beads.insert(BlockHash::from_byte_array(bytes));
        let mut test_braid = DagBraid::new(genesis_beads);
        let referenced_bead = test_dag_bead.clone();
        test_braid.add_bead(test_dag_bead);

        assert_eq!(test_braid.is_bead_orphaned(&referenced_bead), false);
    }
    #[test]
    fn test_update_orphans() {
        let test_dag_bead_1 = create_test_dag_bead(
            2,
            [0x00; 32],
            [
                0xf3, 0xb8, 0x76, 0x2e, 0x7c, 0x1b, 0xd6, 0x47, 0xf1, 0xf6, 0x9d, 0x2a, 0x7f, 0x9c,
                0x85, 0xf0, 0xb2, 0x5e, 0x64, 0x69, 0xf1, 0x07, 0xd2, 0x31, 0xdf, 0xf4, 0x5c, 0x47,
                0x1f, 0x88, 0x94, 0x58,
            ],
            1653195600,
            486604799,
            0,
            [0x00; 32],
            [0xbb; 32],
            [0xbb; 32],
            4040404,
            vec![
                (
                    BeadHash::from_byte_array([0x01; 32]),
                    Time::from_consensus(1690000000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x02; 32]),
                    Time::from_consensus(1690001000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x03; 32]),
                    Time::from_consensus(1690002000).expect("invalid time value"),
                ),
            ], 
            vec![[0x11; 32], [0x22; 32], [0x33; 32], [0x44; 32]],
            436864982,
            [0x00; 32],
            1653195600,
        );
        let test_dag_bead_2 = create_test_dag_bead(
            2,
            [0x00; 32],
            [
                0xf3, 0xb8, 0x76, 0x2e, 0x7c, 0x1b, 0xd6, 0x47, 0xf1, 0xf6, 0x9d, 0x2a, 0x7f, 0x9c,
                0x85, 0xf0, 0xb2, 0x5e, 0x64, 0x69, 0xf1, 0x07, 0xd2, 0x31, 0xdf, 0xf4, 0x5c, 0x47,
                0x1f, 0x88, 0x94, 0x58,
            ],
            1653195600,
            486604799,
            0,
            [0x00; 32],
            [0xbb; 32],
            [0xbb; 32],
            4040404,
            vec![
                (
                    BeadHash::from_byte_array([0x01; 32]),
                    Time::from_consensus(1690000000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x02; 32]),
                    Time::from_consensus(1690001000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x03; 32]),
                    Time::from_consensus(1690002000).expect("invalid time value"),
                ),
            ], 
            vec![[0x11; 32], [0x22; 32], [0x33; 32], [0x44; 32]],
            436864982,
            [0x00; 32],
            1653195600,
        );
        let mut genesis_beads: HashSet<BeadHash> = HashSet::new();
        let child_bead_1 = create_test_dag_bead(
            2,
            [0x00; 32],
            [
                0xf3, 0xb8, 0x76, 0x2e, 0x7c, 0x1b, 0xd6, 0x47, 0xf1, 0xf6, 0x9d, 0x2a, 0x7f, 0x9c,
                0x85, 0xf0, 0xb2, 0x5e, 0x64, 0x69, 0xf1, 0x07, 0xd2, 0x31, 0xdf, 0xf4, 0x5c, 0x47,
                0x1f, 0x88, 0x94, 0x58,
            ],
            1653195600,
            486604799,
            0,
            [0x00; 32],
            [0xbb; 32],
            [0xbb; 32],
            4040404,
            vec![
                (
                    BeadHash::from_byte_array([0x01; 32]),
                    Time::from_consensus(1690000000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x02; 32]),
                    Time::from_consensus(1690001000).expect("invalid time value"),
                ),
                (
                    BeadHash::from_byte_array([0x03; 32]),
                    Time::from_consensus(1690002000).expect("invalid time value"),
                ),
            ], 
            vec![[0x11; 32], [0x22; 32], [0x33; 32], [0x44; 32]],
            436864982,
            [0x00; 32],
            1653195600,
        );

        genesis_beads.insert(test_dag_bead_1.bead_data.bead_hash);
        genesis_beads.insert(test_dag_bead_2.bead_data.bead_hash);

        let mut test_braid = DagBraid::new(genesis_beads);

        test_braid.add_bead(test_dag_bead_1);
        test_braid.add_bead(test_dag_bead_2);
        test_braid.add_bead(child_bead_1);

        assert_eq!(test_braid.update_orphan_bead_set(), 2);
    }
    // fn test_bead_cohort_1() {
    //     let bytes: [u8; 32] = [0; 32];
    //     let mut genesis_beads: HashSet<BeadHash> = HashSet::new();
    //     genesis_beads.insert(BlockHash::from_byte_array(bytes));
    //     let test_braid = DagBraid::new(genesis_beads);

    //     let expected_cohort_1: Vec<Cohort> = Vec::new();
    //     let computed_cohort_1: Vec<Cohort> = test_braid.generate_tip_cohorts();
    //     assert_eq!(expected_cohort_1.len(), computed_cohort_1.len());
    //     for x in 0..expected_cohort_1.len() {
    //         let cohort_expected = &expected_cohort_1[x].0.into_iter().collect();
    //         let cohort_computed = &computed_cohort_1[x].0.into_iter().collect();
    //         assert_eq!(cohort_computed, cohort_expected);
    //     }
    // }
}
