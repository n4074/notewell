# notewell

Notewell is in the very early stages of development. The description below is aspirational.

A CLI-based non-hierarchical note taking application inspired by taskwarrior and notational velocity. Notewell is not intended to provide a wiki, a second-brain or a digitalknowledge garden (whatever that is). It is designed for users who spend a lot of time in the terminal, and who want to take lots of short notes quickly. It is for users who find much of the benefit of note taking is in the act of writing things down. Fast and powerful note searching provides confidence that you can find relevant notes, eliminating the burden of categorising or tagging your notes at creation time (although tags and categories will be supported). notewell is built with rust for portability and speed. It's backed by plaintext files and uses git to track changes, making it easy to track history, sync notes across devices and handle merge conflicts. Although plaintext files are the source of truth for all notes, notewell maintains a fulltext search index, built with [Tantivy](https://github.com/quickwit-inc/tantivy), for rapid searching.

