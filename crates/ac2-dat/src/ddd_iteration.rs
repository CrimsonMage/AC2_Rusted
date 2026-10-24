use crate::error::{DatError, DatResult};
use crate::iteration::{DatDddIterationList, DatDddIterationPair, RETAIL_DDD_ITERATION_DID};

pub(crate) fn parse_payload(bytes: &[u8]) -> DatResult<Option<DatDddIterationList>> {
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() < 4 {
        return Err(DatError::InvalidDddIterationData {
            did: RETAIL_DDD_ITERATION_DID,
            reason: "payload shorter than iteration header",
        });
    }
    let current_iteration = i32::from_le_bytes(bytes[0..4].try_into().unwrap());
    if current_iteration <= 0 {
        return Ok(None);
    }
    let mut remaining_iterations = current_iteration;
    let mut offset = 4usize;
    let mut pairs = Vec::new();
    while remaining_iterations > 0 {
        if offset + 8 > bytes.len() {
            return Err(DatError::InvalidDddIterationData {
                did: RETAIL_DDD_ITERATION_DID,
                reason: "payload ended before iteration token stream completed",
            });
        }
        let consecutive_iterations =
            i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let starting_iteration =
            i32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
        pairs.push(DatDddIterationPair {
            consecutive_iterations,
            starting_iteration,
        });
        remaining_iterations += consecutive_iterations;
        offset += 8;
    }
    if remaining_iterations != 0 {
        return Err(DatError::InvalidDddIterationData {
            did: RETAIL_DDD_ITERATION_DID,
            reason: "iteration token stream overran current iteration count",
        });
    }
    Ok(Some(DatDddIterationList {
        current_iteration,
        pairs,
    }))
}

#[cfg(test)]
mod tests {
    use super::parse_payload;

    #[test]
    fn parses_retail_shaped_ddd_iteration_payload() {
        let payload = [
            5, 0, 0, 0, 253, 255, 255, 255, 1, 0, 0, 0, 254, 255, 255, 255, 4, 0, 0, 0,
        ];
        let parsed = parse_payload(&payload)
            .expect("payload should parse")
            .expect("payload should exist");
        assert_eq!(parsed.current_iteration, 5);
        assert_eq!(parsed.mostly_consecutive_values(), vec![-3, 1, -2, 4]);
    }
}
