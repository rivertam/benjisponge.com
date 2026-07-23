//! The set-log effort score.
//!
//! Deliberately an effort score, not load x reps. This is the single copy:
//! the calendar heatmap's daily totals and the per-set badges must stay in
//! exact lockstep (it used to be mirrored between the Worker's SQL and two
//! Rust view helpers).

pub fn effort_points(effort_hundredths: Option<u64>) -> u32 {
    match effort_hundredths {
        Some(1000) => 5,
        Some(900) => 4,
        Some(800) => 3,
        _ => 2,
    }
}

pub fn set_volume_points(set_type: &str, effort_hundredths: Option<u64>) -> u32 {
    match set_type {
        "FAILURE_SET" => 6,
        "WARMUP_SET" => 0,
        _ => effort_points(effort_hundredths),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_documented_scale() {
        assert_eq!(set_volume_points("FAILURE_SET", Some(500)), 6);
        assert_eq!(set_volume_points("WARMUP_SET", Some(1000)), 0);
        assert_eq!(set_volume_points("NORMAL_SET", Some(1000)), 5);
        assert_eq!(set_volume_points("NORMAL_SET", Some(900)), 4);
        assert_eq!(set_volume_points("NORMAL_SET", Some(800)), 3);
        assert_eq!(set_volume_points("NORMAL_SET", Some(750)), 2);
        assert_eq!(set_volume_points("NORMAL_SET", None), 2);
        assert_eq!(set_volume_points("DROP_SET", Some(1000)), 5);
    }
}
