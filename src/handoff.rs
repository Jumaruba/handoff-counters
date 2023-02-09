use std::cmp::max;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Handoff<K>
where
    K: Clone + Eq + Hash + Copy,
{
    id: K,
    tier: u32,
    val: i64,
    below: i64,
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
    pub fn new(id: K, tier: u32, sck: Option<u32>, dck: Option<u32>) -> Self {
        Self {
            id: id.clone(),
            tier,
            val: 0,
            below: 0,
            vals: HashMap::from([(id.clone(), 0)]),
            sck: sck.unwrap_or(0),
            dck: dck.unwrap_or(0),
            slots: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    pub fn fetch(&self) -> i64 {
        self.val
    }

    pub fn inc(&mut self) {
        self.val += 1;
        self.vals.insert(self.id.clone(), self.vals[&self.id] + 1);
    }

    pub fn merge(&mut self, h: &Self){
        self.fill_slots(h);     
        self.discard_slot(h);
        self.create_slot(h); 
        self.merge_vectors(h);
        self.aggregate(h);
        self.discard_tokens(h);
        self.create_token(h);
        self.cache_tokens(h);

    }

    /// This function creates a slot for the handoff that will receive information.
    pub fn create_slot(&mut self, h: &Self) {
        if self.tier < h.tier && h.vals[&h.id] > 0 && !self.slots.contains_key(&h.id) {
            self.slots.insert(h.id, (h.sck, self.dck));
            self.dck += 1;
        }
    }

    /// Merge the val of two tiers. This action can only be implemented by nodes in tier 0 (servers).
    fn merge_vectors(&mut self, h: &Self) {
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
    pub fn create_token(&mut self, h: &Self) {
        let is_waiting_token: bool = h.slots.contains_key(&self.id);
        let is_slot_valid: bool = h.slots.get(&self.id).unwrap().0.clone() == self.sck;
        if is_waiting_token && is_slot_valid {
            self.tokens.insert(
                (self.id.clone(), h.id.clone()),
                (
                    h.slots.get(&self.id).unwrap().clone(),
                    self.vals.get(&self.id).unwrap().clone(),
                ),
            );
            self.vals.insert(self.id.clone(), 0); // Reset the current value of the node.
            self.sck += 1;
        }
    }

    /// Checked if there are tokens that are able to fill slots.
    pub fn fill_slots(&mut self, h: &Self) {
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
    fn discard_slot(&mut self, h: &Self) {
        if let Some(&(src, _)) = self.slots.get(&h.id) {
            if h.sck > src {
                self.slots.remove(&h.id);
            }
        }
    }

    // Remove tokens out of date.
    // Code from https://github.com/pssalmeida/handoff_counter-rs/blob/master/src/handoff_counter.rs 
    pub fn discard_tokens(&mut self, h: &Self) {
        let token: HashMap<(K,K), ((u32,u32), i64)> = self.tokens.drain().filter(|&((src, dst), ((_, dck), _))| {
            !(dst == h.id && match h.slots.get(&src) {
                Some(&(_, d)) =>  d > dck, 
                None => h.dck > dck
            })
        }).collect();
        self.tokens = token;
    }

    fn cache_tokens(&mut self, h: &Self) {
        if self.tier < h.tier {
            for (&(src, dst), &((sck, dck), n)) in h.tokens.iter() {
                if src == h.id && dst != self.id {
                    let p = &(src, dst);
                    let val = if self.tokens.contains_key(p) && sck >= self.tokens[p].0 .0 {
                        ((sck, dck), n)
                    } else {
                        self.tokens[p]
                    };
                    self.tokens.insert(*p, val);
                }
            }
        }
    }

    fn aggregate(&mut self, h: &Self) {
        self.update_below(h);
        self.update_val(h);
    }

    fn update_below(&mut self, h: &Self) {
        if self.tier == h.tier {
            self.below = max(self.below, h.below);
        } else if self.tier > h.tier {
            self.below = max(self.below, h.val);
        }
    }

    fn update_val(&mut self, h: &Self) {
        if self.tier == 0 {
            self.val = self.vals.values().sum();
        } else if self.tier == h.tier {
            self.val = max(max(self.val, h.val), self.below + self.val + h.val);
        } else {
            self.val = max(self.val, self.below + self.vals[&self.id]);
        }
    }

    // UTILS FUNCTIONS
    pub fn get_sck(&self) -> u32 {
        self.sck.clone()
    }

    pub fn get_dck(&self) -> u32 {
        self.dck.clone()
    }

    pub fn get_slots(&self) -> HashMap<K, (u32, u32)> {
        self.slots.clone()
    }

    pub fn get_tokens(&self) -> HashMap<(K,K), ((u32,u32), i64)> {
        self.tokens.clone()
    }

    pub fn get_self_vals(&self) -> i64 {
        self.vals[&self.id]
    }

}
