pub fn estimate_cost(rule_depth: usize, batch_size: usize) -> f64 {
    let base_cost = 0.0001; // $0.0001 per base eval
    let depth_multiplier = 1.0 + (rule_depth as f64 * 0.1); // 10% more per depth level

    base_cost * depth_multiplier * (batch_size as f64)
}
