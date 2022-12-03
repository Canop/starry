use {
    crate::*,
    anyhow::*,
    cli_log::*,
    chrono::{DateTime, SecondsFormat, Utc},
    std::{collections::HashMap, io::Write},
};

#[derive(Debug)]
pub struct ExtractLine {
    pub time: DateTime<Utc>,
    // this vec is the same len than the extract's names
    pub counts: Vec<Option<usize>>,
}

#[derive(Debug)]
pub struct Extract {
    // names of either users or repos (with a /)
    pub names: Vec<String>,
    pub lines: Vec<ExtractLine>,
}

#[derive(Debug)]
pub(crate) struct Col {
    pub idx: usize,
    pub name: String,
}

impl Extract {
    pub fn write_csv<W: Write>(&self, w: &mut W) -> Result<()> {
        write!(w, "time")?;
        for name in &self.names {
            write!(w, ",{}", name)?;
        }
        writeln!(w)?;
        for line in &self.lines {
            write!(w, "{}", line.time.to_rfc3339_opts(SecondsFormat::Secs, true))?;
            for count in &line.counts {
                if let Some(count) = count {
                    write!(w, ",{}", count)?;
                } else {
                    write!(w, ",")?;
                }
            }
            writeln!(w)?;
        }
        w.flush()?;
        Ok(())
    }
    pub fn read(db: &Db, names: Vec<String>) -> Result<Self> {
        // we first compile the user request in several queries (one per user)
        let mut queries: Vec<UserQuery> = Vec::new();
        for (idx, name) in names.iter().enumerate() {
            let mut tokens = name.split('/');
            let user_id = UserId::new(tokens.next().unwrap()); // SAFETY: first split element is never None
            let query_idx = queries
                .iter()
                .position(|q| q.user_id == user_id)
                .unwrap_or_else(|| {
                    let idx = queries.len();
                    queries.push(UserQuery {
                        user_id,
                        sum: None,
                        repos: Vec::new(),
                    });
                    idx
                });
            match tokens.next() {
                Some(repo) => {
                    queries[query_idx].repos.push(Col {
                        idx,
                        name: repo.to_string(),
                    });
                }
                None => {
                    // if the user specifies twice the same user name, one
                    // won't be filled. We don't care
                    queries[query_idx].sum = Some(Col {
                        idx,
                        name: name.to_string(),
                    });
                }
            }
        }
        debug!("queries: {:#?}", &queries);
        // we now execute all the queries, storing and merging the results in a map per time
        let mut results: HashMap<DateTime<Utc>, ExtractLine> = HashMap::new();
        for query in queries {
            let repo_names = query.repos.iter().map(|col| col.name.as_str()).collect();
            let response_lines = db.extract_user_query(&query.user_id, repo_names)?;
            debug!("response_lines: {:#?}", &response_lines);
            for response_line in response_lines {
                let extract_line =
                    results
                        .entry(response_line.time)
                        .or_insert_with(|| ExtractLine {
                            time: response_line.time,
                            counts: vec![None; names.len()],
                        });
                if let Some(col) = query.sum.as_ref() {
                    extract_line.counts[col.idx] = Some(response_line.sum);
                }
                for (idx, col) in query.repos.iter().enumerate() {
                    extract_line.counts[col.idx] = response_line.counts[idx];
                }
            }
        }
        // we sort the lines
        let mut lines: Vec<ExtractLine> = results.drain().map(|(_, line)| line).collect();
        debug!("lines: {:#?}", &lines);
        lines.sort_by_key(|line| line.time);
        Ok(Self { names, lines })
    }
}
