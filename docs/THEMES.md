# Theme-specific ClearCe branding

The five owner-supplied logo variants map in supplied order to Dark, Light, Midnight, Graphite and Forest. System resolves to Dark or Light and responds to Windows color-scheme changes without a restart.

`public/brand/themes/*-source.png` preserves each original file. Logo/symbol derivatives only crop, proportionally resize and center the supplied artwork; no recoloring, stretching or replacement lettering is used. Full marks share a 600×525 canvas and symbols a 128×128 canvas. The signature and tagline remain part of the approved artwork.

The sidebar, compact sidebar and titlebar select the same resolved theme. Theme changes keep fixed layout bounds, so the navigation does not jump. Each backdrop is matched to its artwork; Light retains a contrasting blue backing to keep white lettering legible. Hidden variants are excluded from accessibility display; one brand label describes the visible artwork.

The running native window icon and favicon also follow the resolved theme. Icon resources are released after use and stale asynchronous loads are discarded. The Windows installer/shortcut uses the new Dark variant as a stable default; pinned shortcut icons are managed/cached by Windows and are not promised to change with the app theme.

Theme selection is saved with existing settings. Engine, history and processing behavior are unchanged.
