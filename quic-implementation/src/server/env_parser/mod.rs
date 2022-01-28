use std::error::Error;
use serde::Deserialize;

fn mucco() -> String {
    String::from("")
}

#[derive(Deserialize, Debug)]
struct EnvConfig {
    sslkeylogfile: String,
    #[serde(default = "mucco")]
    qlogdir: String,
    logs: String,
    testcase: String,
    www: String,
    certs: String,
    ip: String,
    port: u16
}

fn fetch_env() -> Result<EnvConfig, Box<dyn Error>> {
    let config = envy::from_env()?;
    Ok(config)
}

#[derive(Debug)]
pub struct Config {
    /// It contains the path and name of the file used for the key log. The output is required
    /// to decrypt traces and verify tests. The file has to be in the NSS Key Log format 1 .
    pub sslkeylogfile: String,
    /// qlog results are not required but might help to debug your output. However they have
    /// a negative impact on performance so you might want to deactivate it for some tests.
    pub qlogdir: String,
    /// It contains the path to a directory the server can use for its general logs. These will
    /// be uploaded as part of the results artifact.
    pub logs: String,
    /// The name of the test case. You have to make sure a random string can be handled
    /// by your implementation.
    pub testcase: String,
    /// It contains the directory that will contain one or more randomly generated files. Your
    /// server implementation is expected to run on the given port 443 and serve files from
    /// this directory.
    pub www: String,
    /// The runner will create an X.509 certificate and chain to be used by the server during
    /// the handshake. The variable contains the path to a directory that contains a priv.key
    /// and cert.pem file.
    pub certs: String,
    /// The IP the server has to listen on.
    pub ip: String,
    /// The port the server has to listen on.
    pub port: u16
}

impl Config {
    /// It fetches the envinromnent and returns a Config struct.
    pub fn new() -> Config {
        let config = fetch_env().expect("Error in parsing the environment");
        let testcases = vec!["handshake", "transfer", "multihandshake", "chacha20", "retry", "resumption", "zerortt", "transportparameter", "goodput", "optimize"];
        if !testcases.into_iter().any(|el| String::from(el) == config.testcase) {
            println!("exited with code 127");
            std::process::exit(127);
        }
        // TODO: add validation of the config
        Config {
            sslkeylogfile: config.sslkeylogfile,
            qlogdir: config.qlogdir,
            logs: config.logs,
            testcase: config.testcase,
            www: config.www,
            certs: config.certs,
            ip: config.ip,
            port: config.port
        }
    }
}
