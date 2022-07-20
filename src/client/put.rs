use stateright::actor::{Actor, Id};

use super::Request;

/// A client strategy that just puts at a single key into a map.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapSinglePutter {
    pub key: String,
    pub request_count: usize,
}

impl Actor for MapSinglePutter {
    type Msg = Request;

    type State = ();

    fn on_start(
        &self,
        id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        let index: usize = id.into();

        for i in 0..self.request_count {
            let unique_request_id = (i + 1) * index; // next will be 2 * index
            let value = 'A';
            let msg = Request::PutMap(unique_request_id, self.key.clone(), value.to_string());
            o.send(Id::from(0), msg);
        }
    }
}

/// A client strategy that just puts at the start of a list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListStartPutter {
    pub request_count: usize,
}

impl Actor for ListStartPutter {
    type Msg = Request;

    type State = ();

    fn on_start(
        &self,
        id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        let index: usize = id.into();

        for i in 0..self.request_count {
            let unique_request_id = (i + 1) * index; // next will be 2 * index
            let value = 'A';
            let msg = Request::PutList(unique_request_id, 0, value.to_string());
            o.send(Id::from(0), msg);
        }
    }
}
