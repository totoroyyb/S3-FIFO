use s3fifo::prelude::*;
// check lib/lib.rs for prelude content

// Some quick demo

fn main() {
    // it's same as 
    // `let mut cache: S3FIFO<usize, usize> = S3FIFO::new_with_default_ratio(100);`
    let mut cache: S3FIFO<usize, usize> = S3FIFO::new(100, 0.1);

    cache.put(0, 100);
    cache.put(1, 101);

    assert_eq!(cache.get(&0), Some(&100));
    assert_eq!(cache.get(&1), Some(&101));

    // Get non-exist element
    assert!(cache.get(&100).is_none());

    cache.put(10, 101);
    cache.put(10, 100);
    // Same key will update
    assert_eq!(cache.get(&10), Some(&100));

    println!("Finished!");
}
