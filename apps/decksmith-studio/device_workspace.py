#!/usr/bin/env python3
"""Opt-in saved-device workspace. Does not install or start a service."""
from profile_lab import ProfileLab

class DeviceWorkspace(ProfileLab):
    def __init__(self):super().__init__(workspace=True)

if __name__=='__main__':DeviceWorkspace().run(None)
