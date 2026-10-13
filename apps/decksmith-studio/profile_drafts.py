"""Bounded per-device edit state; no widgets, I/O or default-device fallback."""
from dataclasses import dataclass
from editor_model import Draft

@dataclass
class View:
    draft:Draft
    page:int
    key:int
    kind:str
    dial:int
    revision:int

class DraftShelf:
    def __init__(self):self.views={}
    def keep(self,identity,editor,revision):
        if identity not in self.views and len(self.views)>=8:raise ValueError('Too many device drafts')
        self.views[identity]=View(editor.draft,editor.page,editor.key,editor.control_kind,editor.dial_index,revision)
    def restore(self,identity,layout,token):
        view=self.views.get(identity)
        if view is None:return None
        if not view.draft.dirty:
            # Undoing back to the saved layout is clean, but redo/copy state is
            # still meaningful. Keep it if the remote saved layout is unchanged.
            if view.draft.saved==layout and (view.draft.undo_stack or view.draft.redo_stack or view.draft.clipboard):return view
            return None
        # New metadata/page contexts may be rebased if saved layout bytes have
        # the same JSON values. A remote layout change keeps the old expected
        # revision, so this draft cannot silently overwrite it.
        if view.draft.saved!=layout:token['revision']=view.revision
        return view
