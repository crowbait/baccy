# baccy <!-- omit from toc -->

> **Informative, simple, and flexible backup tool based on directory synchronization; written in Rust.**

![demo](demo.svg)

---

## Contents <!-- omit from toc -->

- [Features](#features)
- [Configuration](#configuration)
  - [CLI](#cli)
    - [Options](#options)
  - [JSON](#json)
    - [Operation](#operation)
    - [Example](#example)
- [Additional Information](#additional-information)
  - [Exclusions \& Inclusions](#exclusions--inclusions)
  - [Patterns](#patterns)

## Features

- scanning for changed/new files in parallel to copying
- meaningul output and status
- mirrors directories (= removes files and directories no longer present in source)
  - optional skip for delete step
- optional JSON configuration file for defining multiple jobs at once, without needing external scripting
- flexible exclusion and inclusion rules
- ... and more: *check the available JSON and CLI options*

## Configuration

### CLI

`baccy [OPTIONS] <SOURCE/JSON> [TARGET]`

| Parameter     | Description                                                                                                                                                                                                                                    |
| ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| OPTIONS       | Optional flags and options, see [Options](#options)                                                                                                                                                                                            |
| SOURCE / JSON | **Mandatory**<br>Path to *either* a source folder *or* a [JSON config file](#json).<br>Parsing depends on `TARGET` parameter: *if* a target is given, this is interpreted as a directory; otherwise, it is interpreted as a JSON file to read. |
| TARGET        | Path to a destination folder. This will *directly* contain the contents of the source folder.                                                                                                                                                  |

#### Options

| Option                                                                        | Alias          | Description                                                                                                           |
| ----------------------------------------------------------------------------- | -------------- | --------------------------------------------------------------------------------------------------------------------- |
| `--exclude-dirs <RULES>`<sup>[\[1\]](#opt_f1)[\[2\]](#opt_f2)</sup>           | `--xd`<br>`-d` | [Exclude](#exclusions--inclusions) all directories (recursively) having an exactly matching name.                     |
| `--exclude-files <RULES>`<sup>[\[1\]](#opt_f1)[\[2\]](#opt_f2)</sup>          | `--xf`<br>`-f` | [Exclude](#exclusions--inclusions) all files having an exactly matching name.                                         |
| `--exclude-patterns <RULES>`<sup>[\[1\]](#opt_f1)[\[2\]](#opt_f2)</sup>       | `--xp`<br>`-p` | [Exclude](#exclusions--inclusions) all paths matching a [pattern](#patterns).                                         |
| `--include-dirs <RULES>`<sup>[\[1\]](#opt_f1)[\[2\]](#opt_f2)</sup>           | `--id`         | [Include](#exclusions--inclusions) only directories having an exactly matching name.                                  |
| `--include-files <RULES>`<sup>[\[1\]](#opt_f1)[\[2\]](#opt_f2)</sup>          | `--if`         | [Include](#exclusions--inclusions) only files having and exactly matching name.                                       |
| `--include-patterns <RULES>`<sup>[\[1\]](#opt_f1)[\[2\]](#opt_f2)</sup>       | `--ip`         | [Include](#exclusions--inclusions) only paths matching a [pattern](#patterns).                                        |
| `--force-include-dirs <RULES>`<sup>[\[1\]](#opt_f1)[\[2\]](#opt_f2)</sup>     | `--fid`        | Forces [inclusion](#exclusions--inclusions) (overriding ex- and inclusions) of matching directory names.              |
| `--force-include-files <RULES>`<sup>[\[1\]](#opt_f1)[\[2\]](#opt_f2)</sup>    | `--fif`        | Forces [inclusion](#exclusions--inclusions) (overriding ex- and inclusions) of matching file names.                   |
| `--force-include-patterns <RULES>`<sup>[\[1\]](#opt_f1)[\[2\]](#opt_f2)</sup> | `--fip`        | Forces [inclusion](#exclusions--inclusions) (overriding ex- and inclusions) of paths matching a [pattern](#patterns). |
| `--no-delete`<sup>[\[3\]](#opt_f3)</sup>                                      | `--nd`         | Skips the "delete files from target not present in source" step.                                                      |
| `--log-files`<sup>[\[3\]](#opt_f3)</sup>                                      | `--lf`<br>`-l` | Prints names of files being copied and deleted to the console.                                                        |
| `--log-rules`<sup>[\[3\]](#opt_f3)</sup>                                      | `--lr`         | Prints applied exclude-, include-, and force-include rules for each operation.                                        |

- <a name="opt_f1">1</a>: This option accepts one or multiple values.
- <a name="opt_f2">2</a>: When running in JSON-config-mode, any values passed to this option via the command line will be **merged** with the corresponding global options in the JSON (eg: JSON: `"exclude_dirs":["dir1"]`, cli: `--xd dir2`, result: `["dir1", "dir2"]`).
- <a name="opt_f3">3</a>: When running in JSON-config-mode, passing this option via the command line will **override** all *equivalent* global and per-operation settings set in the JSON.

### JSON

| Property                                                    | Type                                      | Description                                                                                                                                                                      |
| ----------------------------------------------------------- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `exclude_dirs`<sup>[\[1\]](#json_global_f1)</sup>           | `string[]`                                | [Exclude](#exclusions--inclusions) exactly matching directory names globally (for all operations).                                                                               |
| `exclude_files`<sup>[\[1\]](#json_global_f1)</sup>          | `string[]`                                | [Exclude](#exclusions--inclusions) exactly matching file names globally (for all operations).                                                                                    |
| `exclude_patterns`<sup>[\[1\]](#json_global_f1)</sup>       | `string[]`                                | [Exclude](#exclusions--inclusions) paths matching [patterns](#patterns) globally (for all operations).                                                                           |
| `include_dirs`<sup>[\[1\]](#json_global_f1)</sup>           | `string[]`                                | [Include](#exclusions--inclusions) only exactly matching directory names globally (for all operations).                                                                          |
| `include_files`<sup>[\[1\]](#json_global_f1)</sup>          | `string[]`                                | [Include](#exclusions--inclusions) only exactly matching file names globally (for all operations).                                                                               |
| `include_patterns`<sup>[\[1\]](#json_global_f1)</sup>       | `string[]`                                | [Include](#exclusions--inclusions) only paths matching [patterns](#patterns) globally (for all operations).                                                                      |
| `force_include_dirs`<sup>[\[1\]](#json_global_f1)</sup>     | `string[]`                                | [Force-include](#exclusions--inclusions) exactly matching directory names globally (for all operations).                                                                         |
| `force_include_files`<sup>[\[1\]](#json_global_f1)</sup>    | `string[]`                                | [Force-include](#exclusions--inclusions) exactly matching file names globally (for all operations).                                                                              |
| `force_include_patterns`<sup>[\[1\]](#json_global_f1)</sup> | `string[]`                                | [Force-include](#exclusions--inclusions) paths matching [patterns](#patterns) globally (for all operations).                                                                     |
| `log_files`<sup>[\[2\]](#json_global_f2)</sup>              | `bool`<sup>[\[3\]](#json_global_f3)</sup> | Prints names of files being copied and deleted to the console.                                                                                                                   |
| `log_rules`<sup>[\[2\]](#json_global_f2)</sup>              | `bool`<sup>[\[3\]](#json_global_f3)</sup> | Prints applied exclude-, include-, and force-include rules for each operation.                                                                                                   |
| `drive_info`                                                | `string[]`                                | After all operations have concluded, prints information about drive usage (used/total). Will take mount points (for Unix) or drive letters (Windows).                            |
| `post_commands`                                             | `string[]`                                | Runs commands on */bin/sh* / *CMD* after all operations have finished; one string for each command to run.                                                                       |
| `wait_on_end`                                               | `bool`                                    | Waits with "Press Enter to continue" instead of self-terminating.<br>Intended to be used when running in some sort of autostart; to be able to see drive info or command output. |
| `operations`                                                | [Operation](#operation)`[]`               | **Mandatory**<br>Array of [operation definitions](#operation).                                                                                                                   |

- <a name="json_global_f1">1</a>: This value will be **merged** with its per-operation equivalent (eg: global: `"exclude_dirs":["dir1"]`, operation: `"exclude_dirs":["dir2]`, result: `["dir1", "dir2"]`).
- <a name="json_global_f2">2</a>: This option will **override** the equivalent per-operation setting for all operations.
- <a name="json_global_f3">3</a>: Has absolutely no effect if set to false; same as omitting.

#### Operation

| Property                 | Type       | Description                                                                                                    |
| ------------------------ | ---------- | -------------------------------------------------------------------------------------------------------------- |
| `source`                 | `string`   | **Mandatory**<br>Source directory to copy from.                                                                |
| `target`                 | `string`   | **Mandatory**<br>Path to a destination folder. This will *directly* contain the contents of the source folder. |
| `exclude_dirs`           | `string[]` | [Exclude](#exclusions--inclusions) exactly matching directory names.                                           |
| `exclude_files`          | `string[]` | [Exclude](#exclusions--inclusions) exactly matching file names.                                                |
| `exclude_patterns`       | `string[]` | [Exclude](#exclusions--inclusions) paths matching [patterns](#patterns).                                       |
| `include_dirs`           | `string[]` | [Include](#exclusions--inclusions) only exactly matching directory names.                                      |
| `include_files`          | `string[]` | [Include](#exclusions--inclusions) only exactly matching file names.                                           |
| `include_patterns`       | `string[]` | [Include](#exclusions--inclusions) only paths matching [patterns](#patterns).                                  |
| `force_include_dirs`     | `string[]` | [Force-include](#exclusions--inclusions) exactly matching directory names.                                     |
| `force_include_files`    | `string[]` | [Force-include](#exclusions--inclusions) exactly matching file names.                                          |
| `force_include_patterns` | `string[]` | [Force-include](#exclusions--inclusions) paths matching [patterns](#patterns).                                 |
| `no_delete`              | `bool`     | Skips the "delete files from target not present in source" step.                                               |
| `log_files`              | `bool`     | Prints names of files being copied and deleted to the console.                                                 |
| `log_rules`              | `bool`     | Prints applied exclude-, include-, and force-include rules for each operation.                                 |

#### Example

```json
{
  "exclude_dirs": ["node_modules", "venv"],
  "drive_info": ["C", "D"],
  "post_commands": ["echo \"completed\" > log.txt"],
  "wait_on_end": true,
  "log_rules": true,

  "operations": [
    {
      "source": "C:/Documents",
      "target": "D:/Documents-backup",
      "exclude_dirs": ["Taxes"],
      "force_include_files": ["tax_return_latest.pdf"]
    },
    {
      "source": "C:/Documents/Taxes",
      "target": "D:/Taxes-backup-complete"
      "no_delete": true,
    }
  ]
}
```

## Additional Information

### Exclusions & Inclusions

> [!IMPORTANT]
> **Directory** rules are recursive: if a directory rules is given, it applies to directories and files with a matching directory name somewhere in its parent path.
> Eg: directory exlude `dir1` will exclude `dir1/dir2/file1`.

- **Exclusions** are simple: any path that hits any of the given rules will not be copied.
- **Inclusions** are *not* the reverse operation; they are the opposite: if any inclusion rules are passed, *only* paths matching one (or more) inclusion rules will be copied (see below to how rules are applied and combined). Everything else is effectively excluded.
- **Force-Inclusions** override both exclusions as well as inclusions; they force the file to be considered for copying.

> [!NOTE]
>
> - **Exclusions** are combined with a logical *OR* - anything that hits *any* rule will be excluded.
> - **Inclusions** are:
>   - combined with a logical *AND* *across* categories and a logical *OR* *within* categories. This means that, *if* both directory and file name rules are passed (not empty), any file must match *any* directory rule *and any* file rule to be included. The same logic extends to patterns.
>   - checked *after* exclusions have been checked; an excluded path will not be included if targeted this way.
> - **Force-Inclusions** are combined with a logical *OR* - anything that hits *any* rule will be forced to be included.

### Patterns

Patterns are defined in glob style.

Patterns are matched against the *relative* path; relative to the source directory.
This means that the entire path must match. To - for example - target all PDF files, you'd write `**/*.pdf`, the `**` matching "none or more arbitrary directory levels".

## Todo <!-- omit from toc -->

- log-only mode for pipe-able output
- add drive info / wait-on-end options to CLI
