
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub trait Merge {
    type Item;
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item;
}

pub struct MerkleTree<T, M> {
    nodes: Vec<T>,
    merge: PhantomData<M>,
}


pub struct MerkleProof<T, M> {
    indices: Vec<u32>,
    lemmas: Vec<T>,
    merge: PhantomData<M>,
}



#[derive(Default)]
pub struct MerkleData<T, M> {
    data_type: PhantomData<T>,
    merge: PhantomData<M>,
}

//Todo