use iced::Task;

use crate::{update, EndpointKvPair, MHeader, MQueryParam, Message, Request, State};

use super::{
    crud::{
        header::{
            delete_header, update_header, update_header_key, update_header_on, update_header_value,
        },
        query::{
            delete_query_param, update_query_param_key, update_query_param_on,
            update_query_param_value,
        },
    },
    db::get_db,
};

pub fn message_query_param(state: &mut State, message: MQueryParam) -> Task<Message> {
    match message {
        MQueryParam::AddQueryParam() => match &mut state.draft_request {
            Some(q) => {
                q.query_params.push(EndpointKvPair {
                    id: q.query_params.len() as u64,
                    parent_response_id: 0,
                    key: "".to_string(),
                    value: "".to_string(),
                    on: true,
                });
                Task::none()
            }
            None => {
                state.draft_request = Some(Request {
                    query_params: [EndpointKvPair {
                        id: 0,
                        parent_response_id: 0,
                        key: "".to_string(),
                        value: "".to_string(),
                        on: true,
                    }]
                    .to_vec(),
                    headers: [].to_vec(),
                });
                Task::none()
            }
        },
        MQueryParam::SetQueryParamContent(id, content) => match &mut state.draft_request {
            Some(q) => {
                if let Some(elem) = q.query_params.iter_mut().find(|it| it.id == id) {
                    elem.value = content.clone();
                };
                Task::none()
            }
            None => {
                update_query_param_value(&get_db().lock().unwrap(), id, &content).unwrap();
                update(state, Message::RefetchDb)
            }
        },
        MQueryParam::SetQueryParamKey(id, content) => match &mut state.draft_request {
            Some(q) => {
                if let Some(elem) = q.query_params.iter_mut().find(|it| it.id == id) {
                    elem.key = content.clone();
                };
                Task::none()
            }
            None => {
                update_query_param_key(&get_db().lock().unwrap(), id, &content).unwrap();
                update(state, Message::RefetchDb)
            }
        },
        MQueryParam::ToggleQueryParamIsOn(id) => match &mut state.draft_request {
            Some(q) => {
                if let Some(elem) = q.query_params.iter_mut().find(|it| it.id == id) {
                    elem.on = !elem.on;
                };
                Task::none()
            }
            None => {
                update_query_param_on(&get_db().lock().unwrap(), id).unwrap();
                update(state, Message::RefetchDb)
            }
        },
        MQueryParam::DeleteQueryParam(id) => {
            match &mut state.draft_request {
                Some(draft) => match draft.query_params.iter().position(|it| it.id == id) {
                    Some(found) => {
                        draft.query_params.remove(found);
                    }
                    None => {}
                },
                None => {
                    delete_query_param(&get_db().lock().unwrap(), id).unwrap();
                }
            }
            update(state, Message::RefetchDb)
        }
    }
}

pub fn message_header(state: &mut State, message: MHeader) -> Task<Message> {
    match message {
        MHeader::AddHeader() => match &mut state.draft_request {
            Some(q) => {
                q.headers.push(EndpointKvPair {
                    id: q.headers.len() as u64,
                    parent_response_id: 0,
                    key: "".to_string(),
                    value: "".to_string(),
                    on: true,
                });
                Task::none()
            }
            None => {
                state.draft_request = Some(Request {
                    headers: [EndpointKvPair {
                        id: 0,
                        parent_response_id: 0,
                        key: "".to_string(),
                        value: "".to_string(),
                        on: true,
                    }]
                    .to_vec(),
                    query_params: [].to_vec(),
                });
                Task::none()
            }
        },
        MHeader::SetHeaderContent(id, content) => match &mut state.draft_request {
            Some(q) => {
                if let Some(elem) = q.headers.iter_mut().find(|it| it.id == id) {
                    elem.value = content.clone();
                };
                Task::none()
            }
            None => {
                update_header_value(&get_db().lock().unwrap(), id, &content).unwrap();
                update(state, Message::RefetchDb)
            }
        },
        MHeader::SetHeaderKey(id, content) => match &mut state.draft_request {
            Some(q) => {
                if let Some(elem) = q.headers.iter_mut().find(|it| it.id == id) {
                    elem.key = content.clone();
                };
                Task::none()
            }
            None => {
                update_header_key(&get_db().lock().unwrap(), id, &content).unwrap();
                update(state, Message::RefetchDb)
            }
        },
        MHeader::ToggleHeaderIsOn(id) => match &mut state.draft_request {
            Some(q) => {
                if let Some(elem) = q.headers.iter_mut().find(|it| it.id == id) {
                    elem.on = !elem.on;
                };
                Task::none()
            }
            None => {
                update_header_on(&get_db().lock().unwrap(), id).unwrap();
                update(state, Message::RefetchDb)
            }
        },
        MHeader::DeleteHeader(id) => {
            match &mut state.draft_request {
                Some(draft) => match draft.headers.iter().position(|it| it.id == id) {
                    Some(found) => {
                        draft.headers.remove(found);
                    }
                    None => {}
                },
                None => {
                    delete_header(&get_db().lock().unwrap(), id).unwrap();
                }
            }
            update(state, Message::RefetchDb)
        }
    }
}
