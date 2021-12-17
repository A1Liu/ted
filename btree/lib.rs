#![no_std]

mod nodes;
mod traits;
mod tree;
mod util;

extern crate alloc;

pub use traits::*;
pub use tree::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Default, Debug)]
    struct TestData(usize);

    impl BTreeInfo for TestData {
        fn add(self, other: Self) -> Self {
            return Self(self.0 + other.0);
        }
    }

    impl BTreeItem for TestData {
        type Info = Self;

        fn get_info(&self) -> Self::Info {
            return TestData(self.0);
        }
    }

    const TREE_SIZE: usize = 10000;

    fn validate(tree: BTree<TestData>) {
        for i in 0..TREE_SIZE {
            assert_eq!(i, tree.get(i).unwrap().0);
        }

        let mut total = 0;
        for i in 0..TREE_SIZE {
            let next = total + i;
            if next == total {
                continue;
            }

            let test_cases = [
                total,
                (3 * total + next) / 4,
                (total + next) / 2,
                (total + 3 * next) / 4,
                next - 1,
            ];
            for key in test_cases.into_iter() {
                let (value, remainder) = tree.key(key, |n| n.0).unwrap();
                assert_eq!(i, value.0);
                assert_eq!(key - total, remainder);

                let sum = tree.sum_until(i).unwrap().0;
                assert_eq!(total, sum);
            }

            total = next;
        }
    }

    #[test]
    fn forward_insert() {
        let mut tree = BTree::new();

        for i in 0..TREE_SIZE {
            tree.insert(i, TestData(i));
        }

        validate(tree);
    }

    #[test]
    fn reverse_insert() {
        let mut tree = BTree::new();

        for i in 0..TREE_SIZE {
            tree.insert(0, TestData(TREE_SIZE - 1 - i));
        }

        validate(tree);
    }
}
