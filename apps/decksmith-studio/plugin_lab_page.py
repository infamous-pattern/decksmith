"""Homebridge connection manager. One worker; poll only while visible."""
from concurrent.futures import ThreadPoolExecutor
from gi.repository import Adw,Gtk,GLib
from i18n import gettext as tr
from plugin_lab_client import LabClient

STATUS={
 'not_configured':('Not configured','Set up and save a connection to resume assigned controls.'),
 'connected':('Connected','Ready for assigned keys and dials.'),
 'disabled':('Disabled','Assignments are preserved. Devices stay at their last settings.'),
 'connecting':('Working…','Please wait for the current connection or device operation.'),
 'disconnected':('Connection lost','Homebridge is unavailable. Check the server, then reconnect. No commands will be replayed.'),
 'device_unavailable':('Device unavailable','The selected accessory could not be read or no longer supports that action. Other assignments remain available.'),
 'unconfirmed':('Command unconfirmed','A device command could not be confirmed. Check its physical state, then reconnect. The command will not be retried.'),
 'authentication_required':('Sign-in required','Open Connection setup below to sign in or supply a new two-factor code.'),
 'settings_error':('Preference not saved','The enable/disable preference could not be saved. Its previous setting remains in effect.'),
}

class PluginLabPage(Gtk.ScrolledWindow):
 def __init__(self,connection_file):
  super().__init__(hscrollbar_policy=Gtk.PolicyType.NEVER)
  self.add_css_class('section-plugins');self.pool=ThreadPoolExecutor(max_workers=1,thread_name_prefix='homebridge-manager')
  self.pending=False;self.setup_pending=False;self.closed=False;self.timer=None;self.state=None;self.client=None;self.path=connection_file
  clamp=Adw.Clamp(maximum_size=740);self.set_child(clamp)
  box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=22)
  for edge in ('top','bottom','start','end'):getattr(box,'set_margin_'+edge)(22)
  clamp.set_child(box)
  heading=Gtk.Label(label=tr('Plugins'),xalign=0);heading.add_css_class('title-1');box.append(heading)
  group=Adw.PreferencesGroup(title=tr('Homebridge'),description=tr('OpenHomeB · Experimental integration'));box.append(group)
  self.server=Adw.ActionRow(title=tr('Server'),subtitle=tr('Waiting for connection…'));group.add(self.server)
  self.health=Adw.ActionRow(title=tr('Connection'),subtitle=tr('Checking…'));group.add(self.health)
  self.count=Adw.ActionRow(title=tr('Discovered functions'),subtitle='—');group.add(self.count)
  self.enabled_row=Adw.ActionRow(title=tr('Homebridge integration'),subtitle=tr('Disabling preserves every key and dial assignment.'));self.enabled_row.set_subtitle_lines(0);group.add(self.enabled_row)
  self.toggle=Gtk.Button(label=tr('Enable'),valign=Gtk.Align.CENTER);self.toggle.connect('clicked',self.toggle_clicked);self.enabled_row.add_suffix(self.toggle)
  self.toggle.set_tooltip_text(tr('Save the enabled state without changing any physical device.'))
  self.review_rows=[];self.review_value=object();self.removing=False
  self.setup_status=None;self.edit_generation=0;self.test_generation=None;self.setup_dirty=False;self.setup_synced=False;self.candidate=None;self.setup_syncing=False
  setup_group=Adw.PreferencesGroup();box.append(setup_group)
  self.setup_section=Adw.ExpanderRow(title=tr('Connection setup'),subtitle=tr('Test a server before saving it'));setup_group.add(self.setup_section)
  self.address=Adw.EntryRow(title=tr('Server address'));self.setup_section.add_row(self.address)
  self.username=Adw.EntryRow(title=tr('Username (if required)'));self.setup_section.add_row(self.username)
  self.password=Adw.PasswordEntryRow(title=tr('Password'));self.password.set_tooltip_text(tr('Leave blank to use the saved password'));self.setup_section.add_row(self.password)
  self.otp=Adw.PasswordEntryRow(title=tr('Two-factor code'));self.otp.set_visible(False);self.setup_section.add_row(self.otp)
  for entry in (self.address,self.username,self.password,self.otp):entry.connect('changed',self.setup_changed)
  buttons=Gtk.FlowBox(selection_mode=Gtk.SelectionMode.NONE,min_children_per_line=1,max_children_per_line=2,column_spacing=12,row_spacing=8,margin_top=8,margin_bottom=8)
  self.test_connection=Gtk.Button(label=tr('Test Connection'));self.test_connection.connect('clicked',lambda *_:self.submit_setup(False));buttons.append(self.test_connection)
  self.save_connection=Gtk.Button(label=tr('Save Connection'));self.save_connection.connect('clicked',lambda *_:self.submit_setup(True));buttons.append(self.save_connection);self.setup_section.add_row(buttons)
  self.setup_message=Gtk.Label(label=tr('Passwords are stored in GNOME Keyring. Codes are never saved. Prefer HTTPS when available.'),xalign=0,wrap=True);self.setup_section.add_row(self.setup_message)
  self.message=Gtk.Label(label=tr('Reading connection status…'),xalign=0,wrap=True);self.message.set_selectable(True);box.append(self.message)
  row=Gtk.Box(spacing=12);box.append(row)
  self.reconnect=Gtk.Button(label=tr('Reconnect'),tooltip_text=tr('Read current device state without replaying previous commands.'));self.reconnect.connect('clicked',lambda *_:self.fetch('reconnect'));row.append(self.reconnect)
  self.refresh=Gtk.Button(label=tr('Refresh status'));self.refresh.connect('clicked',lambda *_:self.fetch(review=True));row.append(self.refresh)
  box.append(Gtk.Label(label=tr('Assign accessories in Keys & Dials → Homebridge accessory. Lights and sockets default to toggle; cameras and sensors show status.'),xalign=0,wrap=True))
  box.append(Gtk.Label(label=tr('Changing servers preserves assignments. Devices absent from the new server show unavailable. Two-factor sign-in may be needed again after a restart or session expiry.'),xalign=0,wrap=True))
  review_group=Adw.PreferencesGroup(title=tr('Access and assignments'));box.append(review_group)
  access=Adw.ExpanderRow(title=tr('Homebridge access'),subtitle=tr('What this integration can read and control'));review_group.add(access)
  for text in (
   'Reads the accessory list and device status from your configured Homebridge server. Assigned keys and dials can change power, brightness or supported fan speed.',
   'Cameras, sensors, speakers and microphones currently provide status only. This integration does not stream camera video or microphone audio.',
   'The saved account determines Homebridge access. Assignment choices are not per-device security permissions. This experimental plugin runs as your user and is not an operating-system sandbox.',
   'Passwords use GNOME Keyring; login sessions and two-factor codes are held in memory. Removing a connection does not uninstall the plugin or change Homebridge settings.'):
   label=Gtk.Label(label=tr(text),xalign=0,wrap=True,margin_top=8,margin_bottom=8);access.add_row(label)
  self.assignments_section=Adw.ExpanderRow(title=tr('Saved assignments'),subtitle=tr('Loading…'));review_group.add(self.assignments_section)
  self.removal_group=Adw.PreferencesGroup(title=tr('Remove connection'));box.append(self.removal_group)
  self.remove_button=Gtk.Button(label=tr('Remove Connection'),halign=Gtk.Align.START)
  self.remove_button.add_css_class('destructive-action');self.remove_button.connect('clicked',self.show_removal);self.removal_group.add(self.remove_button)
  self.removal_message=Gtk.Label(xalign=0,wrap=True);self.removal_group.add(self.removal_message)
  self.removal_box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=12);self.removal_box.set_visible(False);self.removal_group.add(self.removal_box)
  self.removal_box.append(Gtk.Label(label=tr('Remove this Homebridge connection? Assigned keys and dials will remain saved but unavailable. Devices keep their current settings.'),xalign=0,wrap=True))
  self.delete_password=Gtk.CheckButton();self.delete_password.set_child(Gtk.Label(label=tr('Also delete this connection’s saved password from GNOME Keyring'),wrap=True,xalign=0));self.removal_box.append(self.delete_password)
  self.removal_box.append(Gtk.Label(label=tr('Password deletion cannot be undone. Older passwords retained for backups are not deleted.'),xalign=0,wrap=True))
  confirmation=Gtk.Box(spacing=12);self.removal_box.append(confirmation)
  self.cancel_remove=Gtk.Button(label=tr('Cancel'));self.cancel_remove.connect('clicked',lambda *_:self.hide_removal());confirmation.append(self.cancel_remove)
  self.confirm_remove=Gtk.Button(label=tr('Remove Connection'));self.confirm_remove.add_css_class('destructive-action');self.confirm_remove.connect('clicked',self.remove_confirmed);confirmation.append(self.confirm_remove)
  self.connect('map',self.mapped);self.connect('unmap',self.unmapped);self.set_controls()
 def set_controls(self):
  idle=not self.pending and not self.closed
  valid=bool(self.state and not self.state['busy'])
  configured=bool(self.state and self.state['connection']!='not_configured')
  self.toggle.set_sensitive(idle and valid and configured)
  self.reconnect.set_sensitive(idle and valid and configured and self.state['enabled'])
  self.refresh.set_sensitive(idle)
  self.test_connection.set_sensitive(idle and valid)
  self.save_connection.set_sensitive(idle and valid and bool(self.candidate))
  # Read-only polling must never disable focused entries or disturb drafts.
  removable=bool(self.state and (configured or self.state.get('has_saved_password')))
  self.remove_button.set_sensitive(idle and valid and removable)
  self.confirm_remove.set_sensitive(idle and valid and removable)
  self.cancel_remove.set_sensitive(not self.setup_pending)
  self.delete_password.set_sensitive(not self.setup_pending and bool(self.state and self.state.get('has_saved_password')))
  for entry in (self.address,self.username,self.password,self.otp):entry.set_sensitive(not self.closed and not self.setup_pending)
 def show_removal(self,*_):
  self.removal_box.set_visible(True);self.delete_password.set_active(False)
 def hide_removal(self):
  self.removal_box.set_visible(False);self.delete_password.set_active(False)
 def remove_confirmed(self,*_):
  if self.pending or not self.state or self.state['busy']:return
  self.removing=True
  self.send_setup({'operation':'remove','confirm':True,'delete_password':self.delete_password.get_active()},tr('Removing connection…'))
 def show_review(self,rows):
  if rows==self.review_value:return
  self.review_value=rows
  for row in self.review_rows:self.assignments_section.remove(row)
  self.review_rows=[]
  self.assignments_section.set_subtitle(tr('Could not read saved assignments. Refresh status to retry.') if rows is None else str(len(rows))+' · '+tr('Saved layout; refresh after Save and Apply'))
  for title,subtitle in rows or []:
   row=Adw.ActionRow(title=title,subtitle=subtitle);row.set_title_lines(0);row.set_subtitle_lines(0)
   row.set_use_markup(False);self.assignments_section.add_row(row);self.review_rows.append(row)
 def setup_changed(self,*_):
  if self.setup_syncing:return
  self.edit_generation+=1;self.setup_dirty=True;self.candidate=None;self.set_controls()
 def submit_setup(self,apply):
  if self.pending or not self.state or self.state['busy']:return
  if not apply:self.test_generation=self.edit_generation
  data=({'operation':'apply','candidate':self.candidate} if apply else
        {'operation':'test','server':self.address.get_text(),'username':self.username.get_text(),'password':self.password.get_text(),'otp':self.otp.get_text()})
  self.send_setup(data,tr('Saving connection…') if apply else tr('Testing connection…'))
 def send_setup(self,data,message):
  self.pending=True;self.setup_pending=True;self.set_controls();self.setup_message.set_text(message)
  def work():
   if self.client is None:self.client=LabClient(self.path)
   if not self.client.setup(data)['ok']:raise RuntimeError('Setup busy')
   return self.client.request()
  def done(future):
   try:value=future.result()
   except Exception:value=None
   GLib.idle_add(self.complete,value)
  self.pool.submit(work).add_done_callback(done)
 def toggle_clicked(self,*_):
  if self.state:self.fetch('disable' if self.state['enabled'] else 'enable')
 def mapped(self,*_):
  if self.closed:return
  self.fetch(review=True)
  if self.timer is None:self.timer=GLib.timeout_add(2000,self.tick)
 def unmapped(self,*_):
  if self.timer is not None:GLib.source_remove(self.timer);self.timer=None
 def tick(self):
  self.fetch();return GLib.SOURCE_CONTINUE
 def fetch(self,command=None,review=False):
  if self.pending or self.closed:return
  if command and (not self.state or self.state['busy']):return
  self.pending=True;self.set_controls()
  if command:self.message.set_text(tr('Applying connection preference…') if command!='reconnect' else tr('Reconnecting…'))
  def work():
   if self.client is None:self.client=LabClient(self.path)
   if command and not self.client.request(command)['ok']:raise RuntimeError('Command rejected')
   value=self.client.request()
   if review:
    from plugin_review import read_review
    try:value['_review']=read_review(self.client)
    except Exception:value['_review']=None
   return value
  def done(future):
   try:value=future.result()
   except Exception:value=None
   GLib.idle_add(self.complete,value)
  self.pool.submit(work).add_done_callback(done)
 def complete(self,value):
  if self.closed:return GLib.SOURCE_REMOVE
  self.pending=False;self.state=value
  if value is None or not value['busy']:self.setup_pending=False
  if value is None:
   self.health.set_subtitle(tr('Background connection unavailable'));self.count.set_subtitle('—')
   self.message.set_text(tr('Start Background controls on Home, then refresh status. Saved assignments are unchanged.'))
  else:
   if '_review' in value:self.show_review(value['_review'])
   title,message=STATUS[value['connection']]
   self.server.set_subtitle(value['server'] or tr('No saved connection'));self.health.set_subtitle(tr(title));self.message.set_text(tr(message))
   self.count.set_subtitle(str(value['device_count']) if value['enabled'] else '—')
   self.toggle.set_label(tr('Disable') if value['enabled'] else tr('Enable'))
   if not self.setup_synced:
    self.setup_syncing=True;self.address.set_text(value['server']);self.username.set_text(value.get('username',''));self.setup_syncing=False;self.setup_synced=True
   result=value.get('setup',{});status=result.get('state','idle');previous_setup_status=self.setup_status;self.setup_status=status
   if self.removing or status=='removed':self.removal_message.set_text(result.get('message',''))
   elif status=='saved':self.removal_message.set_text('')
   if status!='idle':
    self.setup_message.set_text(result.get('message',''))
    self.otp.set_visible(status=='two_factor')
    if status=='tested' and not value['busy'] and self.test_generation==self.edit_generation and self.candidate!=result.get('candidate'):
     self.setup_syncing=True;self.address.set_text(result.get('server',''));self.username.set_text(result.get('username',''));
     if not result.get('username'):self.password.set_text('')
     self.setup_syncing=False
     self.setup_syncing=True;self.otp.set_text('');self.setup_syncing=False
     self.candidate=result.get('candidate')
    elif status!='tested':self.candidate=None
    elif self.test_generation!=self.edit_generation:self.setup_message.set_text(tr('Connection fields changed. Test again before saving.'))
    if status in ('saved','removed') and previous_setup_status!=status:
     self.setup_syncing=True;self.password.set_text('');self.otp.set_text('');self.setup_syncing=False
    if status=='removed' and self.removing and not value['busy']:
     self.removing=False;self.hide_removal();self.candidate=None
     self.setup_syncing=True;self.address.set_text('');self.username.set_text('');self.setup_syncing=False
    if status in ('saved','removed') and previous_setup_status!=status and not value['busy']:
     GLib.idle_add(lambda: self.fetch(review=True))

  self.set_controls();return GLib.SOURCE_REMOVE
 def close(self):
  if self.closed:return
  self.closed=True;self.password.set_text('');self.otp.set_text('');self.unmapped();self.pool.shutdown(wait=False,cancel_futures=True)
