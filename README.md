# rtd - local-first cli todo list manager written in Rust.

rtd projects are just markdown files and folders. A folder can have as many nested folders as you want. This is your GTD system now.

## How to set up
* Create the root directory and add it to the .rtd file as `RTD_ROOT=<path_to_created_dir>`.
* Add an `inbox.md` file to the root. If you don't make it, `rtd` will do it for you at the first run.

You can go and create your todo structure in the terminal to make it look smth like that:
```
RTD_ROOT/inbox.md
RTD_ROOT/maybe.md
RTD_ROOT/shopping.md
RTD_ROOT/learning/read.md
RTD_ROOT/learning/watch.md
```

## Roadmap
* [ ] Print out todos for a particular file.
* [ ] Print out todos for a particular folder, including the root.
* [ ] Print out todos for today based on the date.
* [ ] Add todo to a particular project.
* [ ] Toggle todo status (moved to #Done section of the list, and back to the end of the not-done section.)
* [ ] Add label support (e.g. @next)
* [ ] Add date support (e.g. !YYYY-MM-DD)
* [ ] Add the description support. (e.g. $this is a comment)
* [ ] Support recurrent tasks.
