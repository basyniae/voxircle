pub fn format_block_count(nr_blocks: u64) -> String {
    if nr_blocks <= 64 {
        format!("{}", nr_blocks)
    } else {
        format!(
            "{} = {}s{}",
            nr_blocks,
            nr_blocks.div_euclid(64),
            nr_blocks.rem_euclid(64)
        )
    }
}

pub fn format_block_diameter(diameters: [u64; 2]) -> String {
    if diameters[0] == diameters[1] {
        format!("block diameter: {}", diameters[0])
    } else {
        format!("block diameters: {}x by {}y", diameters[0], diameters[1])
    }
}
