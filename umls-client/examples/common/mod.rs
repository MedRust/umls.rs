use clap::Parser;
use umls_client::config::Config;
use url::Url;

/// A command line arguments parser.
///
/// If the arguments are not passed by Cli then are fetched from environment variable.
///
/// The following environment variables are used:
/// - UMLS_BASE_URL: UMLS REST API base url.
/// - UMLS_API_KEY: UMLS REST API key.
/// - UMLS_VERSION: UMLS version default: current.
#[derive(Debug, Parser)]
pub struct Args {
    /// UMLS REST API base url.
    #[arg(long, env = "UMLS_BASE_URL", hide_env_values = false)]
    pub base_url: Url,

    /// UMLS API key.
    #[arg(long, env = "UMLS_API_KEY", hide_env_values = true)]
    pub api_key: String,

    /// UMLS release to query.
    #[arg(long, env = "UMLS_VERSION", default_value = "current")]
    pub version: String,
}

/// Create new configuration from [Args].
pub fn config_from_args(args: Args) -> Result<Config, umls_client::config::Error> {
    Config::builder(args.api_key)
        .base_url(args.base_url)
        .version(args.version)
        .build()
}

pub mod term {

    use clap::Parser;
    use umls_client::search::request::Term;

    /// A command line argument to take search term.
    #[derive(Debug, Parser)]
    pub struct Args {
        #[arg(long)]
        /// The search term.
        pub term: Term,
    }
}
