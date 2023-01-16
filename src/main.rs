#![allow(warnings)]

use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

const DEBUG: bool = false;
const WAVL_TREE: bool = true;

pub struct Tree {
    count: usize,
    root: *mut Node,
    rotations: usize,
    accessed_nodes: usize,
}

#[derive(Debug)]
struct Node {
    data: i32,
    rank: i32,

    left: *mut Node,
    right: *mut Node,
    parent: *mut Node,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            count: 0,
            rotations: 0,
            accessed_nodes: 0,
            root: std::ptr::null_mut(),
        }
    }

    pub fn reset_rotations(&mut self) {
        self.rotations = 0;
    }

    pub fn reset_accessed_nodes(&mut self) {
        self.accessed_nodes = 0;
    }

    fn increase_rotations(&mut self) {
        self.rotations += 1;
    }

    fn increase_nodes(&mut self) {
        self.accessed_nodes += 1;
    }

    // Insert(k)
    pub fn insert(&mut self, data: i32) -> bool {
        self.increase_nodes();
        if self.root.is_null() {
            self.root = Node::new(data);
        } else {
            if !self.insert_node(self.root, data) {
                return false;
            }
        }

        self.count += 1;

        true
    }

    // InorderWalk()
    pub fn inorder(&self) -> Vec<(i32, i32)> {
        let mut v = vec![];
        if !self.root.is_null() {
            let mut node = leftmost_child(self.root);
            loop {
                if node.is_null() {
                    break;
                }
                unsafe {
                    v.push(((*node).data, (*node).rank));
                }
                node = successor_of_node(node);
            }
        }
        v
    }

    // Delete(k)
    pub fn remove(&mut self, data: i32) -> bool {
        let node = self.find_node(self.root, data);
        if node.is_null() {
            false
        } else {
            unsafe {
                let parent = (*node).parent;
                self.remove_node(node, true);
            }

            self.count -= 1;
            true
        }
    }

    // Successor(x)
    pub fn successor(&mut self, data: i32) -> Option<i32> {
        unsafe {
            let node = self.find_node(self.root, data);
            if !node.is_null() {
                let nodesucc = successor_of_node(node);
                if !nodesucc.is_null() {
                    return Some((*nodesucc).data);
                }
            }
            None
        }
    }

    // Predecessor(x)
    pub fn predecessor(&mut self, data: i32) -> Option<i32> {
        unsafe {
            let node = self.find_node(self.root, data);
            if !node.is_null() {
                let nodepred = predecessor_of_node(node);
                if !nodepred.is_null() {
                    return Some((*nodepred).data);
                }
            }
            None
        }
    }

    // Search(k)
    pub fn find(&mut self, data: i32) -> bool {
        !self.find_node(self.root, data).is_null()
    }

    fn remove_node(&mut self, node: *mut Node, rebalance: bool) {
        unsafe {
            let lchild = (*node).left;
            let rchild = (*node).right;
            if lchild.is_null() && rchild.is_null() {
                self.replace_node(node, std::ptr::null_mut(), rebalance);
            } else if !lchild.is_null() && !rchild.is_null() {
                let succ = successor_of_node(node);
                assert!(!succ.is_null());
                (*node).data = (*succ).data;
                self.remove_node(succ, rebalance);
            } else if !lchild.is_null() {
                self.replace_node(node, lchild, rebalance);
            } else if !rchild.is_null() {
                self.replace_node(node, rchild, rebalance);
            } else {
                unreachable!("Unreachable");
            }
        }
    }

    pub fn node_count(&self) -> usize {
        assert!(self.count != 0 || self.root.is_null());
        self.count
    }

    fn replace_node(&mut self, mut node: *mut Node, r: *mut Node, rebalance: bool) {
        unsafe {
            let parent = (*node).parent;
            if parent.is_null() {
                // Remove root node
                self.root = r;
                if !r.is_null() {
                    (*r).parent = std::ptr::null_mut();
                }
            } else {
                if !r.is_null() {
                    (*r).parent = parent;
                }
                if (*parent).left == node {
                    (*parent).left = r;
                } else if (*parent).right == node {
                    (*parent).right = r;
                }
            }

            if rebalance && WAVL_TREE {
                self.balance_deleted(r, parent);
            }
            drop(Box::from_raw(node));
        }
    }

    fn find_node(&mut self, fromnode: *mut Node, data: i32) -> *mut Node {
        unsafe {
            if fromnode.is_null() || (*fromnode).data == data {
                fromnode
            } else if data < (*fromnode).data {
                self.increase_nodes();
                self.find_node((*fromnode).left, data)
            } else {
                self.increase_nodes();
                self.find_node((*fromnode).right, data)
            }
        }
    }

    fn insert_node(&mut self, node: *mut Node, data: i32) -> bool {
        unsafe {
            if (*node).data == data {
                false
            } else if data < (*node).data {
                if (*node).left.is_null() {
                    (*node).left = Node::new_with_parent(data, node);
                    if WAVL_TREE {
                        self.balance_inserted((*node).left);
                    }
                    true
                } else {
                    self.increase_nodes();
                    self.insert_node((*node).left, data)
                }
            } else {
                if (*node).right.is_null() {
                    (*node).right = Node::new_with_parent(data, node);
                    if WAVL_TREE {
                        self.balance_inserted((*node).right);
                    }
                    true
                } else {
                    self.increase_nodes();
                    self.insert_node((*node).right, data)
                }
            }
        }
    }

    fn balance_inserted(&mut self, mut node: *mut Node) {
        if DEBUG {
            println!("Balance inserted");
        }
        unsafe {
            loop {
                let node_ref = if let Some(node_ref) = node.as_ref() {
                    node_ref
                } else {
                    break;
                };

                let parent_ref = if let Some(parent_ref) = node_ref.parent.as_ref() {
                    parent_ref
                } else {
                    break;
                };

                if (*node).parent.is_null() || parent_ref.rank - node_ref.rank == 1 {
                    break;
                }

                let parent = (*node).parent;
                if parent_ref.rank - get_node_sibling_rank(node, parent) == 1 {
                    promote(node_ref.parent);
                    node = node_ref.parent;
                    continue;
                }

                let parent_left_rank = if let Some(l_ref) = (*parent).left.as_ref() {
                    l_ref.rank
                } else {
                    0
                };
                let parent_right_rank = if let Some(r_ref) = (*parent).right.as_ref() {
                    r_ref.rank
                } else {
                    0
                };

                let node_rank = if let Some(node_ref) = node.as_ref() {
                    node_ref.rank
                } else {
                    0
                };

                let node_left_rank = if let Some(node_ref) = node.as_ref() {
                    if !node_ref.left.is_null() {
                        (*node_ref.left).rank
                    } else {
                        0
                    }
                } else {
                    unreachable!("Node cannot be null")
                };

                let node_right_rank = if let Some(node_ref) = node.as_ref() {
                    if !node_ref.right.is_null() {
                        (*node_ref.right).rank
                    } else {
                        0
                    }
                } else {
                    unreachable!("Node cannot be null")
                };

                let parent_rank = if let Some(parent_ref) = parent.as_ref() {
                    parent_ref.rank
                } else {
                    0
                };

                let parent_ref = if let Some(parent_ref) = node_ref.parent.as_ref() {
                    parent_ref
                } else {
                    break;
                };

                if parent_ref.rank - get_node_sibling_rank(node, parent) == 2 {
                    if DEBUG {
                        println!("Start rotation");
                    }
                    if parent_ref.right == node {
                        // Rotate left
                        if node_rank - node_right_rank == 1 {
                            let s = get_node_sibling(node, parent);
                            if parent_left_rank == get_node_sibling_rank(node, parent) {
                                if DEBUG {
                                    println!("Start left rotation");
                                }
                                let t = (*node).right;
                                self.increase_rotations();
                                self.rotate_left(parent);
                                demote(parent);
                                if DEBUG {
                                    println!("End left rotation");
                                }
                            }
                            break;
                        } else if node_rank - node_left_rank == 1 {
                            let s = get_node_sibling(node, parent);
                            if parent_left_rank == get_node_sibling_rank(node, parent) {
                                if DEBUG {
                                    println!("Start right left double rotation");
                                }
                                let t = (*node).left;
                                self.increase_rotations();
                                self.rotate_right(node);
                                demote(node);
                                promote(t);

                                self.increase_rotations();
                                self.rotate_left(parent);
                                demote(parent);
                                if DEBUG {
                                    println!("End right left double rotation");
                                }
                            }
                            break;
                        }
                    } else {
                        // Rotate right
                        if node_rank - node_left_rank == 1 {
                            let s = get_node_sibling(node, parent);
                            if parent_right_rank == get_node_sibling_rank(node, parent) {
                                if DEBUG {
                                    println!("Start right rotation");
                                }
                                let t = (*node).left;
                                self.increase_rotations();
                                self.rotate_right(parent);
                                demote(parent);
                                if DEBUG {
                                    println!("End right rotation");
                                }
                            }
                            break;
                        } else if node_rank - node_right_rank == 1 {
                            let s = get_node_sibling(node, parent);
                            if parent_right_rank == get_node_sibling_rank(node, parent) {
                                if DEBUG {
                                    println!("Start left right double rotation");
                                }
                                let t = (*node).right;
                                self.increase_rotations();
                                self.rotate_left(node);
                                demote(node);
                                promote(t);

                                self.increase_rotations();
                                self.rotate_right(parent);
                                demote(parent);
                                if DEBUG {
                                    println!("End left right double rotation");
                                }
                            }
                            break;
                        }
                    }
                    if DEBUG {
                        println!("End rotation");
                    }
                    break;
                } else {
                    break;
                }
            }
        };
    }

    fn balance_deleted(&mut self, mut node: *mut Node, mut parent: *mut Node) {
        if DEBUG {
            println!("Balance deleted");
        }
        loop {
            unsafe {
                let node_rank = if let Some(node_ref) = node.as_ref() {
                    node_ref.rank
                } else {
                    0
                };
                let parent_rank = if let Some(parent_ref) = parent.as_ref() {
                    parent_ref.rank
                } else {
                    unimplemented!("Parent is null");
                };

                if parent_rank - node_rank == 1 {
                    break;
                }

                if parent_rank - node_rank == 3 {
                    if parent_rank - get_node_sibling_rank(node, parent) == 2 {
                        if DEBUG {
                            println!("Fix inbalance by demoting parent");
                        }
                        demote(parent);

                        node = parent;
                        parent = (*node).parent;

                        continue;
                    }

                    if parent_rank - get_node_sibling_rank(node, parent) == 1 {
                        if get_node_sibling_rank(node, parent)
                            - get_node_sibling_left_rank(node, parent)
                            == 2
                            && get_node_sibling_rank(node, parent)
                                - get_node_sibling_right_rank(node, parent)
                                == 2
                        {
                            if DEBUG {
                                println!("Fix inbalance by demoting parent and sibling");
                            }
                            demote(parent);
                            let sibling = get_node_sibling(node, parent);
                            demote(sibling);

                            node = parent;
                            parent = (*node).parent;
                            continue;
                        }

                        let parent_ref = if let Some(parent_ref) = parent.as_ref() {
                            parent_ref
                        } else {
                            break;
                        };

                        let parent_left_rank = if let Some(parent_ref) = parent.as_ref() {
                            if !parent_ref.left.is_null() {
                                (*parent_ref.left).rank
                            } else {
                                0
                            }
                        } else {
                            unreachable!("parent_left_rank");
                        };

                        let parent_right_rank = if let Some(parent_ref) = parent.as_ref() {
                            if !parent_ref.right.is_null() {
                                (*parent_ref.right).rank
                            } else {
                                0
                            }
                        } else {
                            unreachable!("parent_right_rank");
                        };

                        if parent_ref.right == node {
                            if get_node_sibling_rank(node, parent)
                                - get_node_sibling_left_rank(node, parent)
                                == 1
                            {
                                let s = get_node_sibling(node, parent);

                                if parent_left_rank == get_node_sibling_rank(node, parent) {
                                    if DEBUG {
                                        println!("Single right rotation");
                                    }
                                    let t = (*s).left;
                                    self.increase_rotations();
                                    self.rotate_right(parent);
                                    demote(parent);
                                    promote(s);
                                }
                                check_after_rotation(node, parent);
                                break;
                            } else if get_node_sibling_rank(node, parent)
                                - get_node_sibling_right_rank(node, parent)
                                == 1
                            {
                                if DEBUG {
                                    println!("Double left right rotation");
                                }
                                let s = get_node_sibling(node, parent);
                                if parent_left_rank == get_node_sibling_rank(node, parent) {
                                    let t = (*s).right;
                                    self.increase_rotations();
                                    self.rotate_left(s);
                                    demote(s);
                                    promote(t);

                                    self.increase_rotations();
                                    self.rotate_right(parent);
                                    demote(parent);
                                }
                                check_after_rotation(node, parent);
                                break;
                            } else {
                                unreachable!("Inbalanced tree state");
                            }
                        } else {
                            // Rotate left
                            if get_node_sibling_rank(node, parent)
                                - get_node_sibling_right_rank(node, parent)
                                == 1
                            {
                                let s = get_node_sibling(node, parent);
                                if DEBUG {
                                    println!("Single left rotation");
                                }

                                if parent_right_rank == get_node_sibling_rank(node, parent) {
                                    let t = (*s).right;
                                    self.increase_rotations();
                                    self.rotate_left(parent);
                                    demote(parent);
                                    promote(s);
                                }

                                check_after_rotation(node, parent);
                                break;
                            } else if get_node_sibling_rank(node, parent)
                                - get_node_sibling_left_rank(node, parent)
                                == 1
                            {
                                if DEBUG {
                                    println!("Double right left rotation");
                                }
                                let s = get_node_sibling(node, parent);
                                if parent_right_rank == get_node_sibling_rank(node, parent) {
                                    let t = (*s).left;
                                    self.increase_rotations();
                                    self.rotate_right(s);
                                    demote(s);
                                    promote(t);

                                    self.increase_rotations();
                                    self.rotate_left(parent);
                                    demote(parent);
                                }

                                check_after_rotation(node, parent);
                                break;
                            } else {
                                unreachable!("Inbalanced tree state");
                            }
                        }

                        break;
                    }
                    break;
                }
                break;
            }
        }
    }

    fn rotate_left(&mut self, mut x: *mut Node) {
        unsafe {
            let mut y = (*x).right;
            (*x).right = (*y).left;

            if !(*y).left.is_null() {
                y.as_ref().unwrap().left.as_mut().unwrap().parent = x;
            }

            (*y).parent = (*x).parent;
            if (*x).parent.is_null() {
                self.root = y;
            } else if x == x.as_ref().unwrap().parent.as_ref().unwrap().left {
                x.as_ref().unwrap().parent.as_mut().unwrap().left = y;
            } else {
                x.as_ref().unwrap().parent.as_mut().unwrap().right = y;
            }

            (*y).left = x;
            (*x).parent = y;
        }
    }

    fn rotate_right(&mut self, x: *mut Node) {
        unsafe {
            let mut y = (*x).left;
            (*x).left = (*y).right;

            if !(*y).right.is_null() {
                y.as_ref().unwrap().right.as_mut().unwrap().parent = x;
            }
            (*y).parent = (*x).parent;

            if (*x).parent.is_null() {
                self.root = y;
            } else if x == x.as_ref().unwrap().parent.as_ref().unwrap().right {
                x.as_ref().unwrap().parent.as_mut().unwrap().right = y;
            } else {
                x.as_ref().unwrap().parent.as_mut().unwrap().left = y;
            }

            (*y).right = x;
            (*x).parent = y;
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        while !self.root.is_null() {
            self.remove_node(self.root, false);
        }
    }
}

impl Node {
    fn new(data: i32) -> *mut Self {
        Box::into_raw(Box::new(Self {
            data,
            rank: 1,
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
        }))
    }

    fn new_with_parent(data: i32, parent: *mut Node) -> *mut Self {
        Box::into_raw(Box::new(Self {
            data,
            rank: 1,
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
            parent,
        }))
    }
}

fn get_node_sibling(node: *mut Node, parent: *mut Node) -> *mut Node {
    unsafe {
        let parent_ref = if let Some(parent_ref) = parent.as_ref() {
            parent_ref
        } else {
            unreachable!("Parent node cannot be null");
        };

        if parent_ref.left.is_null() {
            return parent_ref.right;
        }

        if parent_ref.right.is_null() {
            return parent_ref.left;
        }

        let node_data = if let Some(node_ref) = node.as_ref() {
            node_ref.data
        } else {
            unreachable!("Node cannot be null");
        };

        if parent_ref.left.as_ref().unwrap().data == node_data {
            parent_ref.right
        } else {
            parent_ref.left
        }
    }
}

fn get_node_sibling_left_rank(node: *mut Node, parent: *mut Node) -> i32 {
    unsafe {
        let sibling = if let Some(sibling) = get_node_sibling(node, parent).as_ref() {
            sibling
        } else {
            unreachable!("Node cannot be null");
        };

        if (*sibling).left.is_null() {
            0
        } else {
            (*sibling).left.as_ref().unwrap().rank
        }
    }
}

fn get_node_sibling_right_rank(node: *mut Node, parent: *mut Node) -> i32 {
    unsafe {
        let sibling = if let Some(sibling) = get_node_sibling(node, parent).as_ref() {
            sibling
        } else {
            unreachable!("Node cannot be null");
        };

        if (*sibling).right.is_null() {
            0
        } else {
            (*sibling).right.as_ref().unwrap().rank
        }
    }
}

fn get_node_sibling_rank(node: *mut Node, parent: *mut Node) -> i32 {
    unsafe {
        let parent_ref = if let Some(parent_ref) = parent.as_ref() {
            parent_ref
        } else {
            unreachable!("Node cannot be null");
        };

        let parent_left_rank = if let Some(left) = parent_ref.left.as_ref() {
            left.rank
        } else {
            0
        };

        let parent_right_rank = if let Some(right) = parent_ref.right.as_ref() {
            right.rank
        } else {
            0
        };

        if (*parent).left == node {
            return parent_right_rank;
        } else {
            return parent_left_rank;
        }
    }
}

fn check_after_rotation(node: *mut Node, parent: *mut Node) {
    unsafe {
        let parent_left_rank = if let Some(l_ref) = (*parent).left.as_ref() {
            l_ref.rank
        } else {
            0
        };
        let parent_right_rank = if let Some(r_ref) = (*parent).right.as_ref() {
            r_ref.rank
        } else {
            0
        };

        if (*parent).rank - parent_left_rank == 2 && (*parent).rank - parent_right_rank == 2 {
            demote(parent);
        }
    }
}

fn promote(node: *mut Node) {
    unsafe {
        (*node).rank += 1;
    }
}

fn demote(node: *mut Node) {
    unsafe {
        (*node).rank -= 1;
    }
}

fn leftmost_child(node: *mut Node) -> *mut Node {
    unsafe {
        if (*node).left.is_null() {
            node
        } else {
            leftmost_child((*node).left)
        }
    }
}

fn rightmost_child(node: *mut Node) -> *mut Node {
    unsafe {
        if (*node).right.is_null() {
            node
        } else {
            rightmost_child((*node).right)
        }
    }
}

fn successor_of_node(node: *mut Node) -> *mut Node {
    unsafe {
        if !(*node).right.is_null() {
            leftmost_child((*node).right)
        } else {
            parent_with_left(node)
        }
    }
}

fn predecessor_of_node(node: *mut Node) -> *mut Node {
    unsafe {
        if !(*node).left.is_null() {
            rightmost_child((*node).left)
        } else {
            parent_with_right(node)
        }
    }
}

fn parent_with_left(node: *mut Node) -> *mut Node {
    unsafe {
        let parent = (*node).parent;
        if !parent.is_null() {
            if std::ptr::eq((*parent).left, node) {
                return parent;
            }
            return parent_with_left(parent);
        }

        std::ptr::null_mut()
    }
}

fn parent_with_right(node: *mut Node) -> *mut Node {
    unsafe {
        let parent = (*node).parent;
        if !parent.is_null() {
            if std::ptr::eq((*parent).right, node) {
                return parent;
            }
            return parent_with_right(parent);
        }

        std::ptr::null_mut()
    }
}

impl Node {
    fn dot_leaf(&self, leaf: *mut Node, c: &mut i32, nil: &mut Vec<i32>) {
        if leaf.is_null() {
            println!("null{} [shape=point];", c);
            println!("{} -> null{};", &self.data, c);
            nil.push(*c);
            *c += 1;
        } else {
            unsafe {
                println!(
                    "{} -> {} [label=\"{}\" style=\"filled\", fillcolor=\"lightblue\"]",
                    &self.data,
                    (*leaf).data,
                    self.rank - unsafe { (*leaf).rank },
                );
                leaf.as_ref().unwrap().dot(c, nil)
            }
        }
    }

    fn dot(&self, c: &mut i32, nil: &mut Vec<i32>) {
        self.dot_leaf(self.left, c, nil);
        self.dot_leaf(self.right, c, nil);
    }
}

impl Tree {
    fn dot(&self) {
        println!("digraph Tree {{subgraph tier1 {{node [color=\"lightblue\",style=\"filled\",group=\"tier1\"]");

        let mut c = 0i32;
        let mut nil = vec![];

        if !&self.root.is_null() {
            unsafe {
                (*self.root).dot(&mut c, &mut nil);
            }
        }

        let mut ranks: Vec<Vec<i32>> = vec![vec![]; 100];

        for (value, rank) in self.inorder() {
            ranks[rank as usize].push(value);
        }

        for rank in &ranks {
            let mut rank_string = String::new();

            for value in rank {
                rank_string += &format!(" {};", value);
            }

            if !rank.is_empty() {
                println!("{{rank = same;{}}}", rank_string);
            }
        }

        let mut rank_string = String::new();

        for value in nil {
            rank_string += &format!(" null{};", value);
        }

        if !rank_string.is_empty() {
            println!("{{rank = same;{}}}", rank_string);
        }

        println!("}}}}");
    }
}

const NODES_COUNT: usize = 1_000_000;

fn main() {
    /*let keys = vec![30];*/
    /*let keys = vec![30, 40];*/
    /*let keys = vec![30, 40, 50];*/
    /*let keys = vec![30, 40, 50, 24];*/
    /*let keys = vec![30, 40, 50, 24, 8];*/
    /*let keys = vec![30, 40, 50, 24, 8, 58];*/
    /*let keys = vec![30, 40, 50, 24, 8, 58, 48];*/
    /*let keys = vec![30, 40, 50, 24, 8, 58, 48, 26];*/
    //let keys = vec![30, 40, 50, 24, 8, 58, 48, 28, 11];
    /*let keys = vec![30, 40, 50, 24, 8, 58, 48, 28, 11, 13];*/

    /*let mut tree = Tree::new();*/
    /*for k in keys {*/
    /*tree.insert(k);*/
    /*}*/

    /*tree.remove(13);*/
    /*tree.remove(40);*/
    /*tree.remove(48);*/
    /*tree.dot();*/
    /*return;*/

    let mut keys = vec![];
    let mut tree = Tree::new();

    loop {
        let key: i32 = rand::thread_rng().gen();
        if tree.insert(key) {
            keys.push(key);
        }

        if tree.count == NODES_COUNT {
            break;
        }
    }

    println!("Tree count: {:?}", tree.count);

    let mut insertions_count = 0;
    let mut insertion_rotations = 0;
    let mut insertion_nodes = 0;
    let mut deletions_count = 0;
    let mut deletion_rotations = 0;
    let mut deletion_nodes = 0;
    let mut search_count = 0;
    let mut search_rotations = 0;
    let mut search_nodes = 0;

    for _ in 0..NODES_COUNT / 5 {
        let op_key: i32 = rand::thread_rng().gen_range(0..3);
        let index: usize = rand::thread_rng().gen_range(0..NODES_COUNT);
        let key_to_delete = keys[index as usize];
        let key_to_insert: i32 = rand::thread_rng().gen();

        if op_key == 0 {
            tree.reset_rotations();
            tree.reset_accessed_nodes();
            tree.insert(key_to_insert);
            insertions_count += 1;
            insertion_rotations += tree.rotations;
            insertion_nodes += tree.accessed_nodes;
        } else if op_key == 1 {
            tree.reset_rotations();
            tree.reset_accessed_nodes();
            tree.find(key_to_insert);
            search_count += 1;
            search_rotations += tree.rotations;
            search_nodes += tree.accessed_nodes;
        } else if op_key == 2 {
            tree.reset_rotations();
            tree.reset_accessed_nodes();
            tree.remove(key_to_delete);
            deletions_count += 1;
            deletion_rotations += tree.rotations;
            deletion_nodes += tree.accessed_nodes;
        }
    }

    if !tree.root.is_null() {
        unsafe {
            println!("Tree height: {:?}", (*tree.root).rank);
        }
    }
    println!(
        "Insertion rotations ({:?}): {:?}, nodes: {:?}",
        insertions_count, insertion_rotations, insertion_nodes
    );
    println!(
        "Deletion rotations ({:?}): {:?}, nodes: {:?}",
        deletions_count, deletion_rotations, deletion_nodes
    );
    println!(
        "Searches rotations ({:?}): {:?}, nodes: {:?}",
        search_count, search_rotations, search_nodes
    );

    println!("Tree count: {:?}", tree.count);
    //tree.dot();
}
