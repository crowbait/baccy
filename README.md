# baccy

> **Informative, simple backup tool based on directory synchronization, with reasonable efficiency; written in Rust.**

![demo](demo.svg)

---

## Features

- scanning for changed/new files in parallel to copying
- meaningul output and status
- mirrors directories (= removes files and directories no longer present in source)
- optional JSON configuration file for defining multiple jobs at once, without needing external scripting
- optional skip for delete step
- flexible exclusion rules
