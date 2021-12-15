# Github Bus Factor estimation console tool

## Problem

Create console tool that will query selected <b>number</b> of repositories i guven programming <b>language</b> and display percentage of contrubution for most contributing user.

## Solution description

Tool is written in <b>Rust</b> (as a learning excercise) and uses following creates:
-  <b>tokio</b>  - runtime to use asynchronous programming.
-  <b>serde</b>  - for deserializing json Github API responses
-  <b>reqwest</b>  - for calling Github API
-  <b>clap</b> - for getting and parsing command line aruguments and generating text help
- <b>futures</b> - for streaming results

## Some explanation

<i> Disclaimer - I'm learning Rust</i>

I decided to use streams because I was looking for something like async enumerator. Data required to display results can't be fetched with single requests so single request must be made to get some nubmer of repositiories and than for each of them separate request is needed to calculate contribution percentage.<br>
Without streaming user would need to wait for all requests to finish before list is displayed.<br>
Streaming allows user to see data as soon as it's fetched and deals with getting repository data only when it's needed.

## Possible improvements

- Use buffering in streams for better performance ?
- When fetching paged result of repositories set page size to amount that is exactly needed
