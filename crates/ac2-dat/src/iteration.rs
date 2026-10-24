pub const RETAIL_DDD_ITERATION_DID: u32 = 0xFFFF_0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatDddIterationPair {
    pub consecutive_iterations: i32,
    pub starting_iteration: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatDddIterationList {
    pub current_iteration: i32,
    pub pairs: Vec<DatDddIterationPair>,
}

impl DatDddIterationList {
    pub fn mostly_consecutive_values(&self) -> Vec<i32> {
        let mut values = Vec::with_capacity(self.pairs.len() * 2);
        for pair in &self.pairs {
            values.push(pair.consecutive_iterations);
            values.push(pair.starting_iteration);
        }
        values
    }
}
