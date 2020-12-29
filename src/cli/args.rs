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
