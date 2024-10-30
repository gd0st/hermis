use crate::Feed;
use crate::Weighted;
use std::cmp::Ordering;
use std::{cell::RefCell, env::current_exe, fmt::Write, ops::Deref};

use itertools::{concat, Itertools};

#[derive(Debug)]
pub struct Node<T> {
    value: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T> Default for Node<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            value: T::default(),
            left: None,
            right: None,
        }
    }
}

#[derive(Default, Debug)]
pub struct BinarySearchTree<T> {
    root: Option<Box<Node<T>>>,
}

impl<T> BinarySearchTree<T>
where
    T: PartialEq + PartialOrd,
{
    fn insert(&mut self, value: T) {
        if let Some(node) = &mut self.root {
            node.insert(Node::new(value))
        } else {
            self.root = Some(Box::new(Node::new(value)))
        }
    }

    fn root(&self) -> Option<&Node<T>> {
        let Some(root) = &self.root else {
            return None;
        };
        Some(root.as_ref())
    }
}

impl<T> Node<T>
where
    T: Clone,
{
    pub fn recursive_inorder(values: &mut Vec<T>, node: &Self) {
        if let Some(left) = &node.left {
            Self::recursive_inorder(values, left);
        }
        values.push(node.value.clone());

        if let Some(right) = &node.right {
            Self::recursive_inorder(values, right);
        }
    }

    pub fn get_inorder(&self) -> Vec<T> {
        let mut values = vec![];
        Self::recursive_inorder(&mut values, self);
        values
    }
}

impl<T> TryFrom<Vec<T>> for Node<T>
where
    T: PartialEq + PartialOrd,
{
    type Error = String;
    fn try_from(mut value: Vec<T>) -> Result<Self, Self::Error> {
        value.reverse();
        let Some(first) = value.pop() else {
            return Err("cannot build tree from empty list".to_string());
        };

        let mut root = Node::new(first);

        while let Some(value) = value.pop() {
            root.insert(Node::new(value))
        }

        Ok(root)
    }
}

fn delete<T: Weighted>(mut this: Box<Node<T>>, weight: usize) -> Option<Box<Node<T>>> {
    if weight < this.get_ref().weight() {
        if let Some(left) = this.left {
            this.left = delete(left, weight)
        }
        return Some(this);
    }

    if weight > this.get_ref().weight() {
        if let Some(right) = this.right {
            this.right = delete(right, weight);
        }

        return Some(this);
    }

    None
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
        }
    }

    pub fn right(&self) -> Option<&Node<T>> {
        if let Some(right) = &self.right {
            Some(right.as_ref())
        } else {
            None
        }
    }

    pub fn left(&self) -> Option<&Node<T>> {
        if let Some(left) = &self.left {
            Some(left.as_ref())
        } else {
            None
        }
    }

    pub fn has_children(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    pub fn get_ref(&self) -> &T {
        &self.value
    }

    pub fn get_ref_mut(&mut self) -> &mut T {
        // not a fan of this
        &mut self.value
    }

    pub fn inorder(&self) -> Vec<T> {
        let mut output = vec![];
        loop {
            break;
        }

        output
    }
}

impl<T> Node<T>
where
    T: PartialEq + PartialOrd,
{
    pub fn insert(&mut self, node: Self) {
        if node.value < self.value {
            if let Some(left) = &mut self.left {
                left.insert(node)
            } else {
                self.left = Some(Box::new(node))
            }
        } else {
            if let Some(right) = &mut self.right {
                right.insert(node)
            } else {
                self.right = Some(Box::new(node))
            }
        }
    }
}

pub fn search_and_remove(root: &Node<Feed>, value: usize) -> &Node<Feed> {
    let mut current = root;

    loop {
        let left = current.left();
        let right = current.right();
        if left.is_none() && right.is_none() {
            break;
        }

        if current.get_ref().weight == value {
            break;
        }

        if value > current.get_ref().weight {
            if current.right.is_none() {
                break;
            }
            current = &mut right.unwrap();
            continue;
        } else {
            if current.left.is_none() {
                break;
            }
            current = &mut left.unwrap();
            continue;
        }
    }

    if current.left().is_none() && current.right().is_none() {
        return current;
    }
    current
}

pub fn cfd(elements: Vec<usize>) -> Vec<usize> {
    let mut summed_weights = vec![];

    for num in elements {
        summed_weights.push(num + summed_weights.iter().sum::<usize>())
    }

    summed_weights
}

#[cfg(test)]
mod test {

    use super::*;
    use rand::{seq::IteratorRandom, thread_rng, Rng};
    #[test]
    fn get_inorder() {
        let tree: Node<usize> = vec![3, 2, 5].try_into().unwrap();
        assert_eq!(vec![2, 3, 5], tree.get_inorder());
    }

    fn make_feeds(weights: Vec<usize>) -> Vec<Feed> {
        weights
            .into_iter()
            .map(|weight| {
                let mut feed = Feed::default();
                feed.weight = weight;
                feed
            })
            .collect()
    }

    #[test]
    fn sum_weights() {
        // let mut first_feed = Feed::default();
        // let feeds = make_feeds(vec![5, 2]);
        // let summed_weights = cfd(feeds.iter().collect());
        let summed_weights = cfd(vec![5, 2]);
        assert_eq!(summed_weights, vec![5, 7])
    }

    #[test]
    fn insert_feeds_to_bst_and_select() {
        let cfd_weights = cfd(vec![2, 5, 10]);
        let feeds = make_feeds(cfd_weights.clone());
        let target = 20;
        let tree = Node::try_from(feeds).expect("empty list passed to tree");
        let feed = search_and_remove(&tree, target);
        assert_eq!(feed.get_ref().weight, 19)
    }
}
