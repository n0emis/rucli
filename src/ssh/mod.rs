use std::{
    net::TcpStream,
};

use ssh2::Session;

use self::error::SSHError;

pub mod error;

pub struct SSHConnection {
    pub user: String,
    pub target: String,

    pub sess: Option<ssh2::Session>,
}

impl SSHConnection {
    pub fn new(user: &str, target: &str) -> SSHConnection {
        return SSHConnection {
            user: String::from(user),
            target: String::from(target),
            sess: None,
        };
    }

    pub fn connect(&mut self) -> Result<(), SSHError> {
        let tcp = TcpStream::connect(self.target.as_str())?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        sess.userauth_agent(self.user.as_str())?;

        let mut s = sess.channel_session()?;
        s.subsystem("netconf")?;

        self.sess = Some(sess);

        return Ok(());
    }
}
