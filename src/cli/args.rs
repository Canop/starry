use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// The history of current stars tells only half the starry.
///
///
/// Source at https://github.com/Canop/starry
pub struct Args {
    /// print the version
    #[argh(switch, short = 'v')]
    pub version: bool,

    #[argh(subcommand)]
    pub command: Option<ArgsCommand>,

    /// tell what files are modified
    #[argh(switch)]
    pub verbose: bool,

    /// don't do any modification on disk (for tests or dev)
    #[argh(switch)]
    pub no_save: bool,

    /// number max of rows in a report (default: 20)
    #[argh(option, default = "20")]
    pub max_rows: usize,

    /// number of threads to use (default 15)
    #[argh(option, default = "15")]
    pub threads: usize,

    #[argh(option, default = "Default::default()")]
    /// color and style: 'yes', 'no' or 'auto' (auto should be good in most cases)
    pub color: BoolArg,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum ArgsCommand {
    Set(SetCommand),
    Get(GetCommand),
    Follow(FollowCommand),
    Unfollow(UnfollowCommand),
    Gaze(GazeCommand),
    Extract(ExtractCommand),
    Check(CheckCommand),
    List(ListCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// set a property, eg `starry set github-api-token blabla`
#[argh(subcommand, name = "set")]
pub struct SetCommand {
    #[argh(positional)]
    pub name: String,
    #[argh(positional)]
    pub value: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// get a property, eg `starry get github-api-token`
#[argh(subcommand, name = "get")]
pub struct GetCommand {
    #[argh(positional)]
    pub name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// check the existence of a user
#[argh(subcommand, name = "check")]
pub struct CheckCommand {
    #[argh(positional)]
    pub name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// start following a github user
#[argh(subcommand, name = "follow")]
pub struct FollowCommand {
    #[argh(positional)]
    pub name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// stop following a github user
#[argh(subcommand, name = "unfollow")]
pub struct UnfollowCommand {
    #[argh(positional)]
    pub name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// look at the stars (default command)
#[argh(subcommand, name = "gaze")]
pub struct GazeCommand {}

#[derive(FromArgs, PartialEq, Debug)]
/// extract time series for one or several user or repo
#[argh(subcommand, name = "extract")]
pub struct ExtractCommand {
    #[argh(positional)]
    pub names: Vec<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// list either all users, or all repos of a user
#[argh(subcommand, name = "list")]
pub struct ListCommand {
    #[argh(positional)]
    pub login: Option<String>,
}

/// An optional boolean for use in Argh
#[derive(Debug, Clone, Copy, Default)]
pub struct BoolArg(Option<bool>);

impl BoolArg {
    pub fn value(self) -> Option<bool> {
        self.0
    }
}

impl argh::FromArgValue for BoolArg {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        match value.to_lowercase().as_ref() {
            "auto" => Ok(BoolArg(None)),
            "yes" => Ok(BoolArg(Some(true))),
            "no" => Ok(BoolArg(Some(false))),
            _ => Err(format!("Illegal value: {:?}", value)),
        }
    }
}
