#!/usr/bin/env python3
"""Explicit live test workspace. Never starts or adopts an endpoint."""
from profile_lab import ProfileLab

if __name__ == '__main__':
    ProfileLab(connected=True).run(None)
