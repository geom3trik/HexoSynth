use crate::uimsg_queue::{Msg};
use crate::state::State;
use crate::UIParams;

use hexotk::{MButton};
use hexotk::widgets::{
    DialogModel,
};
use hexodsp::{Matrix, CellDir};
use keyboard_types::Key;
use hexodsp::matrix::MatrixError;
use hexodsp::matrix_repr::save_patch_to_file;

use std::rc::Rc;
use std::cell::RefCell;

pub struct Actions<'a, 'b, 'c> {
    pub state:      &'a mut State,
    pub dialog:     Rc<RefCell<DialogModel>>,
    pub matrix:     &'b mut Matrix,
    pub ui_params:  &'c mut UIParams,
}

impl Actions<'_, '_, '_> {
    pub fn save_patch(&mut self) {
        use hexodsp::matrix_repr::save_patch_to_file;

        let diag = self.dialog.clone();

        if catch_err_dialog(self.dialog.clone(), || {
            match save_patch_to_file(self.matrix, "init.hxy") {
                Ok(_) => Ok(()),
                Err(e) => Err(PatchSaveError {
                    path:  "init.hxy".to_string(),
                    error: e
                }.into()),
            }
        }) {
            diag.borrow_mut().open(
                &format!("Patch saved!\nPatch saved successful to 'init.hxy'!"),
                Box::new(|_| ()));
        }
    }

    pub fn toggle_help(&mut self) {
        self.state.toggle_help();
    }

    pub fn escape_dialogs(&mut self) {
        if self.state.show_help {
            self.state.toggle_help();
        }
    }

    pub fn map_messages_to_actions(&mut self, msg: &Msg) {
        match msg {
            Msg::CellDragged { btn, pos_a, pos_b } => {
                // Left & pos_a exists & pos_b empty
                //  => move/swap cell
                // Left & pos_a empty & pos_b exists
                //  => open cell selection dialog for one node
                //  => connect the default input
                // Left & pos_a empty & pos_b empty & adjacent
                //  => open cell selection dialog for two NodeIds
                //  => connect the default output to default input
                //     default is always: first input, first output
                // Left & pos_a exists & pos_b exists & adjacent
                //  => open connection selection dialog for out => inp
                // Left & pos_a exists & pos_b exists & NOT adjacent
                //  => take pos_a as output, pos_b as input
                //  => search an empty input for pos_b
                //  => copy cell at pos_a to that empty cell
                //  => open connection dialog for out => inp

                // Right & pos_a exists & pos_b empty & NOT adjacent
                //  => copy cell, but with empty ports
                // Right & pos_a exists & pos_b empty & adjacent
                //  => copy cell, but with empty ports, open port connection dialog
                // Right & pos_a exists & pos_b exists & adjacent
                //  => open connection menu for both
                // Right & pos_a empty & pos_b empty & adjacent
                //  => ????
                // Right & pos_a empty & pos_b exists
                //  => Delete pos_b
                // Right & pos_a exists & pos_b exists & NOT adjacent
                //  => take pos_a as output, pos_b as input
                //  => search an empty input for pos_b
                //  => NEW INSTANCE NodeId at pos_a to that empty cell
                //  => open connection dialog for out => inp

                let (src_cell, dst_cell) = (
                    self.matrix.get_copy(pos_a.0, pos_a.1),
                    self.matrix.get_copy(pos_b.0, pos_b.1)
                );

                // get_copy returns None for cells outside the matrix
                let src_cell =
                    if let Some(src_cell) = src_cell { src_cell }
                    else { return; };
                let dst_cell =
                    if let Some(dst_cell) = dst_cell { dst_cell }
                    else { return; };

                let adjacent = CellDir::are_adjacent(*pos_a, *pos_b);

                println!("DRAG CELL! {:?} {:?}", btn, msg);

                let src_is_output =
                    if let Some(dir) = adjacent { dir.is_output() }
                    else { false };

                let src =
                    if src_cell.node_id() == NodeId::Nop { None }
                    else { Some(src_cell) };
                let dst =
                    if dst_cell.node_id() == NodeId::Nop { None }
                    else { Some(dst_cell) };

                match (*btn, src, dst, adjacent, src_is_output) {
                      (MButton::Left, Some(_), None, _, _)
                    | (MButton::Left, None, Some(_), _, _) => {
                        catch_err_dialog(self.dialog.clone(), || {
                            self.matrix.change_matrix(|m| {
                                m.place(pos_b.0, pos_b.1, src_cell);
                                m.place(pos_a.0, pos_a.1, dst_cell);
                            })?;
                            m.sync()?;
                        });
                    },
                    (MButton::Left, Some(src), Some(dst), Some(dir), io) => {
                        println!("OPEN MENU!!!!! {:?} aisout={}",
                            dir, io);
                    },
                    (_, _, _, _, _) => (),
                }
            },
            Msg::Key { key } => {
                match key {
                    Key::F1 => self.toggle_help(),
                    Key::F4 => { self.save_patch(); },
                    Key::Escape => { self.escape_dialogs(); },
                    _ => {
                        println!("UNHANDLED KEY: {:?}", key);
                    }
                }
            },
            Msg::UIBtn { id } => {
                match *id {
                    ATNID_HELP_BUTTON => self.toggle_help(),
                    _ => {}
                }
            }
        }
    }
}

pub fn catch_err_dialog<F>(dialog: Rc<RefCell<DialogModel>>, mut f: F) -> bool
    where F: FnMut() -> Result<(), DialogMessage>
{
    match f() {
        Err(DialogMessage::PatchSaveError(err)) => {
            dialog.borrow_mut().open(
                &format!("Patch Saving failed!\n\
                    Path: {}\n\
                    Error: {}\n", err.path, err.error),
                Box::new(|_| ()));
        },
        Err(DialogMessage::IOError(err)) => {
            dialog.borrow_mut().open(
                &format!("An Unknown I/O Error Occured!\n\
                    Error: {}\n", err),
                Box::new(|_| ()));
        },
        Err(DialogMessage::MatrixError(err)) => {
            match err {
                MatrixError::CycleDetected => {
                    dialog.borrow_mut().open(
                        &"Cycle Detected!\n\
                            HexoSynth does not allow to create cyclic configurations.\n\
                            \n\
                            For feedback please use the nodes:\n\
                            * 'FbWr' (Feedback Writer)\n\
                            * 'FbRd' (Feedback Reader)",
                        Box::new(|_| ()));
                },
                MatrixError::DuplicatedInput { output1, output2 } => {
                    dialog.borrow_mut().open(
                        &format!("Unjoined Outputs Detected!\n\
                            It's not possible to assign to an input port twice.\n\
                            Please use a mixer or some other kind of node to join the outputs.\n\
                            \n\
                            Conflicting Outputs:\n\
                            * {} {}, port {}\n\
                            * {} {}, port {}",
                            output1.0.name(),
                            output1.0.instance(),
                            output1.0.out_name_by_idx(output1.1).unwrap_or("???"),
                            output2.0.name(),
                            output2.0.instance(),
                            output2.0.out_name_by_idx(output2.1).unwrap_or("???")),
                        Box::new(|_| ()));
                }
            }
        },
        Ok(_) => { return true; }
    }

    false
}

pub struct PatchSaveError {
    path:   String,
    error:  std::io::Error,
}

pub enum DialogMessage {
    MatrixError(MatrixError),
    IOError(std::io::Error),
    PatchSaveError(PatchSaveError),
}

impl From<MatrixError> for DialogMessage {
    fn from(error: MatrixError) -> Self {
        DialogMessage::MatrixError(error)
    }
}

impl From<std::io::Error> for DialogMessage {
    fn from(error: std::io::Error) -> Self {
        DialogMessage::IOError(error)
    }
}

impl From<PatchSaveError> for DialogMessage {
    fn from(error: PatchSaveError) -> Self {
        DialogMessage::PatchSaveError(error)
    }
}

