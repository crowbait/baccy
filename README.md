# baccy

> **Informative, simple, and flexible backup tool based on directory synchronization; written in Rust.**

![demo](demo.svg)

---

## Features

- scanning for changed/new files in parallel to copying
- meaningul output and status
- mirrors directories (= removes files and directories no longer present in source)
- optional JSON configuration file for defining multiple jobs at once, without needing external scripting
- optional skip for delete step
- flexible exclusion and inclusion rules
- ... and more: *check the available JSON and CLI options*

> [!NOTE]
>
> - **Exclusions** are combined with a logical *OR* - anything that hits *any* rule will be excluded.
> - **Inclusions** are:
>   - combined with a logical *AND* *across* categories and a logical *OR* *within* categories. This means that, if both directory and file name rules are passed (not empty), any file must match *any* directory rule *and any* file rule to be included. The same logic applies to inclusion patterns.
>   - checked *after* exclusions have been checked; an excluded path will not be included if targeted this way.
> - **Force-Inclusions** are combined with a logical *OR* - anything that hits *any* rule will be forced to be included.
