# Strings Synchronizer

A Rust-based command-line tool for synchronizing iOS/macOS `.strings` localization files. This tool helps maintain consistency across multiple localization files by ensuring all translations contain the same keys in the same order as the original file.

## Purpose

When working with iOS/macOS localization, it's common to have a primary `.strings` file (usually `en.lproj/Localizable.strings`) and multiple localized versions. Over time, these files can get out of sync:
- New keys are added to the original but missing in translations
- Keys appear in different orders across files
- Deprecated keys remain in translation files

This tool solves these problems by:
- **Synchronizing keys**: Ensures all localization files contain the same keys as the original
- **Preserving order**: Maintains the same key order across all files
- **Keeping translations**: Retains existing translations for keys that haven't changed
- **Adding missing keys**: Automatically adds new keys from the original to all localization files
- **Preserving comments**: Keeps file headers and comments intact
- **Support multiline keys**: Handles keys with multiple lines of text

## Installation

### Prerequisites
- Rust toolchain (1.70 or later)

### Build from source
```bash
cargo build --release
```

The compiled binary will be available at `target/release/synclproj`.

## Usage

```bash
synclproj <original.strings> <lproj_folder>
```

### Arguments
- `<original.strings>` - Path to the original/master `.strings` file (e.g., `en.lproj/Localizable.strings`)
- `<lproj_folder>` - Path to the folder containing localization directories (e.g., `MyApp/Resources/`)

### Example

```bash
# Sync all .strings files in the localization folder
synclproj en.lproj/Localizable.strings ./Resources/

# Sync specific language folder
synclproj en.lproj/Localizable.strings fr.lproj/
```

### What happens during synchronization

1. **Scans** the specified folder recursively for all `.strings` files
2. **Parses** the original file to extract keys and their order
3. **For each localization file**:
   - Identifies existing keys and their translations
   - Adds missing keys from the original (with original values as placeholders)
   - Reorders all entries to match the original file's order
   - Preserves file header comments
   - Removes excessive empty lines

4. **Reports** which keys were added to each file

### Example Output

```
Scanning folder: ./Resources/
Syncing: ./Resources/fr.lproj/Localizable.strings
   Adding missing key: new_feature_title
   Adding missing key: new_feature_description
Syncing: ./Resources/de.lproj/Localizable.strings
   Adding missing key: new_feature_title
   Adding missing key: new_feature_description
All .strings files synchronized successfully!
```

## File Format Support

The tool supports standard iOS/macOS `.strings` file format:

```
/* Header comment */

"key1" = "value1";
"key2" = "value2";
"multiline_key" = "This is a \
multiline value";
```

## Safety

- The tool overwrites localization files in place
- Consider committing your changes to version control before running the tool
- Always review the changes after synchronization

## License

This project is open source and available for use and modification.

## Contributing

Contributions are welcome! Feel free to submit issues or pull requests.
