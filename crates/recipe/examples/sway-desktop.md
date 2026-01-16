# Sway Desktop Environment - Recipe Stack

Building a complete Wayland desktop using LevitateOS recipes.

## Component Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                    USER APPLICATIONS                         │
│  foot  waybar  wofi  mako  grim  wl-clipboard  thunar       │
├─────────────────────────────────────────────────────────────┤
│                         SWAY                                 │
│  sway  swaybg  swaylock  swayidle                           │
├─────────────────────────────────────────────────────────────┤
│                        WLROOTS                               │
├─────────────────────────────────────────────────────────────┤
│                    WAYLAND CORE                              │
│  wayland  wayland-protocols  libinput  xkbcommon            │
├─────────────────────────────────────────────────────────────┤
│                    GRAPHICS STACK                            │
│  mesa  libdrm  pixman  cairo  pango  freetype  fontconfig   │
├─────────────────────────────────────────────────────────────┤
│                    SESSION/SYSTEM                            │
│  seatd  dbus  polkit  pipewire                              │
├─────────────────────────────────────────────────────────────┤
│                    BASE LIBRARIES                            │
│  glib  json-c  pcre2  libffi  libxml2  expat                │
└─────────────────────────────────────────────────────────────┘
```

## Build Order (dependency-first)

### Tier 0: Base Libraries
1. libffi
2. pcre2
3. expat
4. json-c
5. glib

### Tier 1: Graphics Foundation
6. libdrm
7. pixman
8. freetype
9. fontconfig
10. cairo
11. harfbuzz
12. pango

### Tier 2: Wayland Core
13. wayland
14. wayland-protocols
15. libxkbcommon
16. libinput

### Tier 3: Graphics Drivers
17. mesa

### Tier 4: Session
18. seatd
19. dbus

### Tier 5: Compositor
20. wlroots
21. sway
22. swaybg
23. swaylock
24. swayidle

### Tier 6: Desktop Apps
25. foot (terminal)
26. waybar (status bar)
27. wofi (launcher)
28. mako (notifications)
29. grim + slurp (screenshots)
30. wl-clipboard

## Recipes Created

### Wayland Core
- [x] wayland.recipe
- [x] wayland-protocols.recipe
- [x] libxkbcommon.recipe
- [x] libinput.recipe

### Compositor
- [x] wlroots.recipe
- [x] seatd.recipe

### Sway Ecosystem
- [x] sway.recipe
- [x] swaybg.recipe
- [x] swaylock.recipe
- [x] swayidle.recipe

### Desktop Apps
- [x] foot.recipe (terminal)
- [x] waybar.recipe (status bar)
- [x] wofi.recipe (launcher)
- [x] mako.recipe (notifications)
- [x] gtk-layer-shell.recipe (dependency for waybar/wofi)

### Utilities
- [x] grim.recipe (screenshots)
- [x] slurp.recipe (region selection)
- [x] wl-clipboard.recipe (clipboard)

## Notes

- SHA256 checksums marked as TODO - need to be filled in when building
- Base libraries (glib, cairo, pango, mesa, etc.) assumed from Fedora base system
- All recipes use meson build system
- XWayland disabled by default (pure Wayland)
