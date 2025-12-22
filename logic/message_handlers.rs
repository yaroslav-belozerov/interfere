use iced::Task;

use crate::{EndpointKvPair, MHeader, MQueryParam, Message, Request, State, update};

use super::{
    crud::{
        header::{delete_header, update_header_key, update_header_value},
        query::{delete_query_param, update_query_param_key, update_query_param_value},
    },
    db::get_db,
};

pub fn message_query_param(state: &mut State, message: MQueryParam) -> Task<Message> {
    match message {
        MQueryParam::AddQueryParam() => match &mut state.copy_request {
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
                match state.selected_endpoint {
                    Some(_) => {
                        state.copy_request = Some(Request {
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
                    }
                    None => {
                        state.draft_request.query_params.push(EndpointKvPair {
                            id: match state.draft_request.query_params.last() {
                                Some(it) => it.id + 1,
                                None => 0,
                            },
                            parent_response_id: 0,
                            key: "".to_string(),
                            value: "".to_string(),
                            on: true,
                        });
                    }
                }
                Task::none()
            }
        },
        MQueryParam::SetQueryParamContent(id, content) => match &mut state.copy_request {
            Some(q) => {
                if let Some(elem) = q.query_params.iter_mut().find(|it| it.id == id) {
                    elem.value = content.clone();
                };
                Task::none()
            }
            None => match state.selected_endpoint {
                Some(_) => {
                    update_query_param_value(&get_db().lock().unwrap(), id, &content).unwrap();
                    update(state, Message::RefetchDb)
                }
                None => {
                    if let Some(elem) = state
                        .draft_request
                        .query_params
                        .iter_mut()
                        .find(|it| it.id == id)
                    {
                        elem.value = content.clone();
                    };
                    Task::none()
                }
            },
        },
        MQueryParam::SetQueryParamKey(id, content) => match &mut state.copy_request {
            Some(q) => {
                if let Some(elem) = q.query_params.iter_mut().find(|it| it.id == id) {
                    elem.key = content.clone();
                };
                Task::none()
            }
            None => match state.selected_endpoint {
                Some(_) => {
                    update_query_param_key(&get_db().lock().unwrap(), id, &content).unwrap();
                    update(state, Message::RefetchDb)
                }
                None => {
                    if let Some(elem) = state
                        .draft_request
                        .query_params
                        .iter_mut()
                        .find(|it| it.id == id)
                    {
                        elem.key = content.clone();
                    };
                    Task::none()
                }
            },
        },
        MQueryParam::ToggleQueryParamIsOn(id) => match &mut state.copy_request {
            Some(q) => {
                if let Some(elem) = q.query_params.iter_mut().find(|it| it.id == id) {
                    elem.on = !elem.on;
                };
                Task::none()
            }
            None => match state.selected_endpoint {
                Some(_) => Task::none(),
                None => {
                    if let Some(elem) = state
                        .draft_request
                        .query_params
                        .iter_mut()
                        .find(|it| it.id == id)
                    {
                        elem.on = !elem.on;
                    };
                    Task::none()
                }
            },
        },
        MQueryParam::DeleteQueryParam(id) => {
            match &mut state.copy_request {
                Some(draft) => match draft.query_params.iter().position(|it| it.id == id) {
                    Some(found) => {
                        draft.query_params.remove(found);
                    }
                    None => {}
                },
                None => match state.selected_endpoint {
                    Some(_) => {
                        delete_query_param(&get_db().lock().unwrap(), id).unwrap();
                    }
                    None => {
                        match state
                            .draft_request
                            .query_params
                            .iter()
                            .position(|it| it.id == id)
                        {
                            Some(found) => {
                                state.draft_request.query_params.remove(found);
                            }
                            None => {}
                        };
                    }
                },
            }
            update(state, Message::RefetchDb)
        }
    }
}

pub fn message_header(state: &mut State, message: MHeader) -> Task<Message> {
    match message {
        MHeader::AddHeader() => match &mut state.copy_request {
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
                match state.selected_endpoint {
                    Some(_) => {
                        state.copy_request = Some(Request {
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
                    }
                    None => {
                        state.draft_request.headers.push(EndpointKvPair {
                            id: match state.draft_request.headers.last() {
                                Some(it) => it.id + 1,
                                None => 0,
                            },
                            parent_response_id: 0,
                            key: "".to_string(),
                            value: "".to_string(),
                            on: true,
                        });
                    }
                }
                Task::none()
            }
        },
        MHeader::SetHeaderContent(id, content) => match &mut state.copy_request {
            Some(q) => {
                if let Some(elem) = q.headers.iter_mut().find(|it| it.id == id) {
                    elem.value = content.clone();
                };
                Task::none()
            }
            None => match state.selected_endpoint {
                Some(_) => {
                    update_header_value(&get_db().lock().unwrap(), id, &content).unwrap();
                    update(state, Message::RefetchDb)
                }
                None => {
                    if let Some(elem) = state
                        .draft_request
                        .headers
                        .iter_mut()
                        .find(|it| it.id == id)
                    {
                        elem.value = content.clone();
                    };
                    Task::none()
                }
            },
        },
        MHeader::SetHeaderKey(id, content) => match &mut state.copy_request {
            Some(q) => {
                if let Some(elem) = q.headers.iter_mut().find(|it| it.id == id) {
                    elem.key = content.clone();
                };
                Task::none()
            }
            None => match state.selected_endpoint {
                Some(_) => {
                    update_header_key(&get_db().lock().unwrap(), id, &content).unwrap();
                    update(state, Message::RefetchDb)
                }
                None => {
                    if let Some(elem) = state
                        .draft_request
                        .headers
                        .iter_mut()
                        .find(|it| it.id == id)
                    {
                        elem.key = content.clone();
                    };
                    Task::none()
                }
            },
        },
        MHeader::ToggleHeaderIsOn(id) => match &mut state.copy_request {
            Some(q) => {
                if let Some(elem) = q.headers.iter_mut().find(|it| it.id == id) {
                    elem.on = !elem.on;
                };
                Task::none()
            }
            None => match state.selected_endpoint {
                Some(_) => Task::none(),
                None => {
                    if let Some(elem) = state
                        .draft_request
                        .headers
                        .iter_mut()
                        .find(|it| it.id == id)
                    {
                        elem.on = !elem.on;
                    };
                    Task::none()
                }
            },
        },
        MHeader::DeleteHeader(id) => {
            match &mut state.copy_request {
                Some(draft) => match draft.headers.iter().position(|it| it.id == id) {
                    Some(found) => {
                        draft.headers.remove(found);
                    }
                    None => {}
                },
                None => match state.selected_endpoint {
                    Some(_) => {
                        delete_header(&get_db().lock().unwrap(), id).unwrap();
                    }
                    None => {
                        match state
                            .draft_request
                            .headers
                            .iter()
                            .position(|it| it.id == id)
                        {
                            Some(found) => {
                                state.draft_request.headers.remove(found);
                            }
                            None => {}
                        };
                    }
                },
            }
            update(state, Message::RefetchDb)
        }
    }
}
