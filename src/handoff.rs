use std::cmp::max;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Handoff<K>
where
    K: Clone + Eq + Hash + Copy,
{
    id: K,
    tier: u32,
    val: i64,
    below: u32,
    vals: HashMap<K, i64>,
    sck: u32,
    dck: u32,
    slots: HashMap<K, (u32, u32)>,              // id -> (sck, dck)
    tokens: HashMap<(K, K), ((u32, u32), i64)>, // (i,j) -> ((sck, dck), n)
}

impl<K> Handoff<K>
where
    K: Clone + Eq + Hash + Copy,
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
        if self.tier != 0 || h.tier != 0 {
            return;
        }
        // Perform the union of both vals.
        for (id, val) in h.vals.iter() {
            match self.vals.get(id) {
                Some(curr_val) => self
                    .vals
                    .insert(id.clone(), max(val.clone(), curr_val.clone())),
                None => self.vals.insert(id.clone(), val.clone()),
            };
        }
    }

    /// Creates a token to send to a node that has requested it by creating a slot.
    fn create_token(&mut self, h: Handoff<K>) {
        let is_waiting_token: bool = h.slots.contains_key(&self.id);
        let is_slot_valid: bool = h.slots.get(&self.id).unwrap().0.clone() == self.sck;
        if is_waiting_token && is_slot_valid {
            self.tokens.insert(
                (self.id.clone(), h.id.clone()),
                (
                    self.slots.get(&self.id).unwrap().clone(),
                    self.vals.get(&self.id).unwrap().clone(),
                ),
            );
            self.vals.insert(self.id.clone(), 0); // Reset the current value of the node.
            self.sck += 1;
        }
    }

    /// Checked if there are tokens that are able to fill slots.
    fn fill_slots(&mut self, h: Handoff<K>) {
        let val = self.vals.get_mut(&self.id).unwrap();
        for (&(_, j), &((src, dck), n)) in h.tokens.iter() {
            // This node is the destination.
            if j == self.id {
                if let Some(&v) = self.slots.get(&h.id) {
                    // The slot (src, dck) matches the token (src, dck). 
                    if v == (src, dck) {
                        *val += n;
                        self.slots.remove(&h.id);
                    }
                }
            }
        }
    }

    /// Discards a slot that can never be filled, since sck is higher than the one marked in the slot.
    fn discard_slot(&mut self, h: Handoff<K>) {
        if let Some(&(src, _)) =  self.slots.get(&h.id) {
            if h.sck > src {
                self.slots.remove(&h.id);
            }
        }
    }

    // Remove a token.
    fn discard_tokens(&mut self, h: &Self) {
        if h.slots.contains_key(&self.id) && h.sck > self.slots[&self.id].0 {
            self.tokens.remove(&(h.id, self.id));
        }
    }

    fn cache_tokens(&mut self, h: &Self) { 
        if self.tier < h.tier {
            for (&(src, dst), &((sck, dck), n)) in h.tokens.iter() {
                if src == h.id && dst != self.id {
                    let p = &(src, dst);
                    let val = if self.tokens.contains_key(p) && sck >= self.tokens[p].0.0 {((sck,dck),n)} else {self.tokens[p]};
                    self.tokens.insert(*p, val);
                }
            }
        }
    }


}
