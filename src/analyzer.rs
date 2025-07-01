use crate::arguments::Arguments;
use crate::error::AnalyzerError;
use crate::lexer::tokenize_source;
use crate::parser::{parse_event, Event, Operand, Operation};
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

struct Lock {
    owner: Option<i64>,
    locked: bool,
    row: usize,
}

// used for the GraphViz representation
#[derive(Eq, Hash, PartialEq)]
struct Edge {
    from: i64,
    to: i64,
}

// used for the GraphViz representation
type Graph = HashMap<i64, HashSet<i64>>;

#[derive(Eq, PartialEq, Debug, Clone)]
struct LockDependency {
    thread_id: i64,
    lock_id: i64,
    acquired_locks: HashSet<i64>,
    line: usize,
}

impl LockDependency {
    fn remove_lock(&mut self, lock_id: i64) {
        self.acquired_locks.remove(&lock_id);
    }
}

/// Analyzes a trace for well-formedness
///
/// # Arguments
///
/// * `arguments`: the command line arguments
///
/// returns: Result<(), Vec<AnalyzerError, Global>> Unit if the trace is well-formed, otherwise a vector containing the violations
///
pub fn analyze_trace(arguments: &Arguments) -> Result<(), Vec<AnalyzerError>> {
    // store trace violations
    let mut errors: Vec<AnalyzerError> = Vec::new();
    let mut locks: HashMap<i64, Lock> = HashMap::new();
    let row = 1;

    let file_handle = match File::open(&arguments.input) {
        Ok(file_handle) => file_handle,
        Err(err) => {
            errors.push(AnalyzerError::from(err));
            return Err(errors);
        }
    };

    // stream content of file to avoid OOM
    let mut trace_reader = BufReader::new(file_handle);

    let file_extension = Path::new(&arguments.input)
        .extension()
        .and_then(|ext| ext.to_str());

    // create graphviz representation
    let mut graphviz_locks = String::new(); // create graphical representation of the relation between the locks of a trace
    let mut graphviz_threads = String::new(); // create graphical representation of the relation between the threads of a trace

    let mut lockgraph = HashSet::<Edge>::new();
    let mut lock_dependencies = Vec::<LockDependency>::new();

    if &arguments.graph == &true {
        writeln!(&mut graphviz_locks, "digraph G {{").unwrap();
    }

    // analyze either a STD or RapidBin trace
    match file_extension {
        Some("std") => analyze_std_trace(
            &arguments,
            &mut trace_reader,
            &mut errors,
            &mut locks,
            row,
            &mut lockgraph,
            &mut lock_dependencies,
        ),
        Some("data") => analyze_rapid_trace(
            &arguments,
            &mut trace_reader,
            &mut errors,
            &mut locks,
            row,
            &mut lockgraph,
            &mut lock_dependencies,
        ),
        _ => errors.push(AnalyzerError::UnsupportedFileExtension),
    }

    if &arguments.graph == &true {
        for entry in lockgraph.drain() {
            writeln!(&mut graphviz_locks, "    L{} -> L{};", entry.from, entry.to).unwrap();
        }

        writeln!(&mut graphviz_locks, "}}").unwrap();

        match fs::create_dir_all("output") {
            Ok(()) => {
                let mut file = File::create("output/graphviz_locks.txt").unwrap();
                file.write_all(graphviz_locks.as_bytes()).unwrap();
            },
            Err(e) => 
                eprintln!("Failed to create directory {:?}: {}", "output", e)
        }
    }

    if &arguments.lock_dependencies == &true {
        writeln!(&mut graphviz_threads, "digraph G {{").unwrap();

        let mut graph = HashMap::<i64, HashSet<i64>>::new();

        // clean lock dependencies to rule out false positives (like a cycle where the locks are owned be the identical thread)
        for entry in &lock_dependencies {
            let children = lock_dependencies
                .iter()
                .filter(|other| {
                    other.thread_id != entry.thread_id
                        && other
                            .acquired_locks
                            .intersection(&entry.acquired_locks)
                            .count()
                            == 0
                }) // no guard locks
                .filter(|other| other.acquired_locks.contains(&entry.lock_id))
                .map(|dependency| dependency.thread_id) // check for chain
                .collect::<HashSet<_>>();

            // save information in GraphViz syntax
            for child in children {
                add_edge(&mut graph, entry.thread_id, child);
                writeln!(
                    &mut graphviz_threads,
                    "    T{} -> T{};",
                    entry.thread_id, child
                )
                .unwrap();
            }

            debug!("{:?}", entry);
        }

        writeln!(&mut graphviz_threads, "}}").unwrap();

        let result = validate_dependency_graph(graph);

        info!("{:?} deadlocks were identified", result);

        match fs::create_dir_all("output") {
            Ok(()) => {
                let mut file2 = File::create("output/graphviz_threads.txt").unwrap();
                file2.write_all(graphviz_threads.as_bytes()).unwrap();
            },
            Err(e) => 
                eprintln!("Failed to create directory {:?}: {}", "output", e)
        }
    }

    if errors.is_empty() {
        return Ok(());
    }

    Err(errors)
}

/// Helper function to create a graph structure to represent a trace in GraphViz format
///
/// # Arguments
///
/// * `graph`: the current graph structure of a trace
/// * `from`: the origin of an edge
/// * `to`: the target of an edge
///
/// returns: ()
///
fn add_edge(graph: &mut Graph, from: i64, to: i64) {
    graph.entry(from).or_insert_with(HashSet::new).insert(to);
}

/// Analyzes a trace written in STD format
///
/// # Arguments
///
/// * `arguments`: the command line arguments
/// * `trace_reader`: a buffered reader containing the contents of a STD trace
/// * `errors`: a vector containing the errors the analyzer encountered
/// * `locks`: a vector containing all locks of the trace
/// * `row`: the current row of the trace
/// * `graphviz`: a hashset containing edges for the GraphViz representation of a trace
/// * `lock_dependencies`: a vector containing the lock dependencies of a trace
///
/// returns: () unit
///
fn analyze_std_trace(
    arguments: &Arguments,
    trace_reader: &mut BufReader<File>,
    errors: &mut Vec<AnalyzerError>,
    locks: &mut HashMap<i64, Lock>,
    mut row: usize,
    graphviz: &mut HashSet<Edge>,
    lock_dependencies: &mut Vec<LockDependency>,
) {
    for line in trace_reader.lines() {
        let line = match line.map_err(AnalyzerError::from) {
            Ok(line) => line,
            Err(err) => return errors.push(AnalyzerError::from(err)),
        };

        let tokens = match tokenize_source(line, arguments.normalize) {
            Ok(tokens) => tokens,
            Err(err) => return errors.push(AnalyzerError::from(err)),
        };

        let event = match parse_event(tokens) {
            Ok(event) => event,
            Err(err) => return errors.push(AnalyzerError::from(err)),
        };

        match analyze_event(arguments, event, locks, row, graphviz, lock_dependencies) {
            Ok(_) => {}
            Err(error) => {
                errors.push(error);
            }
        }

        row += 1;
    }
}

const NUM_THREADS_MASK: i16 = 0x7FFF;
const NUM_LOCKS_MASK: i32 = 0x7FFFFFFF;
const NUM_VARS_MASK: i32 = 0x7FFFFFFF;
const NUM_EVENTS_MASK: i64 = 0x7FFFFFFFFFFFFFFF;

const NUM_THREAD_BITS: i16 = 10;
const THREAD_BITS_OFFSET: i16 = 0;
const NUM_OPERATION_BITS: i16 = 4;
const OPERATION_BITS_OFFSET: i16 = THREAD_BITS_OFFSET;
const NUM_OPERAND_BITS: i16 = 34;
const OPERAND_BITS_OFFSET: i16 = NUM_THREAD_BITS + NUM_OPERATION_BITS;
const NUM_LOCATION_BITS: i16 = 15;
const LOCATION_BITS_OFFSET: i16 = NUM_THREAD_BITS + NUM_OPERATION_BITS + NUM_OPERAND_BITS;

const THREAD_MASK: i64 = ((1 << NUM_THREAD_BITS) - 1) << THREAD_BITS_OFFSET;
const OPERATION_MASK: i64 = ((1 << NUM_OPERATION_BITS) - 1) << OPERATION_BITS_OFFSET;
const OPERAND_MASK: i64 = ((1 << NUM_OPERAND_BITS) - 1) << OPERATION_BITS_OFFSET;
const LOCATION_MASK: i64 = ((1 << NUM_LOCATION_BITS) - 1) << LOCATION_BITS_OFFSET;

/// Parses a trace written in RapidBin format
///
/// # Arguments
///
/// * `arguments`:  the command line arguments
/// * `trace_reader`:  the reader containing the contents of a RapidBin file
/// * `errors`: a vector containing the errors the analyzer encountered
/// * `locks`: a vector containing all locks of the trace
/// * `row`: the current row of the trace
/// * `graphviz`: a hashset containing edges for the GraphViz representation of a trace
/// * `lock_dependencies`: a vector containing the lock dependencies of a trace
///
/// returns: () unit
///
fn analyze_rapid_trace(
    arguments: &Arguments,
    trace_reader: &mut BufReader<File>,
    errors: &mut Vec<AnalyzerError>,
    locks: &mut HashMap<i64, Lock>,
    mut row: usize,
    graphviz: &mut HashSet<Edge>,
    lock_dependencies: &mut Vec<LockDependency>,
) {
    parse_trace_header(trace_reader);

    let mut event_buffer = [0u8; 8];

    while trace_reader.read_exact(&mut event_buffer).is_ok() {
        let event = match try_parse_event(event_buffer) {
            None => continue,
            Some(event) => event,
        };

        match analyze_event(arguments, event, locks, row, graphviz, lock_dependencies) {
            Ok(_) => {}
            Err(error) => errors.push(error),
        }

        row += 1;
    }
}

/// Tries to parse the header of a RapidBin file which contains information about the amount of threads, locks, variables and events of a trace
///
/// # Arguments
///
/// * `trace_reader`: the reader containing the contents of RapidBin file
///
/// returns: () unit since it just consumes the buffered reader
///
fn parse_trace_header(trace_reader: &mut BufReader<File>) {
    let mut short_buffer = [0u8; 2];
    let mut integer_buffer = [0u8; 4];
    let mut long_buffer = [0u8; 8];

    trace_reader.read_exact(&mut short_buffer).unwrap();
    let num_threads = i16::from_be_bytes(short_buffer) & NUM_THREADS_MASK;

    trace_reader.read_exact(&mut integer_buffer).unwrap();
    let num_locks = i32::from_be_bytes(integer_buffer) & NUM_LOCKS_MASK;

    trace_reader.read_exact(&mut integer_buffer).unwrap();
    let num_variables = i32::from_be_bytes(integer_buffer) & NUM_VARS_MASK;

    trace_reader.read_exact(&mut long_buffer).unwrap();
    let num_events = i64::from_be_bytes(long_buffer) & NUM_EVENTS_MASK;

    info!("NUM_THREADS: {}", num_threads);
    info!("NUM_LOCKS: {}", num_locks);
    info!("NUM_VARIABLES: {}", num_variables);
    info!("NUM_EVENTS: {}", num_events);
}

/// Tries to parse an event in RapidBin format
///
/// # Arguments
///
/// * `event_buffer`: the buffer containing the bytes of a RapidBin event
///
/// returns: Option<Event> an event if it was successfully parsed, None otherwise
///
fn try_parse_event(event_buffer: [u8; 8]) -> Option<Event> {
    let raw_event = i64::from_be_bytes(event_buffer);

    let thread_identifier = (raw_event & THREAD_MASK) >> THREAD_BITS_OFFSET;
    let operation_id = (raw_event & OPERATION_MASK) >> OPERATION_BITS_OFFSET;
    let operand_id = (raw_event & OPERAND_MASK) >> OPERAND_BITS_OFFSET;
    let loc = (raw_event & LOCATION_MASK) >> LOCATION_BITS_OFFSET;

    let operation = match Operation::new(operation_id) {
        None => return None,
        Some(operation) => operation,
    };

    let operand = Operand::new(&operation, operand_id);

    let event = Event {
        thread_identifier,
        operation,
        operand,
        loc,
    };

    debug!("{:?}", event);

    Some(event)
}

/// Analyzes a single event of a trace
///
/// # Arguments
///
/// * `arguments`: the command line arguments
/// * `event`: the to be analyzed event
/// * `locks`: a hashmap containing all locks of a trace
/// * `line`: the current line of the trace
/// * `graphviz`: a hashset containing edges for the GraphViz representation of a trace
/// * `lock_dependencies`: a vector containing all lock dependencies of a trace
///
/// returns: Result<(), AnalyzerError> unit if the event doesn't violate well-formedness, an error otherwise
///
fn analyze_event(
    arguments: &Arguments,
    event: Event,
    locks: &mut HashMap<i64, Lock>,
    line: usize,
    graphviz: &mut HashSet<Edge>,
    lock_dependencies: &mut Vec<LockDependency>,
) -> Result<(), AnalyzerError> {
    match event.operation {
        Operation::Acquire => {
            let lock_id = event.operand.id().unwrap();
            let thread_owned_locks = locks_of_thread(event.thread_identifier, locks);

            if &arguments.lock_dependencies == &true {
                let acquired_locks = if thread_owned_locks.len() > 0 {
                    thread_owned_locks.clone()
                } else {
                    HashSet::new()
                };

                let existing = lock_dependencies.iter().find(|dependency| {
                    dependency.thread_id == event.thread_identifier
                        && dependency.lock_id == lock_id
                        && dependency.acquired_locks == thread_owned_locks
                });

                if existing.is_none() {
                    let lock_dependency = LockDependency {
                        thread_id: event.thread_identifier,
                        lock_id,
                        acquired_locks,
                        line,
                    };

                    lock_dependencies.push(lock_dependency);
                }
            }

            if &arguments.graph == &true {
                for owned_lock in thread_owned_locks {
                    graphviz.insert(Edge {
                        from: owned_lock,
                        to: lock_id,
                    });
                }
            }

            if let Some(lock) = locks.get(&lock_id) {
                if lock.locked && lock.owner.unwrap() != event.thread_identifier {
                    return Err(AnalyzerError::RepeatedAcquisition {
                        lock_id,
                        thread_id: event.thread_identifier,
                        owner_id: lock.owner.unwrap(),
                        row: line,
                    });
                }
            }

            let lock = Lock {
                owner: Some(event.thread_identifier),
                locked: true,
                row: line,
            };

            locks.insert(lock_id, lock);
            debug!(
                "Thread 'T{}' acquired lock 'L{lock_id}' in line {line}",
                event.thread_identifier
            );
        }
        Operation::Release => {
            let lock_id = event.operand.id().unwrap();

            if &arguments.lock_dependencies == &true {
                if let Some(lock_dependency) =
                    lock_dependency_of_thread(event.thread_identifier, lock_dependencies)
                {
                    lock_dependency.clone().remove_lock(lock_id);
                }
            }

            match locks.get(&lock_id) {
                None => {
                    return Err(AnalyzerError::ReleasedNonAcquiredLock {
                        row: line,
                        lock_id,
                        thread_id: event.thread_identifier,
                    });
                }
                Some(lock) => {
                    if !lock.locked {
                        return Err(AnalyzerError::RepeatedRelease {
                            attempted: line,
                            previous: lock.row,
                            lock_id,
                            thread_id: event.thread_identifier,
                        });
                    }

                    if let Some(owner) = lock.owner {
                        if owner != event.thread_identifier {
                            return Err(AnalyzerError::ReleasedNonOwningLock {
                                row: line,
                                lock_id,
                                thread_id: event.thread_identifier,
                                owner,
                            });
                        }
                    }

                    let updated_lock = Lock {
                        locked: false,
                        owner: None,
                        row: line,
                    };

                    locks.insert(lock_id, updated_lock);

                    debug!(
                        "Thread 'T{}' released lock 'L{lock_id}' in line {line}",
                        event.thread_identifier
                    );
                }
            }
        }
        // other operations are not needed to check well-formedness
        _ => {}
    }

    Ok(())
}

/// Returns all owned locks of a given thread
///
/// # Arguments
///
/// * `thread_id`: the id of the to be searched thread
/// * `locks`: a vector containing all locks of a trace
///
/// returns: HashSet<i64> a HashSet containing all ids of the locks the thread owns
///
fn locks_of_thread(thread_id: i64, locks: &mut HashMap<i64, Lock>) -> HashSet<i64> {
    let thread_owned_locks = locks
        .iter()
        .filter(|(_, lock)| lock.owner.is_some() && lock.owner.unwrap() == thread_id)
        .map(|(id, _)| *id)
        .collect();

    thread_owned_locks
}

/// Returns the lock dependency of a given thread id
///
/// # Arguments
///
/// * `thread_id`: the id of the to be searched thread
/// * `lock_dependencies`: a vector containing all lock dependencies of a trace
///
/// returns: Option<&LockDependency> a lock dependency if a thread with this id exists, otherwise None
///
fn lock_dependency_of_thread(
    thread_id: i64,
    lock_dependencies: &mut Vec<LockDependency>,
) -> Option<&LockDependency> {
    lock_dependencies
        .iter()
        .find(|dependency| dependency.thread_id == thread_id)
}

/// Investigates a given directed graph if it contains a cycle via depth first search
///
/// # Arguments
///
/// * `graph`: the graph to investigate
///
/// returns: usize the amount of detected cycles
///
pub fn validate_dependency_graph(graph: Graph) -> usize {
    let mut visited = HashMap::<i64, bool>::new();
    let mut recursion_stack = HashMap::<i64, bool>::new();

    let mut found_deadlocks = 0;

    for node in graph.keys() {
        if (visited.get(&node).is_none()
            || visited.get(&node).is_some() && visited.get(&node).unwrap() == &false)
            && contains_cycle(&graph, *node, &mut visited, &mut recursion_stack)
        {
            found_deadlocks += 1;
        }
    }

    found_deadlocks
}

/// Helper function to detect a cycle in a given graph
///
/// # Arguments
///
/// * `graph`: the graph to investigate
/// * `node`: the current node of the graph to check
/// * `visited`: a Hashmap containing the already visited nodes
/// * `recursion_stack`: a Hashmap keeping track of the current recursion stack
///
/// returns: bool true if the current node forms a cycle in the given graph
///
fn contains_cycle(
    graph: &Graph,
    node: i64,
    visited: &mut HashMap<i64, bool>,
    recursion_stack: &mut HashMap<i64, bool>,
) -> bool {
    visited.insert(node, true);
    recursion_stack.insert(node, true);

    if let Some(node) = graph.get(&node) {
        for child in node.clone() {
            if visited.get(&child).is_none()
                && contains_cycle(graph, child, visited, recursion_stack)
            {
                return true;
            } else if recursion_stack.get(&child).is_some()
                && recursion_stack.get(&child).unwrap() == &true
            {
                return true;
            }
        }
    }

    recursion_stack.insert(node, false);

    false
}

#[cfg(test)]
mod tests {
    use crate::analyzer::analyze_trace;
    use crate::arguments::Arguments;
    use crate::error::AnalyzerError;

    #[test]
    fn succeed_when_analyzing_valid_trace() -> Result<(), AnalyzerError> {
        // arrange
        let arguments = Arguments::new("test/valid_trace.std", true, false, false, false);

        // act
        let result = analyze_trace(&arguments);

        // assert
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn fail_when_acquire_lock_repeatedly() -> Result<(), AnalyzerError> {
        // arrange
        let arguments = Arguments::new(
            "test/repeated_lock_acquisition.std",
            true,
            false,
            false,
            false,
        );

        // act
        let errors = analyze_trace(&arguments).unwrap_err();

        // assert
        assert_eq!(errors.len(), 1);
        assert!(match errors[0] {
            AnalyzerError::RepeatedAcquisition {
                lock_id,
                thread_id,
                owner_id,
                row,
            } => {
                assert_eq!(lock_id, 9);
                assert_eq!(thread_id, 7);
                assert_eq!(owner_id, 6);
                assert_eq!(row, 7);

                true
            }
            _ => false,
        });

        Ok(())
    }

    #[test]
    fn fail_when_release_lock_repeatedly() -> Result<(), AnalyzerError> {
        // arrange
        let arguments = Arguments::new("test/repeated_lock_release.std", true, false, false, false);

        // act
        let errors = analyze_trace(&arguments).unwrap_err();

        // assert
        assert_eq!(errors.len(), 1);
        assert!(match errors[0] {
            AnalyzerError::RepeatedRelease {
                attempted,
                previous,
                lock_id,
                thread_id,
            } => {
                assert_eq!(previous, 7);
                assert_eq!(attempted, 8);
                assert_eq!(lock_id, 9);
                assert_eq!(thread_id, 6);

                true
            }
            _ => false,
        });

        Ok(())
    }

    #[test]
    fn fail_when_release_non_owning_lock() -> Result<(), AnalyzerError> {
        // arrange
        let arguments = Arguments::new(
            "test/release_non_owning_lock.std",
            true,
            false,
            false,
            false,
        );

        // act
        let errors = analyze_trace(&arguments).unwrap_err();

        // assert
        assert_eq!(errors.len(), 1);
        assert!(match errors[0] {
            AnalyzerError::ReleasedNonOwningLock {
                row: line,
                lock_id,
                thread_id,
                owner,
            } => {
                assert_eq!(line, 7);
                assert_eq!(lock_id, 9);
                assert_eq!(thread_id, 7);
                assert_eq!(owner, 6);

                true
            }
            _ => false,
        });

        Ok(())
    }

    #[test]
    fn fail_when_release_not_acquired_lock() -> Result<(), AnalyzerError> {
        // arrange
        let arguments = Arguments::new(
            "test/release_non_acquired_lock.std",
            true,
            false,
            false,
            false,
        );

        // act
        let errors = analyze_trace(&arguments).unwrap_err();

        // assert
        assert_eq!(errors.len(), 1);
        assert!(match errors[0] {
            AnalyzerError::ReleasedNonAcquiredLock {
                row: line,
                lock_id,
                thread_id,
            } => {
                assert_eq!(line, 6);
                assert_eq!(lock_id, 9);
                assert_eq!(thread_id, 7);

                true
            }
            _ => false,
        });

        Ok(())
    }
}
