use std::cmp::max;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Handoff<K>
    where
        K: Clone + Eq + Hash + Copy
{
    id: K,
    tier: u32,
    val: i64,
    below: u32,
    vals: HashMap<K, i64>,
    sck: u32,
    dck: u32,
    slots: HashMap<K, (u32, u32)>,  // id -> (sck, dck)
    tokens: HashMap<(K, K), ((u32, u32), i64)>, // (i,j) -> ((sck, dck), n)
}

impl<K> Handoff<K>
    where
        K: Clone + Eq + Hash + Copy
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
    fn create_slot(&mut self, h: &Self) {
        if self.tier < h.tier && h.vals[&h.id] > 0 && !self.slots.contains_key(&h.id) {
            self.slots.insert(h.id, (h.sck, self.dck));
        }
        self.dck += 1; 
    }

    /// Merge the val of two tiers. This action can only be implemented by nodes in tier 0 (servers). 
    fn merge_vectors(&mut self, h: Handoff<K>) {
        if self.tier != 0 || h.tier != 0  {
            return; 
        }

        // Perform the union of both vals.
        for (id , val) in h.vals.iter() {
            match self.vals.get(id) {
                Some(curr_val) => self.vals.insert(id.clone(), max(val.clone(), curr_val.clone())),
                None => self.vals.insert(id.clone(), val.clone()),
            };
        }
    }

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

    /// Received the token, and now the node must fill the slots. 
    fn fill_slots(&mut self, h: Handoff<K>) {
        let mut total : i64 = 0; 
        for ((_, j), ((src,dck), n)) in h.tokens.iter() {
            if j.clone() == self.id && self.find_slot(j, src, dck) {
                total += n; 
                self.remove_slot(j, src, dck);
            }
        }

        self.increment_self_val(&total);
    }

    /// Discards a slot that can never be filled, since sck is higher than the one marked in the slot. 
    fn discard_slot(&mut self, h: Handoff<K>) {
        match self.slots.get(&h.id) {
            Some(&(src, _)) => {
                if h.sck > src {
                    self.slots.remove(&h.id);
                }
            }, 
            None => return, 
        }
    }

    // Remove a token. 
    fn discard_tokens(&mut self, h: Handoff<K>) {
        if h.slots.contains_key(&self.id) && h.sck > self.slots.get(&self.id).unwrap().0 {
            self.tokens.remove(&(h.id.clone(), self.id.clone())); 
        }
    }

    /// UTILS FUNCTIONS ======================================================== 
    
    /// Checks if a slot exists.
    fn find_slot(&self, id: &K, src: &u32, dck: &u32) -> bool { 
        match self.slots.get(id) {
            Some(v) => {
                return v.clone() == (src.clone(), dck.clone()); 
            },
            None => false
        }
    }

    // Remove slot 
    fn remove_slot(&mut self, id: &K, src: &u32, dck: &u32) -> bool {
        if self.find_slot(id, src, dck) {
            self.slots.remove(id);
            return true;
        }
        false 
    }

    /// Increments the current value in the hashmap. 
    fn increment_self_val(&mut self, total: &i64){
        let curr_val = self.vals.get(&self.id).unwrap().clone();
        self.vals.insert(self.id.clone(), curr_val + total);
    }


}
