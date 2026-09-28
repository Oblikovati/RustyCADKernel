// =============================================================================
// BVH_BuildQueue Thread Safety Tests
// =============================================================================

#[test]
fn BVH_BuildQueueTest__ConcurrentEnqueue() {
    let mut aQueue = BvhBuildQueue::new();

    let aThreadCount: i32 = 4;
    let aItemsPerThread: i32 = 100;

    std::thread::scope(|scope| {
        let aThreads: Vec<_> = (0..aThreadCount)
            .map(|t| {
                let aQueue = &aQueue;
                scope.spawn(move || {
                    for i in 0..aItemsPerThread {
                        aQueue.enqueue(t * aItemsPerThread + i);
                    }
                })
            })
            .collect();

        for aThread in aThreads {
            aThread.join().unwrap();
        }
    });

    assert_eq!(aQueue.size(), aThreadCount * aItemsPerThread);
}

#[test]
fn BVH_BuildQueueTest__ConcurrentFetch() {
    let mut aQueue = BvhBuildQueue::new();

    let aItemCount: i32 = 400;
    for i in 0..aItemCount {
        aQueue.enqueue(i);
    }

    let aThreadCount: usize = 4;
    let mut aFetchedCounts: Vec<i32> = vec![0; aThreadCount];

    std::thread::scope(|scope| {
        let aThreads: Vec<_> = aFetchedCounts
            .iter_mut()
            .map(|aFetchedCount| {
                let aQueue = &aQueue;
                scope.spawn(move || {
                    let mut wasBusy = false;
                    loop {
                        let aItem = aQueue.fetch(&mut wasBusy);
                        if aItem == -1 {
                            break;
                        }
                        *aFetchedCount += 1;
                    }
                })
            })
            .collect();

        for aThread in aThreads {
            aThread.join().unwrap();
        }
    });

    let mut aTotalFetched = 0;
    for aCount in &aFetchedCounts {
        aTotalFetched += aCount;
    }

    assert_eq!(aTotalFetched, aItemCount);
    assert_eq!(aQueue.size(), 0);
}

#[test]
fn BVH_BuildQueueTest__ConcurrentEnqueueAndFetch() {
    let mut aQueue = BvhBuildQueue::new();

    let aProducerCount: i32 = 2;
    let aConsumerCount: i32 = 2;
    let aItemsPerProducer: i32 = 100;
    let aFetchedCount = AtomicI32::new(0);
    let aDone = AtomicBool::new(false);

    std::thread::scope(|scope| {
        // Producer threads
        let aProducers: Vec<_> = (0..aProducerCount)
            .map(|t| {
                let aQueue = &aQueue;
                scope.spawn(move || {
                    for i in 0..aItemsPerProducer {
                        aQueue.enqueue(t * aItemsPerProducer + i);
                    }
                })
            })
            .collect();

        // Consumer threads
        let aConsumers: Vec<_> = (0..aConsumerCount)
            .map(|_| {
                let (aQueue, aFetchedCount, aDone) = (&aQueue, &aFetchedCount, &aDone);
                scope.spawn(move || {
                    let mut wasBusy = false;
                    while !aDone.load(Ordering::SeqCst) || aQueue.size() > 0 {
                        let aItem = aQueue.fetch(&mut wasBusy);
                        if aItem != -1 {
                            aFetchedCount.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                })
            })
            .collect();

        // Wait for producers to finish
        for aProducer in aProducers {
            aProducer.join().unwrap();
        }

        // Signal consumers that production is done
        aDone.store(true, Ordering::SeqCst);

        // Wait for consumers to finish
        for aConsumer in aConsumers {
            aConsumer.join().unwrap();
        }
    });

    assert_eq!(
        aFetchedCount.load(Ordering::SeqCst),
        aProducerCount * aItemsPerProducer
    );
    assert_eq!(aQueue.size(), 0);
}

// =============================================================================
// BVH_BuildQueue Edge Cases
// =============================================================================

#[test]
fn BVH_BuildQueueTest__RepeatedFetchFromEmpty() {
    let mut aQueue = BvhBuildQueue::new();
    let mut wasBusy = false;

    for _i in 0..10 {
        let mut aResult = aQueue.fetch(&mut wasBusy);
        assert_eq!(aResult, -1);
        assert!(!wasBusy);
    }

    assert!(!aQueue.has_busy_threads());
}

#[test]
fn BVH_BuildQueueTest__EnqueueZero() {
    let mut aQueue = BvhBuildQueue::new();

    aQueue.enqueue(0);
    assert_eq!(aQueue.size(), 1);

    let mut wasBusy = false;
    assert_eq!(aQueue.fetch(&mut wasBusy), 0);
    assert!(wasBusy);
}

#[test]
fn BVH_BuildQueueTest__DuplicateValues() {
    let mut aQueue = BvhBuildQueue::new();

    aQueue.enqueue(5);
    aQueue.enqueue(5);
    aQueue.enqueue(5);

    assert_eq!(aQueue.size(), 3);

    let mut wasBusy = false;
    assert_eq!(aQueue.fetch(&mut wasBusy), 5);
    assert_eq!(aQueue.fetch(&mut wasBusy), 5);
    assert_eq!(aQueue.fetch(&mut wasBusy), 5);
}

#[test]
fn BVH_BuildQueueTest__SingleThreadWorkflow() {
    let mut aQueue = BvhBuildQueue::new();

    // Simulate single-threaded BVH building workflow
    aQueue.enqueue(0); // Root node

    let mut wasBusy = false;
    let mut aNode = aQueue.fetch(&mut wasBusy);

    assert_eq!(aNode, 0);
    assert!(wasBusy);

    // After processing root, enqueue children
    aQueue.enqueue(1); // Left child
    aQueue.enqueue(2); // Right child

    assert_eq!(aQueue.size(), 2);

    aNode = aQueue.fetch(&mut wasBusy);
    assert_eq!(aNode, 1);

    aNode = aQueue.fetch(&mut wasBusy);
    assert_eq!(aNode, 2);

    aNode = aQueue.fetch(&mut wasBusy);
    assert_eq!(aNode, -1);
    assert!(!wasBusy);
}
