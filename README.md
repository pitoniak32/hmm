# hmm
A personal CLI notebook for commands — like man, but for your own notes.

# Basic Usage

To check if there is an existing doc for a command:
```bash
hmm curl
```

if no entry exists you are prompted:
```bash
? No entry for 'curls'! Create one? (Y/n)
```

if an entry exists it will print the markdown using [glow](https://github.com/charmbracelet/glow) in pager mode if its installed, and [less](https://man7.org/linux/man-pages/man1/less.1.html) if not.

# Inspiration
- https://github.com/sinclairtarget/um