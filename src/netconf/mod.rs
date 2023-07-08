use std::{io::{Write, self, Read}, string};

pub struct NETCONFClient {
    // FIXME: Technically, this could be generic.
    channel: ssh2::Channel,
}

const HELLO: &str = "<hello xmlns=\"urn:ietf:params:xml:ns:netconf:base:1.0\">
  <capabilities>
    <capability>urn:ietf:params:netconf:base:1.0</capability>
    <capability>urn:ietf:params:netconf:base:1.1</capability>
  </capabilities>
</hello>";
 
const PAYLOAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <rpc xmlns=\"urn:ietf:params:xml:ns:netconf:base:1.1\" message-id=\"2\">
    <cli xmlns=\"http://cisco.com/ns/yang/cisco-nx-os-device\"><mode>EXEC</mode><cmdline>show version</cmdline></cli>
</rpc>";

impl NETCONFClient {

    pub fn new(channel: ssh2::Channel) -> NETCONFClient{
        return NETCONFClient { channel };
    }   

    fn read(&mut self) -> io::Result<String> {
        let mut result = String::new();
        loop {
            // If you plan to use this, be aware that reading 1 byte at a time is terribly
            // inefficient and should be optimized for your usecase. This is just an example.
            let mut buffer = [1u8; 1];
            let bytes_read = self.channel.read(&mut buffer[..])?;
            let s = String::from_utf8_lossy(&buffer[..bytes_read]);
            result.push_str(&s);
            if result.ends_with("]]>]]>") {
                println!("Found netconf 1.0 terminator, breaking read loop");
                break;
            }
            if result.ends_with("##") {
                println!("Found netconf 1.1 terminator, breaking read loop");
                break;
            }
            if bytes_read == 0 || self.channel.eof() {
                println!("Buffer is empty, SSH channel read terminated");
                break;
            }
        }
        Ok(result)
    } 

    fn write(&mut self, payload: &[u8]) -> io::Result<usize> {
        self.channel.write(payload)
    }

    pub fn send_hello(&mut self) -> io::Result<usize> {
        let payload = format!("{}\n]]>]]>", HELLO);
        return self.write(payload.as_bytes());
    }
}

