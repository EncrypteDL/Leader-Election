///Consensus mechanisms (e.g., Paxos, Raft)
/// Leader election based on Raft Consensus mechanisms
/// It Handles different consensus protocols that might be used.
use crate::{config, log, metadata, peer, proto, rpc, snapshot, state_machine, timer, util};
use logging::*;
use std::cell::RefCell;
use std::io::{Read, Seek, Write};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum State{
    Follwer,
    Candidate,
    Leader,
}

#[derive(Debug)]
pub struct Consensus{
    pub server_id: u64,
    pub server_addr: String,
    pub metatdata: metadata::Metadata,
    pub state: State,
    pub commit_index: u64,
    pub last_applied: u64,
    pub leadr_id: u64,
    pub peer_manager: peer::PeerManager,
    pub log: log::Log,
    pub snapshot: snapshot::Snapshot,
    pub configuration: config::ConfigurationSTate,
    pub election_timer: Arc<Mutex<timer::Timer>>,
    pub heartbeat_timer: Arc<Mutex<timer::Timer>>,
    pub snapshot_timer: Arc<Mutex<timer::Timer>>,
    rpc_client: rpc::Client, // TODO 
    tokio_runtime: tokio::runtime::Runtime,
    pub state_machine: Box<dyn state_machine::StateMachine>,
}

impl Consensus{
    pub fn new(server_id: u64, port: u32, peers: Vec<peer::Peer>, state_machine: Box<dyn state_machine::StateMachine>, snapshot_dir: String, metadata_dir: String,) -> Consensus{
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        let mut consensus = Consensus{
            server_id,
            server_addr: format!("[::1]:{}", port),
            metadata: metadata::Metadata::new(metadata_dir.clone()),
            state: State::Follower,
            election_timer: Arc::new(Mutex::new(timer::Timer::new("election_timer"))),
            heartbeat_timer: Arc::new(Mutex::new(timer::Timer::new("heartbeat_timer"))),
            snapshot_timer: Arc::new(Mutex::new(timer::Timer::new("snapshot_timer"))),
            commit_index: 0, // Submitted log index, monotonically increasing starting from 0
            last_applied: 0, // Todo
            leader_id: config::NONE_SERVER_ID,
            peer_manager: peer::PeerManager::new(),
            log: log::Log::new(1, metadata_dir),
            snapshot: snapshot::Snapshot::new(snapshot_dir),
            configuration_state: config::ConfigurationState::new(),
            rpc_client: rpc::Client {},
            tokio_runtime,
            state_machine,
        };

        //Load raft metadata
        consensus.metadata.reload();
        
        //Load raft log
        consensus.log.reload();

        //Load raft snapshot
        consensus.snapshot.reload_metadata();


        // Load snapshot into the state machine
        if let Some(snapshot_filepath) = consensus.snapshot.latest_snapshot_filepath(){
            consensus.state_machine.restore_snapshot(snapshot_filepath);
        }

        //inittialze other peer
        consensus.peer_manager.add_peers(
            peers,
            consensus
                .log
                .last_index(consensus.snapshot.last_included_index),
        );
        consensus
    }

    pub fn replicate(&mut self, r#type: proto::EntryType, data: Vec<u8>,) -> Result<(), Box<dyn std::error::Error>>{
        if self.state != State::Leader{
            error!("replicate should be processed by leader");
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "not leader",
            )));
        }
        info!("replicate data: {:?}", &data);

        // Save the log entry
        self.log
            .append_data(self.metadata.current_term, vec![(r#type, data.clone())]);

        // Apply the Configuration entry immediately
        if r#type == proto::EntryType::Configuration {
            self.apply_configuration(config::Configuration::from_data(&data), false);
        }

        // Send log entry to peer node
        self.append_entries(false);

        Ok(())
    }

    fn appen_entries(&mut self, Heartbeat: bool) -> bool{
        //Check if you are the leader
        if self.state != State::Leader{
            error!("state is {:?}, can't append entries", self.state);
            return false;
        }
    }

    //Todo: Send additional log requests in parallel
    let peer_server_ids = self.peer_manager.peer_server_ids();
    info!(
        "Start to append entreis (heartbeat: {}) to peers: {:?}", heartbeat, &peer_server_ids
    );

    if peer_server_ids.is_empty()

}