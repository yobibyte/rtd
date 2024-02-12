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
- Show your inbox: `rtd inbox` or just `rtd i`. 
- Show all your todos: `rtd all`. 
- Show all todos with a label: `rtd @next`.
- Show all your labels: `rtd labels`. 
- Show all your due todos (those with date set, <= today's date): `rtd due`.
- Show all todos in a file (e.g. learn/read.md): `rtd learn/read.md`. 
- Add task to your inbox.md: `rtd add "Delete Todoist on your smartphone."`.
- Add task to your inbox.md with a due-date (YYYY-MM-DD) and a label: `rtd add "Delete Todoist on your smartphone. %2024-01-25 @next"`.
- Add task to any other file (e.g. learn/read.md): `rtd add "Read LSTM paper" learn/read.md`. 
- Toggle task status (done/undone) for task with id &42: `rtd toggle 42`.
- Add label to task with id &32: `rtd al 32 @next`.
- Show task with id &42: `rtd 42`.
- Show URLs (if there are any) in the task description: `rtd 42 url`.
- Remove task with id &42: `rtd rm 42`.
- Toggle task 42 date (if date is set, it will be deleted, otherwise it will be due today): `rtd td 42`.
- Move task with id &42 to file maybe.md: `rtd mv 42 maybe.md`.
- Move all done tasks to the .done list: `rtd archive`.

### Why do you need this?

I got frustrated with Todoist not being able to sync my todos, and I like distraction-free apps in my terminal.

### How can I use this on my phone?

My RTD_ROOT directory is in an Obsidian vault that's synced with my phone. I can edit it through Obsidian on my smartphone.

### How can I help?

Any contributions and feedback are welcomed!

### Why not python? 

I wanted to learn Rust, and I wanted rtd to be damn fast.
