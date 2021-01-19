[![MIT][s2]][l2] [![Latest Version][s1]][l1] [![docs][s3]][l3] [![Chat on Miaou][s4]][l4]

[s1]: https://img.shields.io/crates/v/starry.svg
[l1]: https://crates.io/crates/starry

[s2]: https://img.shields.io/badge/license-MIT-blue.svg
[l2]: LICENSE

[s3]: https://docs.rs/starry/badge.svg
[l3]: https://docs.rs/starry/

[s4]: https://miaou.dystroy.org/static/shields/room.svg
[l4]: https://miaou.dystroy.org/3

# starry

A tool to store the counts of GitHub stars.

## Why

Did you notice all those tools pretending to graph the numbers of stars on repositories never show anything going down ?

That's because you've been lied to: those tools only show the current stars, with their age. Because that's the only information you can get with the GitHub API.

If you want to see the real stars graph there's no other solution than to regularly query and store the numbers. That's what this tool does.

Because the history of current stars tells only half the starry.

## Installation

You must have [Rust installed](https://rustup.rs). Do

	cargo install starry

## Usage

In order to query the GitHub API, you must register your API token:


	starry set github_api_token your-token

(see https://docs.github.com/en/free-pro-team@latest/github/authenticating-to-github/creating-a-personal-access-token for creation)

You need to say what user(s) you want to follow:

	starry follow dtolnay
	starry follow ralt

Fetching the stars is done with

	starry

Starry will tell you about new repositories and rising or dropping stars:

![changes](doc/changes.png)

If you want regular data, you should probably add a cron rule.

Data are stored in clear in CSV files (if you're on linux, they're in `~/.local/share/starry/stars`.
Those files can be used as is.

If you want time series, for example to graph them, you may extract them as csv with the `extract` subcommand:

	starry extract shepmaster ralt BurntSushi dtolnay dtolnay/anyhow > test.csv

In this query we want to get the time series of 4 users (meaning their total number of stars) and one repository.

Here's an example of result:

![csv](doc/csv.png)

You may graph the data with [csv2svg](https://github.com/Canop/csv2svg):

If you run

	starry extract dtolnay/thiserror | csv2svg

then your browser displays a graph like this:

![svg_dtolnay_thiserror](doc/svg_dtolnay_thiserror.png)

You may display several entries, like `starry extract dtolnay/thiserror dtolnay/anyhow | csv2svg`

## Starry Online

A limited version of Starry can be seen at [https://dystroy.org/starnet/](https://dystroy.org/starnet/).
