use std::collections::HashMap;
use crate::error::{AnalyzerError, AnalyzerErrorType};
use crate::parser::{Operand, Operation, Program};

struct LockState<'a> {
    id: &'a str,
    acquirer: &'a str,
    locked: bool,
}

pub fn analyze_program<'a>(program: &'a Program) -> Result<(), AnalyzerError<'a>> {
    let mut lock_map: HashMap<&str, LockState> = HashMap::new();
    let mut line = 1;

    for trace in &program.traces {
        match trace.operation {
            Operation::Acquire => {
                // 'acquire' operation only have 'lock_identifier' operands
                let lock_id = if let Operand::LockIdentifier(lock_identifier) = trace.operand {
                    lock_identifier
                } else {
                    let error = AnalyzerError {
                        line,
                        error_type: AnalyzerErrorType::MismatchedOperation(&trace.operation, &Operand::LockIdentifier("_"))
                    };
                    return Err(error);
                };

                if let Some(lock_state) = lock_map.get(lock_id) {
                    // repeated acquisition of the same lock
                    if lock_state.locked {
                        let error = AnalyzerError {
                            line,
                            error_type: AnalyzerErrorType::RepeatedAcquisition(lock_state.id, trace.thread_identifier)
                        };
                        return Err(error);
                    }
                }

                let state = LockState {
                    id: lock_id,
                    locked: true,
                    acquirer: trace.thread_identifier,
                };

                lock_map.insert(lock_id, state);
            }
            Operation::Request => {}
            Operation::Release => {}
            _ => {},
        }

        line += 1;
    }
    Ok(())
}