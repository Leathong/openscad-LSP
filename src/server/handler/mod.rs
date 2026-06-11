use std::error::Error;

use lsp_server::{ExtractError, Message, Response};
use lsp_types::{
    CompletionRequest, DefinitionRequest, DidChangeConfigurationNotification,
    DidChangeTextDocumentNotification, DidCloseTextDocumentNotification,
    DidOpenTextDocumentNotification, DidSaveTextDocumentNotification, DocumentFormattingRequest,
    DocumentSymbolRequest, HoverRequest, PrepareRenameRequest, RenameRequest,
};
use serde_json::json;

use crate::{Server, utils::*};

use super::LoopAction;

pub(crate) mod notification;
pub(crate) mod request;

impl Server {
    pub(crate) fn respond(&self, mut resp: Response) {
        if resp.result.is_none() {
            resp.result = Some(json!(null))
        }
        // log_to_console!("{:?}\n\n", &resp);
        self.connection
            .sender
            .send(Message::Response(resp))
            .unwrap()
    }

    pub(crate) fn notify(&self, notif: lsp_server::Notification) {
        self.connection
            .sender
            .send(Message::Notification(notif))
            .unwrap()
    }

    pub(crate) fn handle_message(
        &mut self,
        msg: Message,
    ) -> Result<LoopAction, Box<dyn Error + Sync + Send>> {
        match msg {
            Message::Request(req) => {
                if self.connection.handle_shutdown(&req)? {
                    return Ok(LoopAction::Exit);
                }

                macro_rules! proc_req {
                    ($request:ident, $req_type:ty, $method:ident) => {
                        match cast_request::<$req_type>($request) {
                            Ok((id, params)) => {
                                self.$method(id, params);
                                return Ok(LoopAction::Continue);
                            }
                            Err(error) => match error {
                                ExtractError::MethodMismatch(req) => req,
                                ExtractError::JsonError { method, error } => {
                                    err_to_console!("method: {} error: {}\n", method, error);
                                    return Ok(LoopAction::Continue);
                                }
                            },
                        }
                    };
                }

                let req = proc_req!(req, HoverRequest, handle_hover);
                let req = proc_req!(req, CompletionRequest, handle_completion);
                let req = proc_req!(req, DefinitionRequest, handle_definition);
                let req = proc_req!(req, DocumentSymbolRequest, handle_document_symbols);
                let req = proc_req!(req, DocumentFormattingRequest, handle_formatting);
                let req = proc_req!(req, PrepareRenameRequest, handle_prepare_rename);
                let req = proc_req!(req, RenameRequest, handle_rename);
                err_to_console!("unknown request: {:?}", req);
            }
            Message::Response(resp) => {
                err_to_console!("got response: {:?}", resp);
            }
            Message::Notification(noti) => {
                macro_rules! proc {
                    ($noti:ident, $noti_type:ty, $method:ident) => {
                        match cast_notification::<$noti_type>($noti) {
                            Ok(params) => {
                                self.$method(params);
                                return Ok(LoopAction::Continue);
                            }
                            Err(error) => match error {
                                ExtractError::MethodMismatch(noti) => noti,
                                ExtractError::JsonError { method, error } => {
                                    err_to_console!("method: {} error: {}\n", method, error);
                                    return Ok(LoopAction::Exit);
                                }
                            },
                        }
                    };
                }

                let noti = proc!(
                    noti,
                    DidOpenTextDocumentNotification,
                    handle_did_open_text_document
                );
                let noti = proc!(
                    noti,
                    DidChangeTextDocumentNotification,
                    handle_did_change_text_document
                );
                let noti = proc!(
                    noti,
                    DidSaveTextDocumentNotification,
                    handle_did_save_text_document
                );
                let noti = proc!(
                    noti,
                    DidCloseTextDocumentNotification,
                    handle_did_close_text_document
                );
                let noti = proc!(
                    noti,
                    DidChangeConfigurationNotification,
                    handle_did_change_config
                );

                err_to_console!("unknown notification: {:?}", noti);
            }
        }
        Ok(LoopAction::Continue)
    }
}
