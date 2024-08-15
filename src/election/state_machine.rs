use std::fmt::Debug;

pub trait StateMachine: Debug + Send + 'static {
    // apply log entries
    fn apply(&mut self, data: &Vec<u8>);

    // Generate Snapshot
    // TODO Copy-on-write
    fn take_snapshot(&mut self, snapshot_filepath: String);

    // Restoring from a snapshot
    fn restore_snapshot(&mut self, snapshot_filepath: String);
}
