use std::collections::BTreeSet;

use crate::{EarlyOut, MergeOperation, MergeStateRead};
use proptest::prelude::*;

struct VecMergeState<'a, T> {
    a: std::slice::Iter<'a, T>,
    b: std::slice::Iter<'a, T>,
    r: Vec<T>,
}

impl<'a, T> MergeStateRead for VecMergeState<'a, T> {
    type A = T;

    type B = T;

    fn a_slice(&self) -> &[Self::A] {
        self.a.as_slice()
    }

    fn b_slice(&self) -> &[Self::B] {
        self.b.as_slice()
    }
}

struct BoolMergeState<'a, T> {
    a: std::slice::Iter<'a, T>,
    b: std::slice::Iter<'a, T>,
    r: bool,
}

impl<'a, T> MergeStateRead for BoolMergeState<'a, T> {
    type A = T;

    type B = T;

    fn a_slice(&self) -> &[Self::A] {
        self.a.as_slice()
    }

    fn b_slice(&self) -> &[Self::B] {
        self.b.as_slice()
    }
}

struct Union;

impl<'a, T: Ord + Copy> MergeOperation<VecMergeState<'a, T>> for Union {
    fn from_a(&self, m: &mut VecMergeState<'a, T>, n: usize) -> EarlyOut {
        m.r.extend((&mut m.a).cloned().take(n));
        Some(())
    }

    fn from_b(&self, m: &mut VecMergeState<'a, T>, n: usize) -> EarlyOut {
        m.r.extend((&mut m.b).cloned().take(n));
        Some(())
    }

    fn collision(&self, m: &mut VecMergeState<'a, T>) -> EarlyOut {
        m.r.extend((&mut m.a).cloned().take(1));
        m.b.next();
        Some(())
    }

    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
}

struct Intersection;

impl<'a, T: Ord + Copy> MergeOperation<VecMergeState<'a, T>> for Intersection {
    fn from_a(&self, m: &mut VecMergeState<'a, T>, n: usize) -> EarlyOut {
        (&mut m.a).take(n).for_each(drop);
        Some(())
    }

    fn from_b(&self, m: &mut VecMergeState<'a, T>, n: usize) -> EarlyOut {
        (&mut m.b).take(n).for_each(drop);
        Some(())
    }

    fn collision(&self, m: &mut VecMergeState<'a, T>) -> EarlyOut {
        m.r.extend((&mut m.a).cloned().take(1));
        m.b.next();
        Some(())
    }

    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
}

struct Intersects;

impl<'a, T: Ord + Copy> MergeOperation<BoolMergeState<'a, T>> for Intersects {
    fn from_a(&self, m: &mut BoolMergeState<'a, T>, n: usize) -> EarlyOut {
        (&mut m.a).take(n).for_each(drop);
        Some(())
    }

    fn from_b(&self, m: &mut BoolMergeState<'a, T>, n: usize) -> EarlyOut {
        (&mut m.b).take(n).for_each(drop);
        Some(())
    }

    fn collision(&self, m: &mut BoolMergeState<'a, T>) -> EarlyOut {
        m.r = true;
        None
    }

    fn cmp(&self, a: &T, b: &T) -> std::cmp::Ordering {
        a.cmp(b)
    }
}

fn arb_sorted_vec() -> BoxedStrategy<Vec<u8>> {
    any::<Vec<u8>>()
        .prop_map(|mut v| {
            v.sort();
            v.dedup();
            v
        })
        .boxed()
}

#[test]
fn smoke() {
    let a = vec![1, 2, 3, 4];
    let b = vec![4, 5, 6, 7];
    let mut s = VecMergeState {
        a: a.iter(),
        b: b.iter(),
        r: Default::default(),
    };
    Union.merge(&mut s);
    assert_eq!(s.r, vec![1, 2, 3, 4, 5, 6, 7]);
    let mut s = VecMergeState {
        a: a.iter(),
        b: b.iter(),
        r: Default::default(),
    };
    Intersection.merge(&mut s);
    assert_eq!(s.r, vec![4]);
    let mut s = BoolMergeState {
        a: a.iter(),
        b: b.iter(),
        r: Default::default(),
    };
    Intersects.merge(&mut s);
    assert_eq!(s.r, true);
}

fn std_set_union(a: Vec<u8>, b: Vec<u8>) -> Vec<u8> {
    let mut r = BTreeSet::new();
    r.extend(a.into_iter());
    r.extend(b.into_iter());
    r.into_iter().collect()
}

fn std_set_intersection(a: Vec<u8>, b: Vec<u8>) -> Vec<u8> {
    let a: BTreeSet<u8> = a.into_iter().collect();
    let b: BTreeSet<u8> = b.into_iter().collect();
    a.intersection(&b).cloned().collect()
}

fn std_set_intersects(a: Vec<u8>, b: Vec<u8>) -> bool {
    let a: BTreeSet<u8> = a.into_iter().collect();
    let b: BTreeSet<u8> = b.into_iter().collect();
    a.intersection(&b).next().is_some()
}

proptest! {
    #[test]
    fn union(
        a in arb_sorted_vec(),
        b in arb_sorted_vec(),
    ) {
        let mut s = VecMergeState {
            a: a.iter(),
            b: b.iter(),
            r: Default::default(),
        };
        Union.merge(&mut s);
        prop_assert_eq!(s.r, std_set_union(a, b));
    }

    #[test]
    fn intersection(
        a in arb_sorted_vec(),
        b in arb_sorted_vec(),
    ) {
        let mut s = VecMergeState {
            a: a.iter(),
            b: b.iter(),
            r: Default::default(),
        };
        Intersection.merge(&mut s);
        prop_assert_eq!(s.r, std_set_intersection(a, b));
    }

    #[test]
    fn intersects(
        a in arb_sorted_vec(),
        b in arb_sorted_vec(),
    ) {
        let mut s = BoolMergeState {
            a: a.iter(),
            b: b.iter(),
            r: Default::default(),
        };
        Intersects.merge(&mut s);
        prop_assert_eq!(s.r, std_set_intersects(a, b));
    }
}
