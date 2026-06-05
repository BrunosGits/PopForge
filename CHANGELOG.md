# Changelog

## Unreleased

### Added

- **Subfolder per game toggle** — new option in Convert Options that controls whether each
  converted game gets its own subfolder. OFF: files are placed directly in the output
  folder root. ON: a single subfolder named after the game is created.

### Fixed

- **Subfolder toggle producing wrong folder structure** — psxpackager's `-o` flag expects a
  **directory**, not a file path. The full output path (e.g.
  `OutputFolder/GameName.PBP`) was passed to `-o`, which caused the tool to create a
  subdirectory named `GameName.PBP` and place the PBP inside it, regardless of the toggle
  setting. Now the directory is passed to `-o` and the desired filename is specified via
  the `-f` flag, so the toggle works correctly.

- **Auto-detect output folder overriding user's choice** — when no output folder was set,
  the app silently copied the input file's parent directory as the output folder,
  potentially injecting a game-name subfolder into the path. Removed so the output folder
  is only set by the user via Browse or restored from saved settings.
