use super::*;

pub fn format_score(score: i64) -> String {
    if score > MATE_EVAL - 1000 {
        let moves_to_mate = (MATE_EVAL - score + 1) / 2;
        format!("mate {}", moves_to_mate)
    } else if score < -MATE_EVAL + 1000 {
        let moves_to_mate = (-MATE_EVAL - score - 1) / 2;
        format!("mate {}", moves_to_mate)
    } else {
        format!("cp {}", score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mate_score_formatting() {
        assert_eq!(format_score(MATE_EVAL - 1), "mate 1");
        assert_eq!(format_score(MATE_EVAL - 2), "mate 1");
        assert_eq!(format_score(MATE_EVAL - 3), "mate 2");

        assert_eq!(format_score(-MATE_EVAL + 1), "mate -1");
        assert_eq!(format_score(-MATE_EVAL + 2), "mate -1");
        assert_eq!(format_score(-MATE_EVAL + 3), "mate -2");

        assert_eq!(format_score(150), "cp 150");
        assert_eq!(format_score(-200), "cp -200");
        assert_eq!(format_score(0), "cp 0");
    }
}
