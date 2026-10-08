"""Record the native bundle environment without claiming release acceptance."""
import platform


def bundle_platform():
    if platform.system() != 'Linux':
        raise ValueError('Decksmith bundles must be built on Linux.')
    release = platform.freedesktop_os_release()
    distribution = release.get('ID', '')
    version = release.get('VERSION_ID', '')
    if not distribution or not version:
        raise ValueError('The build environment must identify its distribution and version.')
    names = {'fedora': 'Fedora', 'debian': 'Debian', 'ubuntu': 'Ubuntu'}
    label = names.get(distribution, release.get('NAME', distribution)) + ' ' + version
    libc, libc_version = platform.libc_ver()
    return {
        'platform': 'Linux',
        'architecture': platform.machine(),
        # Existing candidate admission checks this field. It identifies the
        # native target; qualification evidence remains a separate requirement.
        'tested_distribution': label,
        'build_environment': {
            'distribution': distribution, 'version': version,
            'libc': libc, 'libc_version': libc_version,
        },
    }
