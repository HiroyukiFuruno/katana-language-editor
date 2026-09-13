pub(crate) struct ActualRepoTaskStateAudit;

impl ActualRepoTaskStateAudit {
    pub(crate) fn validate_task_state_matches_evidence(
        tasks_document: &str,
        actual_evidence_present: bool,
        matrix_has_open_gaps: bool,
    ) -> Result<(), String> {
        let task_states = parse_task_states(tasks_document)?;

        if actual_evidence_present {
            return validate_evidence_task_states(&task_states, matrix_has_open_gaps);
        }

        validate_no_evidence_task_states(&task_states)
    }
}

fn validate_evidence_task_states(
    task_states: &TaskStates,
    matrix_has_open_gaps: bool,
) -> Result<(), String> {
    if task_states.task_12_4b_open() {
        return Err(
            "KatanA adapter task 12.4b must be completed in the same repo that has adapter evidence"
                .to_string(),
        );
    }
    if matrix_has_open_gaps && !task_states.task_12_4_open() {
        return Err("12.4 must remain open while the matrix has remaining gaps".to_string());
    }
    if !matrix_has_open_gaps && task_states.task_12_4_open() {
        return Err("12.4 should be closed once the matrix has no remaining gaps".to_string());
    }
    Ok(())
}

fn validate_no_evidence_task_states(task_states: &TaskStates) -> Result<(), String> {
    if !task_states.task_12_4_open() {
        return Err(
            "12.4 must be open when actual KatanA repo adapter evidence is not present".to_string(),
        );
    }
    if !task_states.task_12_4b_open() {
        return Err(
            "12.4b must be open when actual KatanA repo adapter evidence is not present"
                .to_string(),
        );
    }
    Ok(())
}

fn parse_task_states(tasks_document: &str) -> Result<TaskStates, String> {
    let mut states = TaskStates::default();
    for line in tasks_document.lines() {
        let Some(task) = parse_task_from_line(line)? else {
            continue;
        };
        match task.task_id.as_str() {
            "12.4" => states.record_12_4(task.checked)?,
            "12.4b" => states.record_12_4b(task.checked)?,
            _ => {}
        }
    }

    states.ensure_complete()?;
    Ok(states)
}

#[derive(Default)]
struct TaskStates {
    task_12_4_checked: Option<bool>,
    task_12_4b_checked: Option<bool>,
}

impl TaskStates {
    fn record_12_4(&mut self, checked: bool) -> Result<(), String> {
        record_task_state(
            &mut self.task_12_4_checked,
            checked,
            "tasks file has conflicting 12.4 checkbox states",
        )
    }

    fn record_12_4b(&mut self, checked: bool) -> Result<(), String> {
        record_task_state(
            &mut self.task_12_4b_checked,
            checked,
            "tasks file has conflicting 12.4b checkbox states",
        )
    }

    fn ensure_complete(&self) -> Result<(), String> {
        if self.task_12_4_checked.is_none() {
            return Err("tasks file is missing task 12.4".to_string());
        }
        if self.task_12_4b_checked.is_none() {
            return Err("tasks file is missing task 12.4b".to_string());
        }
        Ok(())
    }

    fn task_12_4_open(&self) -> bool {
        !self.task_12_4_checked.unwrap_or(false)
    }

    fn task_12_4b_open(&self) -> bool {
        !self.task_12_4b_checked.unwrap_or(false)
    }
}

fn record_task_state(
    current: &mut Option<bool>,
    checked: bool,
    conflict_message: &str,
) -> Result<(), String> {
    if current.is_some_and(|existing| existing != checked) {
        return Err(conflict_message.to_string());
    }
    *current = Some(checked);
    Ok(())
}

struct ParsedTask {
    task_id: String,
    checked: bool,
}

fn parse_task_from_line(line: &str) -> Result<Option<ParsedTask>, String> {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix("- [") else {
        return Ok(None);
    };
    let Some((checkbox, after_checkbox)) = rest.split_once(']') else {
        return Ok(None);
    };

    let checked = match checkbox {
        "x" | "X" => true,
        " " => false,
        _ => return Ok(None),
    };
    let Some(raw_task_id) = after_checkbox.split_whitespace().next() else {
        return Ok(None);
    };
    let Some(task_id) = parse_task_id(raw_task_id) else {
        return Ok(None);
    };

    Ok(Some(ParsedTask { task_id, checked }))
}

fn parse_task_id(raw_task_id: &str) -> Option<String> {
    let token = raw_task_id
        .trim()
        .trim_end_matches(':')
        .trim_end_matches(')')
        .trim_end_matches('.')
        .trim_matches('`');

    match token {
        "12.4" => Some("12.4".to_string()),
        "12.4b" => Some("12.4b".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_task_states_reads_checkbox_state_regardless_of_task_order() -> Result<(), String> {
        let tasks = "- [ ] 12.4 Some text\n- [x] 12.4b Another\n";
        let states = parse_task_states(tasks)?;
        assert!(states.task_12_4_open());
        assert!(!states.task_12_4b_open());
        Ok(())
    }

    #[test]
    fn parse_task_states_requires_both_tasks() {
        let tasks = "- [ ] 12.4 only\n";
        assert!(parse_task_states(tasks).is_err());
    }

    #[test]
    fn validate_no_evidence_task_states_accepts_open_states() -> Result<(), String> {
        let task_states = TaskStates {
            task_12_4_checked: Some(false),
            task_12_4b_checked: Some(false),
        };
        validate_no_evidence_task_states(&task_states)
    }

    #[test]
    fn validate_no_evidence_task_states_rejects_closed_12_4() {
        let task_states = TaskStates {
            task_12_4_checked: Some(true),
            task_12_4b_checked: Some(false),
        };
        assert!(matches!(
            validate_no_evidence_task_states(&task_states).as_ref(),
            Err(error) if error.contains("12.4 must be open")
        ));
    }

    #[test]
    fn validate_no_evidence_task_states_rejects_closed_12_4b() {
        let task_states = TaskStates {
            task_12_4_checked: Some(false),
            task_12_4b_checked: Some(true),
        };
        assert!(matches!(
            validate_no_evidence_task_states(&task_states).as_ref(),
            Err(error) if error.contains("12.4b must be open")
        ));
    }

    #[test]
    fn validate_evidence_task_states_rejects_open_12_4b() {
        let task_states = TaskStates {
            task_12_4_checked: Some(false),
            task_12_4b_checked: Some(false),
        };
        assert!(matches!(
            validate_evidence_task_states(&task_states, false).as_ref(),
            Err(error) if error.contains("12.4b must be completed")
        ));
    }

    #[test]
    fn validate_evidence_task_states_accepts_open_12_4_when_gaps_exist() -> Result<(), String> {
        let task_states = TaskStates {
            task_12_4_checked: Some(false),
            task_12_4b_checked: Some(true),
        };
        validate_evidence_task_states(&task_states, true)
    }

    #[test]
    fn validate_evidence_task_states_rejects_closed_12_4_when_gaps_exist() {
        let task_states = TaskStates {
            task_12_4_checked: Some(true),
            task_12_4b_checked: Some(true),
        };
        assert!(matches!(
            validate_evidence_task_states(&task_states, true).as_ref(),
            Err(error) if error.contains("12.4 must remain open")
        ));
    }

    #[test]
    fn validate_evidence_task_states_rejects_open_12_4_when_no_gaps() {
        let task_states = TaskStates {
            task_12_4_checked: Some(false),
            task_12_4b_checked: Some(true),
        };
        assert!(matches!(
            validate_evidence_task_states(&task_states, false).as_ref(),
            Err(error) if error.contains("12.4 should be closed")
        ));
    }

    #[test]
    fn validate_evidence_task_states_accepts_closed_12_4_when_no_gaps() -> Result<(), String> {
        let task_states = TaskStates {
            task_12_4_checked: Some(true),
            task_12_4b_checked: Some(true),
        };
        validate_evidence_task_states(&task_states, false)
    }
}
