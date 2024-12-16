/// Sparse representation of a blocks object (that forgets about the origin)
/// `indices` contains x iff there is a block at index x
/// todo: should index be usize or [usize;2]? if the latter, forget about grid_size also
struct SparseBlocks {
    indices: Vec<usize>,
    grid_size: usize
}

// todo: functions for getting connected components,
//  another struct which functions as a pattern to match against (SparseBlocks without grid size?)
//  routine function which uses this to create a building thing... 
//  (but must not forget placement on grid!)