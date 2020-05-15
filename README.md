# Sox: inter-process job communication

Sox is a tool for coordinating jobs across your computer. You may have run into this situation before, multiple terminal windows for one project that need coordination around one task. This task could be building, installing dependencies, running a bootstrapping script, etc.

Example usage:
```
TERMINAL WINDOW 1:
[~/my_project]-[master] > sox cast -c yarn
...
Running job: 18854


TERMINAL WINDOW 2:
[~/my_project]-[master] > sox listen 18854 -c yarn start-server
Waiting for: 18854
...
...
yarn run 1.21
$ node ./src/server.js
...
```

Running the `cast` command broadcasts the command, `-c` with the given job id. Running `listen` with a job id will wait until it receives a status code about the job id, and then executes the command `-c` if the job is successful.

## Installation

Clone this repositiory and change into the directory. Then run `cargo install --path .`


### This program does not run on Windows since it uses Unix sockets.
