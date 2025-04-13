// ************************************************************************************************
// use
// ************************************************************************************************

use super::UnionFind;

// ************************************************************************************************
// struct
// ************************************************************************************************

impl UnionFind {
    // ********************************************************************************************
    // helper functions
    // ********************************************************************************************

    // ********************************************************************************************
    // API
    // ********************************************************************************************

    pub fn new(num_of_elements: usize) -> Self {
        let mut parent = vec![0; num_of_elements];
        let size = vec![1; num_of_elements];
        for (i, x) in parent.iter_mut().enumerate() {
            *x = i;
        }
        UnionFind {
            parent,
            size,
            count: num_of_elements,
        }
    }

    /// Time: O(log n) | Space: O(1)
    pub fn find(&mut self, mut node: usize) -> usize {
        while node != self.parent[node] {
            // path compression
            self.parent[node] = self.parent[self.parent[node]];
            node = self.parent[node];
        }
        node
    }

    /// Time: O(1) | Space: O(1)
    pub fn union(&mut self, node1: usize, node2: usize) {
        let root1 = self.find(node1);
        let root2 = self.find(node2);

        // already in the same set
        if root1 == root2 {
            return;
        }

        if self.size[root1] > self.size[root2] {
            self.parent[root2] = root1;
            self.size[root1] += 1;
        } else {
            self.parent[root1] = root2;
            self.size[root2] += 1;
        }

        self.count -= 1;
    }
}

// impl UnionFind {
//     fn new(num_of_elements: usize) -> Self {
//         let mut parent = vec![0; num_of_elements];
//         let size = vec![1; num_of_elements];
//         for i in 0..num_of_elements {
//             parent[i] = i;
//         }
//         UnionFind {
//             parent,
//             size,
//             count: num_of_elements,
//         }
//     }

//     // Time: O(log n) | Space: O(1)
//     fn find(&mut self, mut node: usize) -> usize {
//         while node != self.parent[node] {
//             // path compression
//             self.parent[node] = self.parent[self.parent[node]];
//             node = self.parent[node];
//         }
//         node
//     }

//     // Time: O(1) | Space: O(1)
//     fn union(&mut self, node1: usize, node2: usize) {
//         let root1 = self.find(node1);
//         let root2 = self.find(node2);

//         // already in the same set
//         if root1 == root2 {
//             return;
//         }

//         if self.size[root1] > self.size[root2] {
//             self.parent[root2] = root1;
//             self.size[root1] += 1;
//         } else {
//             self.parent[root1] = root2;
//             self.size[root2] += 1;
//         }

//         self.count -= 1;
//     }
// }
