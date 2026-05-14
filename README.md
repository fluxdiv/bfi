# [b]etter [f]ile [i]nfo

Print a configurable list of info about a file or directory

Wrapper around (and thus requires):
- readlink
- file
- du
- wc
- stat

# Usage

## Option 1
Use a default config that prints all information about a file:
```
bfi some_file --all
```

## Option 2
Using a custom config file:
```
bfi some_file
```

## Creating a custom config file:
- Create a config file at `/home/user/.config/bfi/config.json`
- Ex config with all available options used:
```
{
  "include": {
    "general": ["type", "path"],
    "permissions": ["permissions", "owner", "group"],
    "metadata": ["device", "inode", "links"],
    "size": ["B", "KB", "MB", "GB", "TB"], 
    "count": ["lines", "words", "blocks"],
    "access": ["read", "modified", "changed", "birth"]
  }
}
```

