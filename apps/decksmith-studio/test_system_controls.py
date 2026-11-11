import os
import subprocess
import sys
import tempfile
import time
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
            patch("system_controls.run_action", side_effect=lambda *argv: calls.append(argv) or True),
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

    def test_omarchy_night_light_failure_is_reported(self):
        with (patch("system_controls.run", return_value=None),
              patch("system_controls.run_action", return_value=False),
              patch("system_controls.shutil.which", return_value="/usr/bin/x")):
            with self.assertRaisesRegex(ValueError, "Control unavailable"):
                Controls().execute("night_light")

    def test_omarchy_lock_uses_shell_and_rejects_failure(self):
        def helper(*argv):
            if argv == ("omarchy-shell", "lock", "status"):
                return '{"passwordPam":true,"realScreens":1}'
            if argv == ("omarchy-shell", "lock", "lock"):
                return "ok"
            return None

        with patch("system_controls.run", side_effect=helper) as run:
            controls = Controls()
            self.assertTrue(controls.state("lock")["available"])
            controls.execute("lock")
            self.assertIn(unittest.mock.call("omarchy-shell", "lock", "lock"), run.call_args_list)
        with patch("system_controls.run", side_effect=lambda *args: helper(*args) if args[-1] == "status" else "failed"):
            with self.assertRaisesRegex(ValueError, "Control unavailable"):
                Controls().execute("lock")

    def test_no_locker_does_not_claim_lock_is_available(self):
        with patch("system_controls.run", return_value=None):
            self.assertFalse(Controls().state("lock")["available"])
            with self.assertRaisesRegex(ValueError, "Control unavailable"):
                Controls().execute("lock")


class ActionProcessTests(unittest.TestCase):
    def test_action_does_not_wait_for_a_background_child_to_close_output(self):
        from system_controls import run_action

        # Model Omarchy's launcher: its child inherits stdout and stays alive
        # after the helper exits. Capturing a pipe would wait for that child.
        with tempfile.TemporaryDirectory() as directory:
            pidfile = os.path.join(directory, "child.pid")
            code = (
                "import subprocess,sys; "
                "p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(8)']); "
                "open(sys.argv[1], 'w').write(str(p.pid))"
            )
            try:
                start = time.monotonic()
                self.assertTrue(run_action(sys.executable, "-c", code, pidfile))
                self.assertLess(time.monotonic() - start, 4)
            finally:
                if os.path.exists(pidfile):
                    try:
                        with open(pidfile) as child_pid:
                            os.kill(int(child_pid.read()), 15)
                    except ProcessLookupError:
                        pass

    def test_action_timeout_is_failure(self):
        from system_controls import run_action

        with patch("system_controls.subprocess.run", side_effect=subprocess.TimeoutExpired("helper", 5)):
            self.assertFalse(run_action(sys.executable, "-c", "pass"))

    def test_query_timeout_is_unavailable(self):
        from system_controls import run

        with patch("system_controls.subprocess.run", side_effect=subprocess.TimeoutExpired("helper", 5)):
            self.assertIsNone(run(sys.executable, "-c", "pass"))
