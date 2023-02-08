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

    /// This function creates a slot for the handoff that will receive information. 
    fn create_slot(&mut self, h: Handoff<K>) {
        if self.tier < h.tier && h.val > 0 && !self.slots.contains_key(&h.id) {
            self.slots.insert(h.id.clone(), (h.sck, self.dck));
        }
        self.dck += 1; 
    }

    /*fn discard_slot(&mut self, h: Handoff<K>) {
        if (&self.slots.contains_key(&h.id) && h.sck > self.slots.get(&h.id)[0]) {
            
        }
    }*/

    /// Creates a token to send to a node that has requested it by creating a slot. 
    fn create_token(&mut self, h: Handoff<K>) {
        let is_waiting_token : bool = h.slots.contains_key(&self.id);
        let is_slot_valid: bool = h.slots.get(&self.id).unwrap().0.clone() == self.sck; 
        if  is_waiting_token &&  is_slot_valid {
            self.tokens.insert((self.id.clone(), h.id.clone()), (self.slots.get(&self.id).unwrap().clone(), self.vals.get(&self.id).unwrap().clone()));
            self.vals.insert(self.id.clone(), 0);   // Reset the current value of the node. 
            self.sck += 1;  
        }
    }

    
    

}
