//! Atom cache for the ICCCM/EWMH subset implemented by Etapa 04.

use crate::{X11BackendError, operation_error};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, ConnectionExt as _};

#[derive(Clone, Debug)]
pub(crate) struct Atoms {
    pub utf8_string: Atom,
    pub wm_protocols: Atom,
    pub wm_delete_window: Atom,
    pub wm_take_focus: Atom,
    pub net_supported: Atom,
    pub net_supporting_wm_check: Atom,
    pub net_wm_name: Atom,
    pub net_active_window: Atom,
    pub net_client_list: Atom,
    pub net_client_list_stacking: Atom,
    pub net_close_window: Atom,
    pub net_wm_state: Atom,
    pub net_wm_state_maximized_vert: Atom,
    pub net_wm_state_maximized_horz: Atom,
    pub net_wm_state_fullscreen: Atom,
}

impl Atoms {
    /// Interns the complete atom set once so the event loop never performs
    /// repeated name lookups on latency-sensitive paths.
    pub fn load<C: Connection>(conn: &C) -> Result<Self, X11BackendError> {
        Ok(Self {
            utf8_string: intern(conn, b"UTF8_STRING")?,
            wm_protocols: intern(conn, b"WM_PROTOCOLS")?,
            wm_delete_window: intern(conn, b"WM_DELETE_WINDOW")?,
            wm_take_focus: intern(conn, b"WM_TAKE_FOCUS")?,
            net_supported: intern(conn, b"_NET_SUPPORTED")?,
            net_supporting_wm_check: intern(conn, b"_NET_SUPPORTING_WM_CHECK")?,
            net_wm_name: intern(conn, b"_NET_WM_NAME")?,
            net_active_window: intern(conn, b"_NET_ACTIVE_WINDOW")?,
            net_client_list: intern(conn, b"_NET_CLIENT_LIST")?,
            net_client_list_stacking: intern(conn, b"_NET_CLIENT_LIST_STACKING")?,
            net_close_window: intern(conn, b"_NET_CLOSE_WINDOW")?,
            net_wm_state: intern(conn, b"_NET_WM_STATE")?,
            net_wm_state_maximized_vert: intern(conn, b"_NET_WM_STATE_MAXIMIZED_VERT")?,
            net_wm_state_maximized_horz: intern(conn, b"_NET_WM_STATE_MAXIMIZED_HORZ")?,
            net_wm_state_fullscreen: intern(conn, b"_NET_WM_STATE_FULLSCREEN")?,
        })
    }

    pub fn supported(&self) -> [Atom; 10] {
        [
            self.net_supporting_wm_check,
            self.net_wm_name,
            self.net_active_window,
            self.net_client_list,
            self.net_client_list_stacking,
            self.net_close_window,
            self.net_wm_state,
            self.net_wm_state_maximized_vert,
            self.net_wm_state_maximized_horz,
            self.net_wm_state_fullscreen,
        ]
    }
}

fn intern<C: Connection>(conn: &C, name: &[u8]) -> Result<Atom, X11BackendError> {
    conn.intern_atom(false, name)
        .map_err(operation_error)?
        .reply()
        .map(|reply| reply.atom)
        .map_err(operation_error)
}
