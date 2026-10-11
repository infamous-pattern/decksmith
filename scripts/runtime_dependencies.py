"""Runtime package names shared by installation and read-only diagnostics."""

FEDORA = {
    'python3': ('python3',), 'systemctl': ('systemd',), 'gdbus': ('glib2',),
    'pactl': ('pulseaudio-utils',), 'wpctl': ('wireplumber',),
    'artwork': ('gtk4', 'libadwaita', 'python3-gobject', 'python3-pillow',
                'python3-cairo', 'librsvg2'),
    'libpulse.so.0': ('pulseaudio-libs',), 'libudev.so.1': ('systemd-libs',),
}
DEBIAN = {
    'python3': ('python3',), 'systemctl': ('systemd',), 'gdbus': ('libglib2.0-bin',),
    'pactl': ('pulseaudio-utils',), 'wpctl': ('wireplumber',),
    'artwork': ('gir1.2-gtk-4.0', 'gir1.2-adw-1', 'gir1.2-rsvg-2.0',
                'python3-gi', 'python3-gi-cairo', 'python3-pil', 'python3-cairo'),
    'libpulse.so.0': ('libpulse0',), 'libudev.so.1': ('libudev1',),
}


def package_groups(release):
    ids = {release.get('ID', ''), *release.get('ID_LIKE', '').split()}
    if ids & {'debian', 'ubuntu'}:
        return DEBIAN
    if 'fedora' in ids:
        return FEDORA
    return {}


def runtime_packages(release):
    return tuple(dict.fromkeys(package for group in package_groups(release).values()
                               for package in group))


def package_hints(release):
    groups = package_groups(release)
    return {name: ', '.join(groups.get(name, (name + ' (see distribution guide)',)))
            for name in ('python3', 'systemctl', 'gdbus', 'pactl', 'wpctl',
                         'artwork', 'libpulse.so.0', 'libudev.so.1')}
