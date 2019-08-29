use yee_merkle::{BMT, Merge};
use primitives::{H256,blake2_256};
use parity_codec::{Encode, Decode, Input};
use blake2_rfc::blake2b::{Blake2b, blake2b};

pub struct MergeH256;

fn main() {
    impl Merge for MergeH256 {
        type Item = H256;
        fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
            let mut dest = [0u8; 32];
            let mut context = Blake2b::new(32);
            context.update(left.as_bytes());
            context.update(right.as_bytes());
            let hash = context.finalize();
            dest.copy_from_slice(hash.as_bytes());
            dest.into()
        }
    }

    let hash: H256 = blake2_256(&[3].encode()).into();

    type BMTI32 = BMT<H256, MergeH256>;

    let leaves = vec![blake2_256(&[1].encode()).into(), blake2_256(&[3].encode()).into(), blake2_256(&[5].encode()).into(),
                      blake2_256(&[7].encode()).into(), blake2_256(&[11].encode()).into()];

    let indices = vec![0, 4];
    let proof_leaves = vec![blake2_256(&[1].encode()).into(), blake2_256(&[11].encode()).into()];
    let root = BMTI32::build_merkle_root(&leaves);
    println!("thsi is root {:?}", root);

    let proof = BMTI32::build_merkle_proof(&leaves, &indices).unwrap();
    let tree = BMTI32::build_merkle_tree(leaves);


    let v = proof.verify(&root, &proof_leaves);

    println!("verify return --  {:?}", v);
}