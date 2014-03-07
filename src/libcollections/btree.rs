// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// btree.rs
//

//! Simple implementation of a B-tree.

///A B-tree contains a root node (which contains a vector of elements),
///a length (the height of the tree), and lower and upper bounds on the
///number of elements that a given node can contain.

#[allow(missing_doc)]
pub struct BTree<K, V> {
    priv root: Node<K, V>,
    //priv len: uint,
    //priv lower_bound: uint,
    //priv upper_bound: uint
    priv min_deg: uint
}

//A node contains a vector of elements (key-value pairs) as well as children, optionally.
struct Node<K, V> {
    elts: ~[Elt<K, V>],
    children: Option<~[~Node<K,V>]>, // if Some(), a Branch
}

//An Elt contains a key-value pair.
struct Elt<K, V> {
    key: K,
    value: V
}

impl<K: TotalOrd, V> BTree<K, V> {

    ///Returns new BTree with root node (leaf) and user-supplied lower bound
    ///The lower bound applies to every node except the root node.
    pub fn new(k: K, v: V, md: uint) -> BTree<K, V> {
        BTree {
            root: Node {elts: ~[Elt {key: k, value: v}], children: None},
            //len: 1,
            //lower_bound: lb,
            //upper_bound: 2 * lb
            min_deg: md
        }
    }

    pub fn insert(&mut self, k: K, v: V) {
        //First, check to see if the root is full.
        if self.root.elts.len() >= self.min_deg * 2 - 1 {
            let mut new_root_elts = ~[];
            let new_root_children = match self.root.children {
                None => None,
                Some(ref mut kids) => {
                    let mut child_vec = ~[];
                    for i in range(0, kids.len()) {
                        child_vec.push(kids.pop().unwrap());
                    }
                    child_vec.reverse();
                    Some(child_vec)
                }
            };
            for i in range(0, self.root.elts.len()){
                new_root_elts.push(self.root.elts.pop().unwrap());
            }
            new_root_elts.reverse();
            let new_root = ~Node { elts: new_root_elts, children: new_root_children };
            self.root = Node { elts: ~[], children: Some(~[new_root]) };
            //self.root = Node {elts: ~[], children: Some(~[~self.root])};
            self.root.split_child(0, self.min_deg * 2 - 1);
            self.root.insert_nonfull(k, v, self.min_deg * 2 - 1);
        }
        //If it is not full, call the helper method for a non-full Node.
        else {
            self.root.insert_nonfull(k, v, self.min_deg * 2 - 1);
        }
    }
}

impl<K: TotalOrd, V> Node<K, V> {
    fn split_child(&mut self, i: uint, ub: uint) {
        if self.children.get_ref()[i].elts.len() < ub { return; }
        let mut new_elts_left = ~[];
        let mut new_elts_right = ~[];
        let new_node_left;
        let new_node_right;
        let mid_elt;
        {
            let child: &mut Node<K,V> = &mut *self.children.get_mut_ref()[i];
            for j in range(0, child.elts.len() / 2) {
                new_elts_right.push(child.elts.pop().unwrap());
            }
            new_elts_right.reverse();
            mid_elt = child.elts.pop().unwrap();
            for j in range(0, child.elts.len()) {
                new_elts_left.push(child.elts.pop().unwrap());
            }
            new_elts_left.reverse();
            let new_opt_grandchildren_right = match child.children {
                None => None,
                Some(ref mut gchild) => {
                    let mut grandchildren = ~[];
                    for j in range(0, gchild.len() / 2) {
                        grandchildren.push(gchild.pop().unwrap());
                    }
                    grandchildren.reverse();
                    Some(grandchildren)
                }
            };
            let new_opt_grandchildren_left = match child.children {
                None => None,
                Some(ref mut gchild) => {
                    let mut grandchildren = ~[];
                    for j in range(0, gchild.len()) {
                        grandchildren.push(gchild.pop().unwrap());
                    }
                    grandchildren.reverse();
                    Some(grandchildren)
                }
            };
            new_node_left = ~Node { elts: new_elts_left, children: new_opt_grandchildren_left };
            new_node_right = ~Node { elts: new_elts_right, children: new_opt_grandchildren_right };
        }
        self.elts.insert(i, mid_elt);
        self.children.get_mut_ref().insert(i, new_node_left);
        self.children.get_mut_ref().insert(i + 1, new_node_right);
    }

    fn insert_nonfull(&mut self, k: K, v: V, ub: uint) {
        match self.children {
            //If we have no children, we are a Leaf and can insert here.
            None => {
                //Check the index returned by bsearch: is the key already there?
                let mut index = self.bsearch_node(&k);
                //Check to make sure the index is in bounds.
                if self.elts.len() <= index {
                    self.elts.push(Elt { key: k, value: v });
                }
                else {
                    match self.elts[index].key.cmp(&k) {
                        //If there is already a key at that index that matches
                        //the one we want to add, just update the value.
                        Equal => {
                            self.elts[index].value = v;
                        }
                        //Check this: it should be Greater every time it's not Equal.
                        _ => {
                            self.elts.insert(index, Elt { key: k, value: v });
                        }
                    }
                }
                //If we have no children, we're done here.
                return;
            }
            Some(..) => {
                let mut index = self.bsearch_node(&k);
                self.split_child(index, ub);

                //First check to make sure index is in bounds.
                if index < self.elts.len() {

                    //Does the split cause us to change the index?  Check here.
                    match self.elts[index].key.cmp(&k) {
                        Greater => {
                            index = index + 1;
                        }
                        //Index should stay the same and equal cases are handled by leaves.
                        _ => {}
                    }
                }
                //Check to see if we need to split the child.
                let child: &mut Node<K,V> = &mut *self.children.get_mut_ref()[index];
                //Regardless of whether we split the child, we now move to that child.
                //let child: &mut Node<K,V> = &mut *self.children.get_mut_ref()[index];
                child.insert_nonfull(k, v, ub);
            }
        }
    }

    ///Searches a node for an index at which to insert a new key.
    fn bsearch_node(&self, k: &K) -> uint {
        let mut min = 0;
        let mut max = self.elts.len();
        let mut mid = (min + max) / 2;
        match self.elts[min].key.cmp(k) {
            Greater => { return 0; }
            _ => {}
        }
        match self.elts[max - 1].key.cmp(k) {
            Less => { return max; }
            _ => {}
        }
        //println!("min is {} max is {}", min, max);
        while max > min && min != mid && max != mid {
            //println!("mid is {}", mid);
            match self.elts[mid].key.cmp(k) {
                Equal => {
                    return mid;
                }
                Less => {
                    max = mid;
                }
                Greater => {
                    min = mid;
                }
            }
            mid = (min + max) / 2;
        }
        mid
    }
}

#[cfg(test)]
mod test_btree {
    use super::{BTree, Node, Elt};

    #[test]
    fn split_child_test_1() {
        let mut new_node = Node { elts: ~[Elt { key: 3, value: ~"a" },
                                          Elt { key: 7, value: ~"b" }],
                                  children: Some(~[~Node { elts: ~[Elt { key: 1, value: ~"c" },
                                                                   Elt { key: 2, value: ~"d" }],
                                                           children: None },
                                                   ~Node { elts: ~[Elt { key: 4, value: ~"e" },
                                                                   Elt { key: 5, value: ~"f" },
                                                                   Elt { key: 6, value: ~"g" }],
                                                           children: None },
                                                   ~Node { elts: ~[Elt { key: 8, value: ~"h" },
                                                                   Elt { key: 9, value: ~"i" }],
                                                           children: None }])};
        new_node.split_child(1, 2);
        assert_eq!(new_node.elts[1].key, 5);
    }

    #[test]
    fn split_child_test_2() {
        let mut new_node = Node { elts: ~[Elt { key: 3, value: ~"a" },
                                          Elt { key: 7, value: ~"b" }],
                                  children: Some(~[~Node { elts: ~[Elt { key: 1, value: ~"c" },
                                                                   Elt { key: 2, value: ~"d" }],
                                                           children: None },
                                                   ~Node { elts: ~[Elt { key: 4, value: ~"e" },
                                                                   Elt { key: 5, value: ~"f" },
                                                                   Elt { key: 6, value: ~"g" }],
                                                           children: None },
                                                   ~Node { elts: ~[Elt { key: 8, value: ~"h" },
                                                                   Elt { key: 9, value: ~"i" }],
                                                           children: None }])};
        new_node.split_child(1, 2);
        assert_eq!(new_node.children.unwrap()[1].elts[0].key, 4);
    }

    #[test]
    fn split_child_test_3() {
        let mut new_node = Node { elts: ~[Elt { key: 3, value: ~"a" },
                                          Elt { key: 11, value: ~"b" }],
                                  children: Some(~[~Node { elts: ~[Elt { key: 1, value: ~"c" },
                                                                   Elt { key: 2, value: ~"d" }],
                                                           children: None },
                                                   ~Node { elts: ~[Elt { key: 5, value: ~"e" },
                                                                   Elt { key: 7, value: ~"f" },
                                                                   Elt { key: 9, value: ~"g" }],
                                                           children: Some(~[~Node { elts: ~[Elt { key: 4, value: ~"h" }],
                                                                                    children: None },
                                                                            ~Node { elts: ~[Elt { key: 6, value: ~"i" }],
                                                                                    children: None },
                                                                            ~Node { elts: ~[Elt { key: 8, value: ~"j" }],
                                                                                    children: None },
                                                                            ~Node { elts: ~[Elt { key: 10, value: ~"k" }],
                                                                                    children: None }])},
                                                   ~Node { elts: ~[Elt { key: 12, value: ~"l" },
                                                                   Elt { key: 13, value: ~"m" }],
                                                           children: None }])};
        new_node.split_child(1, 2);
        assert_eq!(new_node.children.unwrap()[1].children.unwrap()[0].elts[0].key, 4);
    }

    #[test]
    fn insert_test_1() {
        let mut new_tree = BTree { root: Node { elts: ~[Elt { key: 1, value: ~"a" },
                                                        Elt { key: 3, value: ~"c" }],
                                                children: None },
                                   min_deg: 2 };
        new_tree.insert(2, ~"b");
        assert_eq!(new_tree.root.elts[1].key, 2);
    }

    #[test]
    fn insert_test_2() {
        let mut new_tree = BTree { root: Node { elts: ~[Elt { key: 1, value: ~"a" },
                                                        Elt { key: 2, value: ~"b" }],
                                                children: None },
                                   min_deg: 2 };
        new_tree.insert(3, ~"c");
        assert_eq!(new_tree.root.elts[2].key, 3);
    }

    #[test]
    fn insert_test_3() {
        let mut new_tree = BTree { root: Node { elts: ~[Elt { key: 2, value: ~"b" },
                                                        Elt { key: 3, value: ~"c" }],
                                                children: None },
                                   min_deg: 2 };
        new_tree.insert(1, ~"a");
        assert_eq!(new_tree.root.elts[0].key, 1);
    }

    #[test]
    fn insert_test_4() {
        let mut new_tree = BTree { root: Node { elts: ~[Elt { key: 1, value: ~"a" },
                                                        Elt { key: 2, value: ~"b" },
                                                        Elt { key: 3, value: ~"c" },
                                                        Elt { key: 4, value: ~"d" }],
                                                children: None },
                                   min_deg: 2 };
        new_tree.insert(5, ~"3");
        assert_eq!(new_tree.root.elts[0].key, 2);
    }

    #[test]
    fn insert_test_5() {
        let mut new_node = Node { elts: ~[Elt { key: 2, value: ~"a" },
                                          Elt { key: 8, value: ~"b" }],
                                  children: Some(~[~Node { elts: ~[Elt { key: 0, value: ~"c" },
                                                                   Elt { key: 1, value: ~"d" }],
                                                           children: None },
                                                   ~Node { elts: ~[Elt { key: 3, value: ~"x" },
                                                                   Elt { key: 4, value: ~"e" },
                                                                   Elt { key: 6, value: ~"f" },
                                                                   Elt { key: 7, value: ~"g" }],
                                                           children: None },
                                                   ~Node { elts: ~[Elt { key: 9, value: ~"h" },
                                                                   Elt { key: 10, value: ~"i" }],
                                                           children: None }])};
        let mut new_tree = BTree { root: new_node,
                                   min_deg: 2 };
        new_tree.insert(5, ~"omg");
        assert_eq!(new_tree.root.elts[1].key, 4);
    }


}
