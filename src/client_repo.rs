use crate::error;
use crate::models::client::Client;
use log::debug;
use std::collections::HashMap;

///Owns a hash map of clients for time constant lookups, storage, and auditing.
#[derive(Default, Debug)]
pub struct ClientRepo {
    ///Map of all client accounts. Returns the client and their data.
    pub clients: HashMap<u16, Client>,
}

impl ClientRepo {
    ///Creates a new client repo.
    pub fn new() -> Self {
        Self { clients: HashMap::new() }
    }
    ///Adds a new client if not found, or gets an existing client. If locked, an error is returned.
    pub fn get_client(&mut self, id: u16) -> Result<&mut Client, error::Error> {
        let client = self.clients.entry(id).or_insert_with(|| Client::new(id));
        if client.locked {
            return Err(error::Error::Client(format!("Client {} is locked", client.client)));
        } else {
            debug!("[!] Client {} returned from get_client", client.client);
            Ok(client)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ClientRepo;

    #[test]
    fn get_new_client() {
        let mut repo = ClientRepo::new();
        let result = match repo.get_client(1){
            Ok(_) => true,
            Err(_) => false
        };
        assert!(result);
    }

    #[test]
    fn client_locked() {
        let mut repo = ClientRepo::new();
        let client = repo.get_client(1).unwrap();
        client.locked = true;
        let result = match repo.get_client(1){
            Ok(_) => true,
            Err(_) => false
        };
        assert!(!result);
    }
}