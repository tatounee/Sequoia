use super::Client;

pub struct ClientRef(Client);

impl ClientRef {
    pub const fn as_ref(&self) -> &Client {
        &self.0
    }
}

impl From<Client> for ClientRef {
    fn from(client: Client) -> Self {
        Self(client)
    }
}
