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

## FAQ

### How do I use this tool?

- Show your gtd directory structure: `rtd list`. 
- Show your inbox: `rtd show`. 
- Show all your todos: `rtd show all`. 
- Show all todos in a file (e.g. learn/read.md): `rtd show learn/read.md`. 
- Add task to your inbox.md: `rtd add "Delete Todoist on your smartphone."`.
- Add task to any other file (e.g. learn/read.md): `rtd add "Read LSTM paper" learn/read.md`. 
- Toggle task status (done/undone) for task with id &32: `rtd toggle 32`.
- Remove task with id &32: `rtd rm 32`.
- Move task with id &32 to file maybe.md: `rtd mv 32 maybe.md`.
- Move all done tasks to the .done list: `rtd archive`.

### Why do you need this?

I got frustrated with Todoist not being able to sync my todos, and I like distraction-free apps in my terminal.

### How can I use this on my phone?

My RTD_ROOT directory is in an Obsidian vault that's synced with my phone. I can edit it through Obsidian on my smartphone.

### How can I help?

Any contributions and feedback are welcomed!

### Why not python? 

I wanted to learn Rust, and I wanted rtd to be damn fast.


## Roadmap
- [x] Print out the inbox content.
- [x] Print out all todos in the root recursively.
- [x] Print out todos for a particular file.
- [x] Print out todos for a particular folder, including the root.
- [x] Toggle todo status 
- [x] Add todo to a particular project.
- [x] Archive done todos.
- [x] Add documentation.
- [ ] Add date support (e.g. !YYYY-MM-DD).
- [ ] Print out todos for today based on the date.
- [ ] Add label support (e.g. @next).
- [ ] Cleanup exception handling.
- [ ] Support recurrent tasks.
- [ ] Add the description support. (e.g. $this is a comment)
