use std::collections::HashMap;
use std::hash::Hash;

pub struct Handoff<K>
    where
        K: Clone + Eq + Hash
{
    id: K,
    tier: u32,
    val: i64,
    below: u32,
    vals: HashMap<K, i64>,
    sck: u32,
    dck: u32,
    slots: HashMap<K, (u32, u32)>,
    tokens: HashMap<(K, K), ((u32, u32), i64)>,
}

impl<K> Handoff<K>
    where
        K: Clone + Eq + Hash
{
    pub fn new(id: &K, tier: u32) -> Self {
        Self {
            id: id.clone(),
            tier,
            val: 0,
            below: 0,
            vals: HashMap::from([(id.clone(), 0)]),
            sck: 0,
            dck: 0,
            slots: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    pub fn fetch(&self) -> i64 {
        self.val
    }

    pub fn inc(&mut self) {
        self.val += 1;
        let curr_val = self.vals.get(&self.id).unwrap().clone();
        self.vals.insert(self.id.clone(), curr_val);
    }



    
    

}
