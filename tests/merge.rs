use HandoffCounter::Handoff;

fn setup() -> (Handoff<char>, Handoff<char>) {
    let mut ni: Handoff<char> = Handoff::new('i', 1, Some(2), Some(0));
    let mut nj: Handoff<char> = Handoff::new('j', 0, Some(0), Some(5));

    for _ in 0..9 {
        ni.inc();
    }

    for _ in 0..1021 {
        nj.inc();
    }
    return (ni, nj);
}

fn setup2() -> (Handoff<char>, Handoff<char>) {
    let mut nk: Handoff<char> = Handoff::new('k', 2, Some(1), Some(0));
    let mut ni: Handoff<char> = Handoff::new('i', 1, Some(3), Some(0));

    ni.val = 1030;
    ni.below = 1030;

    for _ in 0..4 {
        nk.inc();
    }
    return (ni, nk);
}

#[test]
fn test1(){
    let (mut ni, mut nj) = setup();
    nj.merge(&ni);  // Create the slots. 
    println!("CREATE SLOT = {:?}", nj); 
    ni.merge(&nj); //Crete token.
    println!("CREATE TOKEN = {:?}", ni); 
    nj.merge(&ni);  // Fill the slot. 
    println!("FILL SLOT = {:?}", nj); 
    ni.merge(&nj); 
    println!("ACK, DISCARD TOKEN = {:?}", ni); 
}

#[test]
fn test2(){
    let (mut ni, mut nk) = setup2();
    ni.merge(&nk);  // Create the slots. 
    println!("CREATE SLOT = {:?}", ni); 
    nk.merge(&ni); //Crete token.
    println!("CREATE TOKEN = {:?}", nk); 
    ni.merge(&nk);  // Fill the slot. 
    println!("FILL SLOT = {:?}", ni); 
    nk.merge(&ni); 
    println!("ACK, DISCARD TOKEN = {:?}", nk); 
}