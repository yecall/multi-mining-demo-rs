use yee_merkle::{BMT,Merge};
struct MergeI32 {}

fn main(){
    impl Merge for MergeI32 {
        type Item = i32;
        fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
            right.wrapping_sub(*left)
        }
    }

    type BMTI32 = BMT<i32, MergeI32>;

    let leaves = vec![1, 3, 5, 7, 11];
    let indices = vec![0, 4];
    let proof_leaves = vec![1, 11];
    let root = BMTI32::build_merkle_root(&leaves);
    println!("thsi is root {:?}",root);

    let proof = BMTI32::build_merkle_proof(&leaves, &indices).unwrap();


    let tree = BMTI32::build_merkle_tree(leaves);


    let v = proof.verify(&root,&proof_leaves);

    println!("verify return --  {:?}",v);

}