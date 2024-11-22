use std::collections::HashMap;
use crate::error::{AnalyzerError, AnalyzerErrorType};
use crate::parser::{Operand, Operation, Trace};

struct LockState<'a> {
    id: &'a str,
    acquirer: &'a str,
    locked: bool,
}

pub fn analyze_trace<'a>(trace: &'a Trace) -> Result<(), AnalyzerError<'a>> {
    let mut lock_map: HashMap<&str, LockState> = HashMap::new();
    let mut line = 1;

    for event in &trace.events {
        match event.operation {
            Operation::Acquire => {
                // 'acquire' operations only have 'lock_identifier' operands
                let lock_id = lock_id(&event.operand, &event.operation, line)?;

                if let Some(lock_state) = lock_map.get(lock_id) {
                    // repeated acquisition of the same lock
                    if lock_state.locked {
                        let error = AnalyzerError {
                            line,
                            error_type: AnalyzerErrorType::RepeatedAcquisition(lock_state.id, event.thread_identifier),
                        };
                        return Err(error);
                    }
                }

                let state = LockState {
                    id: lock_id,
                    locked: true,
                    acquirer: event.thread_identifier,
                };

                lock_map.insert(lock_id, state);
            }
            Operation::Release => {
                // 'release' operations only have 'lock_identifier' operands
                let lock_id = lock_id(&event.operand, &event.operation, line)?;
            }
            Operation::Read => {}
            Operation::Write => {}
            // other operations are not needed to check well-formedness
            _ => {}
        }

        line += 1;
    }
    Ok(())
}

fn lock_id<'a>(operand: &'a Operand, operation: &'a Operation, line: usize) -> Result<&'a str, AnalyzerError<'a>> {
    if let Operand::LockIdentifier(lock_identifier) = operand {
        Ok(lock_identifier)
    } else {
        let error = AnalyzerError {
            line,
            error_type: AnalyzerErrorType::MismatchedArguments(operation, &Operand::LockIdentifier("_")),
        };
        Err(error)
    }
}