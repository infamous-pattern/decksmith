import os
import unittest
from unittest.mock import Mock, patch
from system_controls import Controls, COMMANDS, LABELS
from system_confirm import confirmed_power


class SystemControlTests(unittest.TestCase):
    def test_non_gnome_dnd_never_claims_or_changes_gnome_notification_settings(
        self,
    ):
        c = Controls()
        c.call = Mock(return_value=(False,))
        c.setting = Mock()
        self.assertEqual(
            c.state("dnd"),
            {"available": False, "active": False, "text": "Unavailable"},
        )
        with self.assertRaises(ValueError):
            c.execute("dnd")
        c.setting.assert_not_called()
        self.assertEqual(c.call.call_args.args[4], "NameHasOwner")

    def test_gnome_dnd_still_reads_and_toggles_banner_setting(self):
        c = Controls()
        c.call = Mock(return_value=(True,))
        setting = Mock()
        setting.get_boolean.return_value = True
        c.setting = Mock(return_value=setting)
        self.assertEqual(
            c.state("dnd"), {"available": True, "active": False, "text": "Off"}
        )
        with patch("system_controls.Gio.Settings.sync"):
            c.execute("dnd")
        setting.set_boolean.assert_called_once_with("show-banners", False)

    def test_unknown_command_rejected_before_bus_access(self):
        c = Controls()
        c.call = Mock()
        with self.assertRaises(ValueError):
            c.execute("run_shell")
        c.call.assert_not_called()

    def test_power_actions_only_open_confirmation(self):
        for command in ("reboot", "shutdown"):
            c = Controls()
            c.state = Mock(return_value={"available": True})
            c.call = Mock()
            with patch("system_controls.Gio.Settings.sync"), patch(
                "subprocess.Popen"
            ) as spawn:
                c.execute(command)
            self.assertEqual(spawn.call_args.args[0][-1], command)
            c.call.assert_not_called()

    def test_inhibitor_blocks_confirmed_power(self):
        c = Mock()
        c.call.return_value = (True,)
        with self.assertRaises(RuntimeError):
            confirmed_power("reboot", c)
        self.assertEqual(c.call.call_count, 1)

    def test_confirmed_power_uses_nonforced_authorized_method(self):
        c = Mock()
        c.call.side_effect = [(False,), ()]
        confirmed_power("shutdown", c)
        self.assertEqual(c.call.call_args.args[4], "PowerOff")
        self.assertEqual(c.call.call_args.args[5].unpack(), (True,))

    def test_labels_and_icons_complete(self):
        from system_artwork import image

        self.assertEqual(set(LABELS), COMMANDS)
        for command, label in LABELS.items():
            self.assertLessEqual(len(label), 24)
            self.assertTrue(image(command).startswith(b"\x89PNG"))

    def test_assignment_inherits_theme_and_preserves_custom_styles(self):
        from system_artwork import populate
        from themes import effective

        key = {"action": {"type": "system", "command": "lock"}, "label": "Lock"}
        populate(key)
        self.assertNotIn("label_position", key)
        self.assertNotIn("label_background", key)
        layout = {
            "theme": {
                "preset": "light",
                "appearance": {
                    "label_position": "top",
                    "label_background": "dark",
                },
            }
        }
        style = effective(layout, key)
        self.assertEqual(style["icon_size"], 75)
        self.assertEqual(style["background"], [236, 240, 245])
        self.assertEqual(style["label_position"], "top")
        self.assertEqual(style["label_background"], "dark")
        layout["theme"]["appearance"]["icon_size"] = 60
        self.assertEqual(effective(layout, key)["icon_size"], 60)
        key.update(
            label_position="bottom",
            background_color="blue",
            appearance={"icon_size": 50},
        )
        populate(key)
        style = effective(layout, key)
        self.assertEqual(style["icon_size"], 50)
        self.assertEqual(style["label_position"], "bottom")
        self.assertEqual(style["background"], [90, 170, 255])


# Tests may run inside a real Hyprland session; default to GNOME behaviour.
_environment = patch.dict("os.environ")


def setUpModule():
    _environment.start()
    os.environ.pop("HYPRLAND_INSTANCE_SIGNATURE", None)


def tearDownModule():
    _environment.stop()


@patch.dict("os.environ", {"HYPRLAND_INSTANCE_SIGNATURE": "test"})
class HyprlandControlTests(unittest.TestCase):
    def test_uses_omarchy_and_logind_instead_of_gnome(self):
        calls = []
        replies = {"isDnd": "off", "temperature": "4000"}

        def run(*argv):
            calls.append(argv)
            return replies.get(argv[2] if len(argv) > 2 else "", "ok")

        c = Controls()
        c.call = Mock()
        c.setting = Mock()
        with (
            patch("system_controls.run", side_effect=run),
            patch("system_controls.shutil.which", return_value="/usr/bin/x"),
        ):
            off = {"available": True, "active": False, "text": "Off"}
            on = {"available": True, "active": True, "text": "On"}
            self.assertEqual(c.state("dnd"), off)
            self.assertEqual(c.state("night_light"), on)
            c.execute("dnd")
            c.execute("night_light")
            c.execute("lock")
        self.assertIn(("omarchy-shell", "notifications", "setDnd", "on"), calls)
        self.assertIn(("omarchy-toggle-nightlight",), calls)
        lock = ("org.freedesktop.login1.Session", "Lock")
        self.assertEqual(c.call.call_args.args[3:5], lock)
        c.setting.assert_not_called()

    def test_without_notification_daemon_is_unavailable(self):
        with (
            patch("system_controls.run", return_value=None),
            patch("system_controls.shutil.which", return_value=None),
        ):
            self.assertFalse(Controls().state("dnd")["available"])
            self.assertFalse(Controls().state("night_light")["available"])
