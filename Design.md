# Design

## Endpoints

### `meta.json`

```json
{
    "version": "1.2.3",
    "messages": [
        {
            "id": "update_1.0",
            "version": "<semver constraint>",   // optional, e.g. "<1.0"
            "message": "You should update!!",   // can contain markdown or something like that
            "icon": "warning",                  // optional, defaults to "info", possible values: "info"/"warning"/"error"
            "show_always": true,                // optional, defaults to false
        }
    ],
    "games": [
        {
            "id": "Desperados3",
            "name": "Desperados III",
            "appid": "610370",
            // "image": "<url>",                // something to consider for the future
        }
    ],
    "mods": [ /* See <game>.json */ ]
}
```

### `games/<game>.json`

Download url can be `.dll` or `.zip` that will be placed in the `plugins` directory.

```json
{
    "mods": [
        {
            "version": "1.2.3",
            "id": "ExtendedCheats",
            "name": "Extended Cheats",              // optional
            "description": "Adds more cheats",      // optional
            "ytinu_version": "<semver constraint>", // optional
            "source": "https://github.com/benediktwerner/Desperados3Mods",      // optional
            "homepage": "https://github.com/benediktwerner/Desperados3Mods",    // optional
            "download": "https://github.com/benediktwerner/Desperados3Mods/releases/download/cheats-v1.1.1/ExtendedCheats.dll",
        }
    ]
}
```

### External mod repo

```json
{
    "games": [ /* See meta.json */ ],
    "mods": {
        "Desperados3": [ /* See <game>.json */ ]
    }
}
```
