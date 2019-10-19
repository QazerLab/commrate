Introduction
============

`commrate` is a tool for analyzing Git commits and checking them for well-formedness by trying to assign each commit a grade from `A` to `F` depending on its size and message structure.

Plain `commrate` command without arguments prints the commit log starting from `HEAD`:

```
$ commrate
COMMIT  GRADE AUTHOR              SUBJECT
52e92f4 A     Kan Liang           perf/x86/cstate: Add Tiger Lake CPU support
0917b95 A     Kan Liang           perf/x86/msr: Add Tiger Lake CPU support
23645a7 A     Kan Liang           perf/x86/intel: Add Tiger Lake CPU support
f1857a2 A     Kan Liang           perf/x86/cstate: Update C-state counters for Ice Lake
1a5da78 A     Kan Liang           perf/x86/msr: Add new CPU model numbers for Ice Lake
1ffa6c0 A     Kan Liang           perf/x86/cstate: Add Comet Lake CPU support
9674b1c A     Kan Liang           perf/x86/msr: Add Comet Lake CPU support
9066288 A     Kan Liang           perf/x86/intel: Add Comet Lake CPU support
b9918bd A     Joe Perches         Documentation/process: Add fallthrough pseudo-keyword
```

This resembles the output of `git log --oneline --no-merges`, but note the `GRADE` column. In this example, each commit is assigned the top grade, which is good.

It is important to remember that this tool is intended for self-assessment (if your commits get the `D` and `F` grades, you *probably* do something wrong) and detecting *obviously* bad commits (i.e. receiving the `F` grade &mdash; these are usually the ones with useless messages like "Fix" or "[TICKET-100500](#faq)"). It is *not* a good idea to use this tool for enforcement in your CI pipeline (even though commits with lowest grades are usually of low quality) for the following reasons:

* the scoring system is not perfect and in some very rare cases even the `F` grade may be assigned to the reasonable commit;
* it is easy to trick the scoring system and bypass the check without even trying to understand, *why* the commit got the bad grade.

So, enjoy the self-assessment, but do not use this tool for enforcement.

The list of `commrate` CLI options may be checked via `commrate --help`.



Scoring Principles
==================

It is hard for computer to tell for sure which commit is good and which is bad (regarding not the payload, but the commit itself). However, it is possible to _guess_ based on the following assumptions:

* the **good** commit message has the subject, the body and the empty line between them, though the body may be absent in some exceptional cases;
* the **good** commit message lines are wrapped;
* the **good** commit subject is meaningful and self-contained, thus, it is usually longer than 15-20 symbols;
* small commits with short commit messages are usually **good** (typo fixes, version changes, minor refactoring, easy bug fixes &mdash; in most cases, messages of such commits contain only the subject);
* small commits with medium and long commit messages are **good** (tricky bug fixes, non-trivial workarounds);
* medium size commits with detailed commit messages are **good** (these usually are the feature implementations);
* medium size commits with short commit messages are usually **bad**;
* huge commits are usually **bad** disregarding the message length.

There are some obvious exceptions to the last assumption: initial commits, some types of refactoring, updates to the vendored dependencies, etc. Some of these exceptions are detected by `commrate` automatically, while some aren't. However, considering that the overall score is based on more than one rule, it is really hard to get the worst grade even when some exceptional case is not handled properly.




Building Commrate
=================

This tool is written in Rust and requires Rust of version 1.37.0 at least.

To build `commrate`, [install](https://doc.rust-lang.org/cargo/getting-started/installation.html) `cargo` and run the following command:

```
cargo build --release
```

The resulting executable binary is `target/release/commrate`.



FAQ
===

**Q**: why do you consider commit messages containing only the ticket ID as bad? Our team tracks everything in JIRA and everyone knows where the information resides, so we just specify the ticket ID and everyone is happy.

**A**: the repository must be self-sufficient and contain the reasonable amount of information, which could be required for work. There are many things which may go wrong if you do not follow this rule: the JIRA project may be renamed; your Internet connection may go down while you troubleshoot some issue (or someone may occasionally want to work offline); finally, think about what the output of `git log --oneline`, `git shortlog` and `git bisect` will look like without meaningful commit subjects. So, there is nothing wrong in specifying the ticket ID in the commit message, but it must always go along with the in-place textual explanation.



Reaching the Release 1.0.0
==========================

The following improvements are expected to be done before the first stable version will be released:

* cli:
    * `--pass-grade`, `-p` option for specifying the minimum "pass" grade. Any commit scored below this grade should cause failure with non-zero exit code. Optionally, add `--pass-score`;
    * `--explain`, `-e` option for showing the commit classes and the results of specific rules;
    * `--explain-grades`, `-g` option for specifying the grades for which the `--explain` should print the output (default: `ABCDF`). Implies `--explain`;
    * automatically invoke `less -FR` for output paging (`--no-pager`, `-r` option for overriding this behavior);

* scoring:
    * provide the smooth transition between short and ordinary commits;
    * make `--explain` provide hints for typical mistakes (e.g.: "hint: move part of long subject into the body");

* misc:
    * after previous item, cover the parsing/scoring with tests;
    * finally, add the manpage.
