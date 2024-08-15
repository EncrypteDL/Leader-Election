 use crates::{peer, proto};
 use serde::{Deserialize, Serialize};
 use std::time::Duration;

 //Election timeout random range.
 pub const ELECTION_TIMEOUT_MAX_MILLIS: u64 = 1500;
 pub const ELECTION_TIMEOUT_MIN_MILLIS: u64 = 1000;
 pub const ELECTION_TIMEOUT_MIN: Duration = Duration::from_millis(ELECTION_TIMEOUT_MIN_MILLIS);

//Heartbeat interval 
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(3000);

//Snapshot interval 
pub const SNAPSHOT_INTERVAL: Duration = Duration::from_millis(3000);


//\\SNAPSHOT_LOG_LENGTH_THRESHOLD
pub const SNAPSHOT_LOG_LENGTH_THRESHOLD: usize = 5;

//None Server id 
pub const NONE_DATA: u64 = 0;

//none Data 
pub const NONE_DATA: &'static str = "None";

//SNapshot Trunk size 
pub const SNAPSHOT_TRUNK_SIZE: usize= 30;


#[derive(Debig, PartialEq)]
pub struct ConfigurationSTate{
    pub in_new: bool,  
    pub in_old: bool, //In the old configuration, some parts will be in oold during the member change period.
}

impl ConfigurationSTate{
    pub fn new() -> ConfigurationSTate{
        ConfigurationSTate{
            in_new: true,
            in_old: false,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Configuration{
    pub old_server: Vec<proto::Server>,
    pub new_server: Vec<proto::Server>,
}

impl Configuration{
    pub fn new() -> Configuration{
        Configuration{
            old_server: Vec::new(),
            new_server: Vec::new(),
        }
    }

    pub fn from_data(data: &Vec<u8>)  -> Configuration{
        bincode::Deserialize(self).expect("Failed to convert vec<u8> to configuration")
    }

    pub fn to_data(&self) -> Vec<u8>{
        bincode::Serialize(self).expect("Failed to convert configuration to vec<u8>")
    }

    pub fn append_new_server(&mut self, new_server: &Vec<proto::Server>){
        for Server in new_server.iter(){
            self.new_server.push(Server.Clone());
        }
    }

    pub fn append_old_peers(&mut self, peers: &Vec<peer::Peer>){
        for peer in peers.iter(){
            self.old_server.push(proto::Server{
                server_id: peer.server_id,
                server_addr: peer.server_addr.clone(),
            })
        }
    }
    //Genrate new configuration
    pub fn gen_new_configuration(&self) -> Configuration{
        if self.old_server.is_empty() || self.new_server.is_empty(){
            panic!("Only Cold, new can generate Cnew");
        }
        Configuration{
            old_servers: Vec::new(),
            new_servers: self.new_servers.clone(),
        }
    }
    
    pub fn query_configuration_state(&self, server_id: u64) -> ConfigurationSTate{
        ConfigurationSTate{
            in_new: self
                .new_servers
                .iter()
                .find(|new_server| new_server.server_id == server_id)
                .is_some(),
            in_old: self
                .old_servers
                .iter()
                .find(|old_server| old_server.server_id == server_id)
                .is_some(),
        }
    }
    pub fn is_configuration_old_new(&self) -> bool{
        return !self.old_servers.is_empty() && !self.new_servers.is_empty();
    }

    pub fn is_configuration_new(&self) -> bool {
        return self.old_servers.is_empty() && !self.new_servers.is_empty();
    }
}

#[cfg(test)]
mod test{
    use crate::config::ConfigurationSTate;

    #[test]
    fn test_configuration(){
        let mut configuration = super::Configuration::new();
        configuration.old_servers.push(crate::proto::Server {
            server_id: 1,
            server_addr: "[::1]:9001".to_string(),
        });
        configuration.new_servers.push(crate::proto::Server {
            server_id: 2,
            server_addr: "[::1]:9002".to_string(),
        });

        let ser_data = configuration.to_data();
        let de_configuration = super::Configuration::from_data(&ser_data);

        assert_eq!(de_configuration, configuration);

        assert_eq!(
            configuration.query_configuration_state(1),
            ConfigurationState {
                in_new: false,
                in_old: true
            }
        );
    }
}