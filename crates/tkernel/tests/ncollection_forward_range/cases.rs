#[test]
fn NCollection_ForwardRangeTest__SFINAEDetection_Value() {
    assert_eq!(flags::<MockValueIterator<'_>>(), (true, false, false));
}

#[test]
fn NCollection_ForwardRangeTest__SFINAEDetection_Current() {
    assert_eq!(flags::<MockCurrentIterator<'_>>(), (false, true, false));
}

#[test]
fn NCollection_ForwardRangeTest__SFINAEDetection_CurrentId() {
    assert_eq!(flags::<MockCurrentIdIterator<'_>>(), (false, false, true));
}

#[test]
fn NCollection_ForwardRangeTest__SFINAEDetection_BothValueAndCurrent() {
    assert_eq!(flags::<MockBothValueAndCurrent<'_>>(), (true, true, false));
    let values = [1];
    let mut host = MockBothValueAndCurrent::new(&values, 1);
    let iter = ForwardRangeIter::new(&mut host);
    let _: &i32 = iter.current().unwrap();
}

#[test]
fn NCollection_ForwardRangeTest__SFINAEDetection_BothCurrentAndId() {
    assert_eq!(flags::<MockBothCurrentAndId<'_>>(), (false, true, true));
    let values = [1];
    let mut host = MockBothCurrentAndId::new(&values, 1);
    let iter = ForwardRangeIter::new(&mut host);
    let _: &i32 = iter.current().unwrap();
}

#[test]
fn NCollection_ForwardRangeTest__StandaloneRange_ValueIterator() {
    let values = [10, 20, 30, 40, 50];
    let mut sum = 0;
    let mut count = 0;
    for value in ForwardRange::new(MockValueIterator::new(&values, 5)) {
        sum += value;
        count += 1;
    }
    assert_eq!(sum, 150);
    assert_eq!(count, 5);
}

#[test]
fn NCollection_ForwardRangeTest__StandaloneRange_CurrentIterator() {
    let values = [1, 2, 3];
    let mut sum = 0;
    for value in ForwardRange::new(MockCurrentIterator::new(&values, 3)) {
        sum += value;
    }
    assert_eq!(sum, 6);
}

#[test]
fn NCollection_ForwardRangeTest__StandaloneRange_CurrentIdIterator_ByValue() {
    let values = [100, 200, 300];
    let mut sum = 0;
    for value in ForwardRange::new(MockCurrentIdIterator::new(&values, 3)) {
        let value: i32 = value;
        sum += value;
    }
    assert_eq!(sum, 600);
}

#[test]
fn NCollection_ForwardRangeTest__StandaloneRange_Empty() {
    let values = [1];
    let mut count = 0;
    for _value in ForwardRange::new(MockValueIterator::new(&values, 0)) {
        count += 1;
    }
    assert_eq!(count, 0);
}

#[test]
fn NCollection_ForwardRangeTest__StandaloneRange_SingleElement() {
    let values = [42];
    let mut result = 0;
    for value in ForwardRange::new(MockValueIterator::new(&values, 1)) {
        result = *value;
    }
    assert_eq!(result, 42);
}

#[test]
fn NCollection_ForwardRangeTest__StandaloneRange_NonCopyable_Move() {
    let values = [5, 10, 15];
    let mut sum = 0;
    for value in ForwardRange::new(MockNonCopyableIterator::new(&values, 3)) {
        sum += value;
    }
    assert_eq!(sum, 30);
}

#[test]
fn NCollection_ForwardRangeTest__InClassBeginEnd_RangeFor() {
    let values = [7, 14, 21];
    let mut host = MockWithBeginEnd::new(&values, 3);
    let mut sum = 0;
    for value in &mut host {
        sum += value;
    }
    assert_eq!(sum, 42);
}

#[test]
fn NCollection_ForwardRangeTest__InClassBeginEnd_Temporary() {
    let values = [1, 2, 3, 4];
    let mut sum = 0;
    for value in MockWithBeginEnd::new(&values, 4) {
        sum += value;
    }
    assert_eq!(sum, 10);
}

#[test]
fn NCollection_ForwardRangeTest__IteratorTraits_ValueType() {
    type Iter = ForwardRangeIter<'static, MockValueIterator<'static>>;
    assert_eq!(
        TypeId::of::<<Iter as ForwardIteratorTraits>::Value>(),
        TypeId::of::<i32>()
    );
    assert_eq!(
        TypeId::of::<<Iter as ForwardIteratorTraits>::Category>(),
        TypeId::of::<InputIterator>()
    );
    assert_eq!(
        TypeId::of::<<Iter as ForwardIteratorTraits>::Difference>(),
        TypeId::of::<isize>()
    );
}

#[test]
fn NCollection_ForwardRangeTest__IteratorTraits_ReferenceReturn() {
    type Iter = ForwardRangeIter<'static, MockValueIterator<'static>>;
    assert_eq!(
        TypeId::of::<<Iter as Iterator>::Item>(),
        TypeId::of::<&i32>()
    );
}

#[test]
fn NCollection_ForwardRangeTest__IteratorTraits_ValueReturn() {
    type Iter = ForwardRangeIter<'static, MockCurrentIdIterator<'static>>;
    assert_eq!(
        TypeId::of::<<Iter as ForwardIteratorTraits>::Value>(),
        TypeId::of::<i32>()
    );
}

#[test]
fn NCollection_ForwardRangeTest__ArrowOperator_Reference() {
    let values = [Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];
    let mut range = ForwardRange::new(MockPointIterator::new(&values, 2));
    let mut iter = range.iter();
    assert_eq!(iter.arrow().unwrap().x, 1);
    assert_eq!(iter.arrow().unwrap().y, 2);
    iter.advance();
    assert_eq!(iter.arrow().unwrap().x, 3);
    assert_eq!(iter.arrow().unwrap().y, 4);
}

#[test]
fn NCollection_ForwardRangeTest__ArrowOperator_ArrowProxy() {
    let values = [Point { x: 10, y: 20 }, Point { x: 30, y: 40 }];
    let mut range = ForwardRange::new(MockPointByValueIterator::new(&values, 2));
    let mut iter = range.iter();
    assert_eq!(iter.arrow().unwrap().x, 10);
    assert_eq!(iter.arrow().unwrap().y, 20);
    iter.advance();
    assert_eq!(iter.arrow().unwrap().x, 30);
    assert_eq!(iter.arrow().unwrap().y, 40);
}

#[test]
fn NCollection_ForwardRangeTest__PostfixIncrement() {
    let values = [100, 200, 300];
    let mut host = MockValueIterator::new(&values, 3);
    let mut iter = ForwardRangeIter::new(&mut host);
    let proxy = iter.post_increment().unwrap();
    assert_eq!(*proxy, 100);
    assert_eq!(*iter.current().unwrap(), 200);
}

#[test]
fn NCollection_ForwardRangeTest__SentinelComparison() {
    let values = [1];
    let mut host = MockValueIterator::new(&values, 1);
    let mut iter = ForwardRangeIter::new(&mut host);
    let sentinel = ForwardRangeSentinel;
    assert!(iter != sentinel);
    assert!(!(iter == sentinel));
    assert!(sentinel != iter);
    assert!(!(sentinel == iter));
    iter.advance();
    assert!(!(iter != sentinel));
    assert!(iter == sentinel);
    assert!(!(sentinel != iter));
    assert!(sentinel == iter);
}

#[test]
fn NCollection_ForwardRangeTest__Priority_ValueOverCurrent() {
    let values = [99];
    let mut result = 0;
    for value in ForwardRange::new(MockBothValueAndCurrent::new(&values, 1)) {
        result = *value;
    }
    assert_eq!(result, 99);
}

#[test]
fn NCollection_ForwardRangeTest__Priority_CurrentOverCurrentId() {
    let values = [55, 66];
    let mut sum = 0;
    for value in ForwardRange::new(MockBothCurrentAndId::new(&values, 2)) {
        sum += value;
    }
    assert_eq!(sum, 121);
}
