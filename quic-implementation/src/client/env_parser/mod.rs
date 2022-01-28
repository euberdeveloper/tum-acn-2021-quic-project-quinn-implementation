use serde::Deserialize;
use std::error::Error;

fn default_requests() -> String {
    String::from("")
}

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
    downloads: String,
    #[serde(default="default_requests")]
    requests: String,
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
    /// The directory is initially empty, and your client implementation is expected to store
    /// downloaded files into this directory. Served and downloaded files are compared to
    /// verify the test.
    pub downloads: String,
    /// A space separated list of requests a client should execute one by one. (e.g.,
    /// https://127.0.0.2:445/xyz)
    pub requests: Vec<String>,
}

impl Config {
    /// It fetches the envinromnent and returns a Config struct.
    pub fn new() -> Config {
        let config = fetch_env().expect("Error in parsing the environment");
        let testcases = vec!["handshake", "transfer", "multihandshake", "chacha20", "retry", "resumption", "transportparameter", "goodput", "optimize"];
        if !testcases.into_iter().any(|el| String::from(el) == config.testcase) {
            println!("exited with code 127");
            std::process::exit(127);
        }
        let requests = config
            .requests
            .split_whitespace()
            .map(|word| word.to_string())
            .collect();
        // TODO: add validation of the config
        Config {
            sslkeylogfile: config.sslkeylogfile,
            qlogdir: config.qlogdir,
            logs: config.logs,
            testcase: config.testcase,
            downloads: config.downloads,
            requests,
        }
    }
}
