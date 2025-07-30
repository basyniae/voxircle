use std::collections::HashSet;

/// Sparse graph data structure. Size in memory is O(n) where n is the number of edges
/// Undirected, at most single edge between points, self-loops allowed
/// Vertices are labelled 0, 1, ... nr_verts - 1.
#[derive(Default)]
pub struct SparseGraph {
    nr_verts: usize,
    edges: Vec<[usize; 2]>, // order of the edges does not matter
}

impl SparseGraph {
    pub fn new(nr_verts: usize, edges: Vec<[usize; 2]>) -> Self {
        SparseGraph { nr_verts, edges }
    }

    /// O(1)
    pub fn get_nr_verts(&self) -> usize {
        self.nr_verts
    }

    /// O(1)
    pub fn get_edges(&self) -> &Vec<[usize; 2]> {
        &self.edges
    }

    /// O(1)
    pub fn get_edges_mut(&mut self) -> &mut Vec<[usize; 2]> {
        &mut self.edges
    }

    /// O(n) where n is the number of edges
    /// Note that i is its own neighbor if we have a self loop at i
    fn get_neighbors(&self, i: &usize) -> Vec<usize> {
        let mut running_neighbors = vec![];
        for edge in self.edges.iter() {
            if edge.contains(i) {
                let [a, b] = edge;
                if a == i {
                    running_neighbors.push(*b)
                } else {
                    running_neighbors.push(*a)
                }
            }
        }

        running_neighbors
    }

    /// O(n) where n is the number of edges
    fn get_degree(&self, i: &usize) -> usize {
        self.get_neighbors(i).len()
    }
}

impl SparseGraph {
    /// Find a maximal length cycle in the graph
    pub fn longest_cycle(&self) -> Vec<usize> {
        let mut record_cycle = self.longest_cycle_starting_at(&0);
        let mut record_length = record_cycle.len();

        // loop over all vertices to visit
        for i in 0..self.nr_verts {
            // degree-1 vertices can never be part of a strictly longer cycle than the trivial one
            if self.get_degree(&i) > 1 {
                let path = self.longest_cycle_starting_at(&i);
                if path.len() > record_length {
                    record_length = path.len();
                    record_cycle = path;
                }
            }
        }

        record_cycle
    }

    fn longest_cycle_starting_at(&self, start: &usize) -> Vec<usize> {
        let neighs_of_start: Vec<usize> = self.get_neighbors(start);

        let cond = |p: &Vec<usize>| -> bool {
            if let Some(x) = p.last() {
                neighs_of_start.contains(x)
            } else {
                false
            }
        };

        let res = self.longest_path_from_start(start, &cond);

        // edge case handling, loops of length 2 require a double edge, or no edges at the start
        if let Some((path, len)) = res {
            if len <= 2 {
                vec![*start]
            } else {
                path
            }
        } else {
            vec![*start]
        }
    }

    /// Exhaustively search for the path satisfying path_condition which is as long as possible
    /// (if any exists)
    /// Variant of depth-first search
    fn longest_path_from_start(
        &self,
        start: &usize,
        path_condition: &impl Fn(&Vec<usize>) -> bool,
    ) -> Option<(Vec<usize>, usize)> {
        // return the longest path length
        fn longest_path_recur(
            graph: &SparseGraph,
            current: &usize,
            visited: Vec<usize>,
            path_condition: &impl Fn(&Vec<usize>) -> bool,
        ) -> Option<(Vec<usize>, usize)> {
            let mut visited = visited.clone();
            // println!("Current vertex: {current}, visited: {visited:?}");

            visited.push(*current);
            let mut running_max = visited.len();
            let mut best_path = visited.clone();
            for neigh in graph.get_neighbors(current).iter() {
                // println!("visiting neighbor {neigh} of {current}");
                if !visited.contains(neigh) {
                    // println!("this neighbor hasn't been visited yet");
                    if let Some((x, y)) =
                        longest_path_recur(graph, neigh, visited.clone(), path_condition)
                    {
                        if y > running_max && path_condition(&x) {
                            running_max = y;
                            best_path = x;
                        }
                    }
                }
            }

            Some((best_path, running_max))
        }

        longest_path_recur(self, start, vec![], path_condition)
    }

    /// Depth-first search, return the first (in visiting order) that satisfies the input predicate p
    /// Search starts at the start vector and goes only through its connected component
    fn dfs(&self, start: &usize, p: fn(&usize) -> bool) -> Option<usize> {
        fn dfs_recur(
            graph: &SparseGraph,
            current: &usize,
            visited: HashSet<usize>,
            p: fn(&usize) -> bool,
        ) -> Option<usize> {
            let mut visited = visited.clone();
            // println!("Current vertex: {current}, visited: {visited:?}");
            if p(current) {
                return Some(*current);
            }

            visited.insert(*current);
            for neigh in graph.get_neighbors(current).iter() {
                // println!("visiting neighbor {neigh} of {current}");
                if !visited.contains(neigh) {
                    // println!("this neighbor hasn't been visited yet");
                    if let Some(x) = dfs_recur(graph, neigh, visited.clone(), p) {
                        return Some(x);
                    }
                }
            }
            None
        }

        // println!("started dfs");
        dfs_recur(self, start, HashSet::new(), p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dfs_test() {
        let graph = SparseGraph {
            nr_verts: 5,
            edges: vec![[0, 1], [0, 2], [0, 3], [2, 3], [2, 4]],
        };

        println!("{:?}", graph.dfs(&0, |i| false));
    }

    #[test]
    fn longest_path_from_test() {
        let graph = SparseGraph {
            nr_verts: 5,
            edges: vec![[0, 1], [0, 2], [0, 3], [2, 3], [2, 4]],
        };

        println!("{:?}", graph.longest_path_from_start(&0, &|_| true));
        assert_eq!(
            graph.longest_path_from_start(&0, &|_| true).unwrap(),
            (vec![0, 3, 2, 4], 4)
        );

        println!();

        println!("Longest odd-length path");
        println!(
            "{:?}",
            graph.longest_path_from_start(&0, &|p| p.len() % 2 == 1)
        );

        assert_eq!(
            graph
                .longest_path_from_start(&0, &|p| p.len() % 2 == 1)
                .unwrap(),
            (vec![0, 2, 3,], 3)
        );

        println!();

        println!("Longest path ending in 2");
        println!(
            "{:?}",
            graph.longest_path_from_start(&0, &|p| p.last().is_some_and(|i| *i == 2))
        );
        assert_eq!(
            graph
                .longest_path_from_start(&0, &|p| p.last().is_some_and(|i| *i == 2))
                .unwrap(),
            (vec![0, 3, 2], 3)
        );
    }

    #[test]
    fn longest_cycle_test() {
        let graph = SparseGraph {
            nr_verts: 5,
            edges: vec![[0, 1], [0, 2], [0, 3], [2, 3], [2, 4]],
        };

        println!("{:?}", graph.longest_cycle_starting_at(&4));
        assert_eq!(graph.longest_cycle_starting_at(&4), vec![4]); // edge point
        assert_eq!(graph.longest_cycle_starting_at(&132), vec![132]); // outside of graph, no edges
        assert_eq!(graph.longest_cycle_starting_at(&0), vec![0, 2, 3]); // note ascending order
        assert_eq!(graph.longest_cycle_starting_at(&2), vec![2, 0, 3]);

        // basepoint-independent longest cycle
        assert_eq!(graph.longest_cycle(), vec![0, 2, 3]);

        let graph = SparseGraph {
            nr_verts: 6,
            edges: vec![[0, 1], [1, 2], [2, 3], [3, 4], [4, 5]],
        };
        assert_eq!(graph.longest_cycle(), vec![0]);

        // circle graph with attachment
        let graph = SparseGraph {
            nr_verts: 8,
            edges: vec![
                [0, 1],
                [1, 2],
                [2, 3],
                [3, 4],
                [4, 5],
                [5, 0], // end of circle (size 6)
                [0, 6],
                [6, 7],
            ],
        };
        assert_eq!(graph.longest_cycle(), vec![0, 1, 2, 3, 4, 5]);

        // figure 8 graph (6 nodes)
        let graph = SparseGraph {
            nr_verts: 6,
            edges: vec![[0, 1], [1, 2], [3, 4], [4, 5], [0, 3], [1, 4], [2, 5]],
        };
        assert_eq!(graph.longest_cycle(), vec![0, 1, 2, 5, 4, 3]);
    }
}
