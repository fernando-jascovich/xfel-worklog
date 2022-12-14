# xfel-worklog

This is cli handler for managing a diary of markdown files.

Given a directory with markdown files (under `DIARY_ROOT`) environmental variable,
this handler will perform certain queries or modifications to those files.

I am assuming a particular frontmatter format which is the key for the majority of the functions resolved by this tool. Here's an example of that frontmatter:
```md
---
author: Some Name here
date: 1970-01-01
tags:
- A-tag
- cool
- yep-is-an-array
estimate: any string for now
worklog:
- 2022-10-11T15:44:00,2022-10-11T15:54:18
- 2022-10-13T15:20:57,2022-10-13T16:18:21

---
# My great file
...
```
That's the main format, `worklog` is an array of comma separated timestamps in ISO format. Each entry has `START,STOP` timestamps.

Each parameter has (thanks to [clap](https://docs.rs/clap/latest/clap/)) details about its behavior. On any parameter you can use `--help` in order to check details:

```bash
$ xfel-worklog --help
Usage: xfel-worklog <COMMAND>

Commands:
  query   Query for elements
  action  Perform an action on elements
  browse  Similar to query but this will return a list of matched paths
  fetch   Fetch element from Jira
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

## Query

It will print a ascii table with query results and time spent on each document according their worklogs.

A query can contain:
 - Filter by tags: `xfel-worklog query -t a-tag -t cool`
 - Filter by path: `xfel-worklog query -p something/to/match/against/file/blob`
 - A start date: `xfel-worklog query 2020-01-01`. When not specified, start date will be equals to today in iso format.
 - An end date: `xfel-worklog query 2020-01-01 2020-01-30`.

There are _certain_ natural language queries supported for start and end date positional arguments. Check [here](./src/cli/date_parse.rs) on `#input` for a list of supported queries.

Two env variables are involved here:

- `DIARY_ROOT`: defines root directory of diary files
- `DIARY_INCLUDE_ARCHIVE`: include (or not, this is default) files under `$DIARY_ROOT/_archive` directory

## Action

The following actions are supported:

- Start: It will create the first part of a new worklog entry on matched element
- Stop: It will put end timestamp of current worklog entry on matched element
- SyncWorklog: This will create worklog entries on Jira based on frontmatter's entries.

In order to perform an action, you need to provided a path. Given a file named `my/file/is/here.md`, any of the following should work:

- `xfel-worklog action -p here.md start`
- `xfel-worklog action -p my/file/is/here.md start`

Be aware that, if provided path matches multiple documents, action will be performed on all of them.

## Browse

This command will return a list of files present in user's diary (this is `DIARY_ROOT` environmental variable).

There's a modifier that allows to list only files with an open worklog entry:

`xfel-worklog browse -a`

## Fetch

Given a set of credentials configured as environmental variables, this command will download a Jira ticket and place it into user's diary.

Required environmental variables are listed in [./env.example](./env.example) file.

You need to provide Jira ticket's key in order to download it: `xfel-worklog fetch XXX-1`. And there's an optional argument for set output's path into user's diary. For example, `xfel-worklog fetch -p some/path/here YYY-2` will generate the following file: `$DIARY_ROOT/some/path/here/YYY-2.md`.

